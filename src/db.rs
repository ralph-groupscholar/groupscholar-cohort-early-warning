use anyhow::Context;
use chrono::NaiveDate;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::SignalRecord;

pub async fn init_db(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn seed(pool: &PgPool) -> anyhow::Result<()> {
    let scholars = vec![
        (
            Uuid::parse_str("3d7f5d6f-24f7-4e8e-8b4b-3e7e44b4a7b2")?,
            "Avery Lee",
            "avery.lee@groupscholar.com",
            "2026",
        ),
        (
            Uuid::parse_str("0c22f1f1-9184-4fd4-9b21-28c68a6a89dc")?,
            "Jules Moreno",
            "jules.moreno@groupscholar.com",
            "2025",
        ),
        (
            Uuid::parse_str("d5a0a1a2-2a3c-44c2-8f73-60b7897a9dd2")?,
            "Kiara Patel",
            "kiara.patel@groupscholar.com",
            "2026",
        ),
    ];

    for (id, name, email, cohort) in scholars {
        sqlx::query(
            r#"
            INSERT INTO cohort_early_warning.scholars (id, full_name, email, cohort)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (email) DO UPDATE
            SET full_name = EXCLUDED.full_name, cohort = EXCLUDED.cohort
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(email)
        .bind(cohort)
        .fetch_one(pool)
        .await?;
    }

    let signals = vec![
        (
            "seed-001",
            "avery.lee@groupscholar.com",
            "attendance",
            3,
            "Missed last two sessions",
            NaiveDate::from_ymd_opt(2026, 2, 2).context("invalid date")?,
        ),
        (
            "seed-002",
            "jules.moreno@groupscholar.com",
            "engagement",
            2,
            "Slow response to outreach",
            NaiveDate::from_ymd_opt(2026, 1, 30).context("invalid date")?,
        ),
        (
            "seed-003",
            "kiara.patel@groupscholar.com",
            "academic",
            4,
            "Reported GPA dip",
            NaiveDate::from_ymd_opt(2026, 1, 28).context("invalid date")?,
        ),
    ];

    for (source_key, email, signal_type, severity, note, occurred_at) in signals {
        let scholar_id: Uuid = sqlx::query(
            "SELECT id FROM cohort_early_warning.scholars WHERE email = $1",
        )
        .bind(email)
        .fetch_one(pool)
        .await?
        .get("id");

        sqlx::query(
            r#"
            INSERT INTO cohort_early_warning.signals
            (id, scholar_id, signal_type, severity, note, occurred_at, source_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (source_key) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(scholar_id)
        .bind(signal_type)
        .bind(severity)
        .bind(note)
        .bind(occurred_at)
        .bind(source_key)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn fetch_signals(
    pool: &PgPool,
    since_date: NaiveDate,
    cohort: Option<&str>,
    email: Option<&str>,
) -> anyhow::Result<Vec<SignalRecord>> {
    let mut query = String::from(
        "SELECT sc.id as scholar_id, sc.full_name, sc.email, sc.cohort, \
         s.signal_type, s.severity, s.note, s.occurred_at\
         FROM cohort_early_warning.signals s\
         JOIN cohort_early_warning.scholars sc ON sc.id = s.scholar_id\
         WHERE s.occurred_at >= $1",
    );

    if cohort.is_some() {
        query.push_str(" AND sc.cohort = $2");
    } else if email.is_some() {
        query.push_str(" AND sc.email = $2");
    }

    let mut rows = sqlx::query(&query).bind(since_date);

    if let Some(value) = cohort {
        rows = rows.bind(value);
    } else if let Some(value) = email {
        rows = rows.bind(value);
    }

    let records = rows.fetch_all(pool).await?;
    let mut signals = Vec::new();

    for row in records {
        signals.push(SignalRecord {
            scholar_id: row.get("scholar_id"),
            scholar_name: row.get("full_name"),
            scholar_email: row.get("email"),
            cohort: row.get("cohort"),
            signal_type: row.get("signal_type"),
            severity: row.get("severity"),
            occurred_at: row.get("occurred_at"),
            note: row.get("note"),
        });
    }

    Ok(signals)
}

pub async fn import_csv(pool: &PgPool, csv_path: &std::path::Path) -> anyhow::Result<usize> {
    #[derive(serde::Deserialize)]
    struct CsvRow {
        full_name: String,
        email: String,
        cohort: String,
        signal_type: String,
        severity: i32,
        note: String,
        occurred_at: NaiveDate,
        source_key: Option<String>,
    }

    let mut reader = csv::Reader::from_path(csv_path)?;
    let mut inserted = 0usize;

    for result in reader.deserialize::<CsvRow>() {
        let row = result?;
        let scholar_id: Uuid = sqlx::query(
            r#"
            INSERT INTO cohort_early_warning.scholars
            (id, full_name, email, cohort)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (email) DO UPDATE
            SET full_name = EXCLUDED.full_name, cohort = EXCLUDED.cohort
            RETURNING id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&row.full_name)
        .bind(&row.email)
        .bind(&row.cohort)
        .fetch_one(pool)
        .await?
        .get("id");

        let source_key = row
            .source_key
            .unwrap_or_else(|| format!("import-{}", Uuid::new_v4()));

        let result = sqlx::query(
            r#"
            INSERT INTO cohort_early_warning.signals
            (id, scholar_id, signal_type, severity, note, occurred_at, source_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (source_key) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(scholar_id)
        .bind(&row.signal_type)
        .bind(row.severity)
        .bind(&row.note)
        .bind(row.occurred_at)
        .bind(source_key)
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            inserted += 1;
        }
    }

    Ok(inserted)
}
