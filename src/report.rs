use std::fmt::Write;

use chrono::NaiveDate;

use crate::models::{SignalRecord, SignalTrend, SignalTypeSummary};
use crate::risk;

pub fn summarize_by_type(signals: &[SignalRecord]) -> Vec<SignalTypeSummary> {
    let mut map: std::collections::HashMap<String, (usize, i32)> =
        std::collections::HashMap::new();

    for signal in signals {
        let entry = map.entry(signal.signal_type.clone()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += signal.severity;
    }

    let mut summaries: Vec<SignalTypeSummary> = map
        .into_iter()
        .map(|(signal_type, (count, total_severity))| SignalTypeSummary {
            signal_type,
            count,
            avg_severity: if count == 0 {
                0.0
            } else {
                total_severity as f64 / count as f64
            },
        })
        .collect();

    summaries.sort_by(|a, b| b.count.cmp(&a.count));
    summaries
}

pub fn build_report(
    cohort: Option<&str>,
    since_days: i64,
    cutoff: NaiveDate,
    signals: &[SignalRecord],
    trends: &[SignalTrend],
) -> String {
    let scores = risk::score_signals(signals, since_days);
    let summaries = summarize_by_type(signals);

    let mut output = String::new();
    let cohort_label = cohort.unwrap_or("all cohorts");

    let _ = writeln!(output, "# Cohort Early Warning Report");
    let _ = writeln!(
        output,
        "Generated for {} (signals since {})",
        cohort_label, cutoff
    );
    let _ = writeln!(output);
    let _ = writeln!(output, "## Signal Mix");

    if summaries.is_empty() {
        let _ = writeln!(output, "No signals recorded for this window.");
    } else {
        for summary in summaries.iter() {
            let _ = writeln!(
                output,
                "- {}: {} signals (avg severity {:.1})",
                summary.signal_type, summary.count, summary.avg_severity
            );
        }
    }

    let _ = writeln!(output);
    let _ = writeln!(output, "## Highest Risk Scholars");

    if scores.is_empty() {
        let _ = writeln!(output, "No scholars with signals in this window.");
    } else {
        for score in scores.iter().take(10) {
            let _ = writeln!(
                output,
                "- {} ({}, {}) score {:.2} across {} signals",
                score.scholar_name,
                score.scholar_email,
                score.cohort,
                score.score,
                score.signal_count
            );
        }
    }

    let mut recent_signals = signals.to_vec();
    recent_signals.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
    let _ = writeln!(output);
    let _ = writeln!(output, "## Recent Signal Notes");

    if recent_signals.is_empty() {
        let _ = writeln!(output, "No signals recorded for this window.");
    } else {
        for signal in recent_signals.iter().take(5) {
            let _ = writeln!(
                output,
                "- {} ({}) on {}: {}",
                signal.scholar_name, signal.signal_type, signal.occurred_at, signal.note
            );
        }
    }

    let _ = writeln!(output);
    let _ = writeln!(output, "## Weekly Signal Trend");

    if trends.is_empty() {
        let _ = writeln!(output, "No weekly trend data available for this window.");
    } else {
        for trend in trends {
            let _ = writeln!(
                output,
                "- Week of {}: {} signals across {} scholars (avg severity {:.2})",
                trend.week_start, trend.signal_count, trend.scholar_count, trend.avg_severity
            );
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn sample_signal(days_ago: i64, severity: i32) -> SignalRecord {
        let occurred_at = chrono::Utc::now().date_naive() - chrono::Duration::days(days_ago);
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
    fn summarizes_signal_types() {
        let signals = vec![sample_signal(2, 3), sample_signal(1, 1)];
        let summaries = summarize_by_type(&signals);
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].count, 2);
        assert!((summaries[0].avg_severity - 2.0).abs() < 0.01);
    }

    #[test]
    fn report_includes_weekly_trend_section() {
        let signals = vec![sample_signal(2, 3)];
        let trends = vec![SignalTrend {
            week_start: NaiveDate::from_ymd_opt(2026, 2, 2).unwrap(),
            signal_count: 2,
            avg_severity: 2.5,
            scholar_count: 1,
        }];
        let report = build_report(
            Some("2026"),
            30,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            &signals,
            &trends,
        );
        assert!(report.contains("## Weekly Signal Trend"));
        assert!(report.contains("Week of 2026-02-02"));
    }
}
