use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SignalRecord {
    pub scholar_id: Uuid,
    pub scholar_name: String,
    pub scholar_email: String,
    pub cohort: String,
    pub signal_type: String,
    pub severity: i32,
    pub occurred_at: NaiveDate,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct ScholarScore {
    pub scholar_name: String,
    pub scholar_email: String,
    pub cohort: String,
    pub score: f64,
    pub signal_count: usize,
}

#[derive(Debug, Clone)]
pub struct SignalTypeSummary {
    pub signal_type: String,
    pub count: usize,
    pub avg_severity: f64,
}
