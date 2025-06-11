use chrono::{Duration, NaiveDate, Utc};

use crate::models::{ScholarScore, SignalRecord};

pub fn score_signals(signals: &[SignalRecord], since_days: i64) -> Vec<ScholarScore> {
    let cutoff = Utc::now().date_naive() - Duration::days(since_days.max(1));
    let mut scores: std::collections::HashMap<uuid::Uuid, ScholarScore> =
        std::collections::HashMap::new();

    for signal in signals.iter() {
        if signal.occurred_at < cutoff {
            continue;
        }

        let days_ago = (Utc::now().date_naive() - signal.occurred_at).num_days();
        let weight = recency_weight(days_ago);
        let entry = scores.entry(signal.scholar_id).or_insert_with(|| ScholarScore {
            scholar_name: signal.scholar_name.clone(),
            scholar_email: signal.scholar_email.clone(),
            cohort: signal.cohort.clone(),
            score: 0.0,
            signal_count: 0,
        });

        entry.score += (signal.severity as f64) * weight;
        entry.signal_count += 1;
    }

    let mut values: Vec<ScholarScore> = scores.into_values().collect();
    values.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    values
}

pub fn recency_weight(days_ago: i64) -> f64 {
    match days_ago {
        0..=7 => 1.0,
        8..=30 => 0.7,
        31..=60 => 0.4,
        _ => 0.2,
    }
}

pub fn cutoff_date(since_days: i64) -> NaiveDate {
    Utc::now().date_naive() - Duration::days(since_days.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_signal(days_ago: i64, severity: i32) -> SignalRecord {
        let occurred_at = Utc::now().date_naive() - Duration::days(days_ago);
        SignalRecord {
            scholar_id: Uuid::new_v4(),
            scholar_name: "Avery Lee".to_string(),
            scholar_email: "avery@example.com".to_string(),
            cohort: "2026".to_string(),
            signal_type: "attendance".to_string(),
            severity,
            occurred_at,
            note: "missed session".to_string(),
        }
    }

    #[test]
    fn weights_follow_expected_tiers() {
        assert_eq!(recency_weight(2), 1.0);
        assert_eq!(recency_weight(15), 0.7);
        assert_eq!(recency_weight(40), 0.4);
        assert_eq!(recency_weight(90), 0.2);
    }

    #[test]
    fn scores_accumulate_by_scholar() {
        let scholar_id = Uuid::new_v4();
        let signals = vec![
            SignalRecord {
                scholar_id,
                scholar_name: "Avery Lee".to_string(),
                scholar_email: "avery@example.com".to_string(),
                cohort: "2026".to_string(),
                signal_type: "attendance".to_string(),
                severity: 3,
                occurred_at: Utc::now().date_naive() - Duration::days(3),
                note: "missed session".to_string(),
            },
            SignalRecord {
                scholar_id,
                scholar_name: "Avery Lee".to_string(),
                scholar_email: "avery@example.com".to_string(),
                cohort: "2026".to_string(),
                signal_type: "engagement".to_string(),
                severity: 2,
                occurred_at: Utc::now().date_naive() - Duration::days(12),
                note: "no response".to_string(),
            },
        ];

        let scores = score_signals(&signals, 30);
        assert_eq!(scores.len(), 1);
        let score = &scores[0];
        let expected = 3.0 * 1.0 + 2.0 * 0.7;
        assert!((score.score - expected).abs() < 0.001);
        assert_eq!(score.signal_count, 2);
    }

    #[test]
    fn cutoff_date_respects_since_days() {
        let cutoff = cutoff_date(14);
        let expected = Utc::now().date_naive() - Duration::days(14);
        assert_eq!(cutoff, expected);
    }

    #[test]
    fn ignores_signals_outside_window() {
        let signals = vec![sample_signal(2, 2), sample_signal(90, 5)];
        let scores = score_signals(&signals, 30);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].signal_count, 1);
    }
}
