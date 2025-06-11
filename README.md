# Group Scholar Cohort Early Warning

A Rust CLI that tracks early warning signals for scholar cohorts, scores risk, and generates weekly summaries for ops and program teams.

## Features
- Stores scholar profiles and risk signals in Postgres
- Imports signals from CSV with idempotent source keys
- Scores scholars based on severity and recency
- Generates markdown reports with signal mix and top risk list

## Tech Stack
- Rust
- SQLx + Postgres
- Clap

## Setup

This CLI expects a production Postgres database. Do not point it at a local dev database.

```bash
export DATABASE_URL="postgres://USER:PASSWORD@HOST:PORT/DB"
```

### Initialize the schema

```bash
cargo run -- init-db
```

### Seed data

```bash
cargo run -- seed
```

### Import signals

```bash
cargo run -- import --csv examples/sample-signals.csv
```

### Score risk

```bash
cargo run -- score --cohort 2026 --since-days 30
```

### Generate a report

```bash
cargo run -- report --cohort 2026 --since-days 30 --out report.md
```

## CSV Format

Headers:

```
full_name,email,cohort,signal_type,severity,note,occurred_at,source_key
```

- `occurred_at` should be `YYYY-MM-DD`
- `source_key` is optional; if omitted, one is generated

## Tests

```bash
cargo test
```
