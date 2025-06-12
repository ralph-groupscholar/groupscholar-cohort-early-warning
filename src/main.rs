use std::path::PathBuf;

use anyhow::Context;
use clap::{ArgGroup, Parser, Subcommand};
use sqlx::postgres::PgPoolOptions;

mod db;
mod models;
mod report;
mod risk;

#[derive(Parser)]
#[command(name = "cohort-early-warning")]
#[command(about = "Cohort early warning signal tracker for Group Scholar", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create or upgrade the database schema
    InitDb,
    /// Load realistic seed data
    Seed,
    /// Import signals from a CSV file
    Import {
        #[arg(long)]
        csv: PathBuf,
    },
    /// Score risk across scholars
    #[command(group(
        ArgGroup::new("scope")
            .args(["cohort", "email"])
            .multiple(false)
    ))]
    Score {
        #[arg(long)]
        cohort: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long, default_value_t = 30)]
        since_days: i64,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Generate a markdown report
    #[command(group(
        ArgGroup::new("scope")
            .args(["cohort", "email"])
            .multiple(false)
    ))]
    Report {
        #[arg(long)]
        cohort: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long, default_value_t = 30)]
        since_days: i64,
        #[arg(long, default_value = "report.md")]
        out: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL must be set to a production Postgres instance")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("failed to connect to Postgres")?;

    match cli.command {
        Commands::InitDb => {
            db::init_db(&pool).await?;
            println!("Schema ready.");
        }
        Commands::Seed => {
            db::seed(&pool).await?;
            println!("Seed data inserted.");
        }
        Commands::Import { csv } => {
            let inserted = db::import_csv(&pool, &csv).await?;
            println!("Inserted {inserted} signals from {}.", csv.display());
        }
        Commands::Score {
            cohort,
            email,
            since_days,
            limit,
        } => {
            let since_date = risk::cutoff_date(since_days);
            let signals = db::fetch_signals(
                &pool,
                since_date,
                cohort.as_deref(),
                email.as_deref(),
            )
            .await?;
            let scores = risk::score_signals(&signals, since_days);

            if scores.is_empty() {
                println!("No signals found for this window.");
                return Ok(());
            }

            println!("Top scholars by risk score:");
            for score in scores.iter().take(limit) {
                println!(
                    "- {} ({}, {}) score {:.2} across {} signals",
                    score.scholar_name,
                    score.scholar_email,
                    score.cohort,
                    score.score,
                    score.signal_count
                );
            }
        }
        Commands::Report {
            cohort,
            email,
            since_days,
            out,
        } => {
            let since_date = risk::cutoff_date(since_days);
            let signals = db::fetch_signals(
                &pool,
                since_date,
                cohort.as_deref(),
                email.as_deref(),
            )
            .await?;
            let trends = db::fetch_weekly_trends(
                &pool,
                since_date,
                cohort.as_deref(),
                email.as_deref(),
            )
            .await?;
            let report = report::build_report(
                cohort.as_deref().or(email.as_deref()),
                since_days,
                since_date,
                &signals,
                &trends,
            );
            std::fs::write(&out, report)?;
            println!("Report written to {}.", out.display());
        }
    }

    Ok(())
}
