use std::{
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Context, bail};
use chrono::{NaiveDate, Utc};

use crate::{
    diagnostics::{Diagnostic, DiagnosticLevel, OutputMode},
    fixtures,
};

const MAX_AGE_DAYS: i64 = 90;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Override wasm-test-versions.yaml path.
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshnessReport {
    pub newest_released_at: Option<NaiveDate>,
    pub age_days: Option<i64>,
}

pub fn run(args: Args, output_mode: OutputMode) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    run_with_writer(args, output_mode, &mut stdout)
}

pub fn run_with_writer(
    args: Args,
    output_mode: OutputMode,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    let config_path = args.config.unwrap_or_else(|| {
        args.repo_root
            .join(".github/config/wasm-test-versions.yaml")
    });
    if !config_path.exists() {
        Diagnostic::new(
            DiagnosticLevel::Warn,
            "PM10000",
            format!(
                "{} is not present yet; wasm runner freshness is skipped",
                config_path.display()
            ),
        )
        .emit(output_mode, writer)?;
        return Ok(());
    }

    let value: serde_json::Value = fixtures::load_yaml(&config_path)
        .with_context(|| format!("failed to load {}", config_path.display()))?;
    let today = Utc::now().date_naive();
    let report = analyze_value(&value, today)?;
    let errors = validate_report(&report);
    if errors.is_empty() {
        Diagnostic::info("PM10001", render_report(&report)).emit(output_mode, writer)?;
        return Ok(());
    }
    for error in &errors {
        Diagnostic::error("PM10002", error).emit(output_mode, writer)?;
    }
    bail!("wasm runner freshness has {} error(s)", errors.len())
}

pub fn analyze_value(
    value: &serde_json::Value,
    today: NaiveDate,
) -> anyhow::Result<FreshnessReport> {
    let dates = released_at_dates(value)?;
    let newest = dates.into_iter().max();
    let age_days = newest.map(|date| today.signed_duration_since(date).num_days());
    Ok(FreshnessReport {
        newest_released_at: newest,
        age_days,
    })
}

pub fn validate_report(report: &FreshnessReport) -> Vec<String> {
    match report.age_days {
        None => vec!["wasm runner config does not contain any released_at date".to_owned()],
        Some(age) if age > MAX_AGE_DAYS => vec![format!(
            "wasm runner version is {age} day(s) old; maximum allowed age is {MAX_AGE_DAYS} days"
        )],
        Some(age) if age < 0 => vec![format!(
            "wasm runner released_at is {age} day(s) in the future"
        )],
        Some(_) => Vec::new(),
    }
}

fn released_at_dates(value: &serde_json::Value) -> anyhow::Result<Vec<NaiveDate>> {
    let mut dates = Vec::new();
    collect_released_at_dates(value, &mut dates)?;
    Ok(dates)
}

fn collect_released_at_dates(
    value: &serde_json::Value,
    dates: &mut Vec<NaiveDate>,
) -> anyhow::Result<()> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                if key == "released_at" {
                    let Some(raw) = value.as_str() else {
                        bail!("released_at must be a string");
                    };
                    dates.push(parse_date(raw)?);
                } else {
                    collect_released_at_dates(value, dates)?;
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_released_at_dates(value, dates)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_date(raw: &str) -> anyhow::Result<NaiveDate> {
    let date = raw.get(0..10).unwrap_or(raw);
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .with_context(|| format!("released_at `{raw}` is not a YYYY-MM-DD date"))
}

fn render_report(report: &FreshnessReport) -> String {
    match (report.newest_released_at, report.age_days) {
        (Some(date), Some(age)) => {
            format!("wasm runner newest released_at {date} is {age} day(s) old")
        }
        _ => "wasm runner freshness has no release date".to_owned(),
    }
}
