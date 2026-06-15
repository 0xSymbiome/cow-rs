use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::policy::{fixtures, workspace};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Coverage mode.
    #[arg(long, value_enum, default_value = "informational")]
    pub mode: Mode,
    /// Override principle/ADR map path.
    #[arg(long)]
    pub map: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum Mode {
    Informational,
    Blocking,
}

#[derive(Debug, Deserialize)]
pub struct PrincipleAdrMap {
    pub version: u32,
    pub principles: Vec<PrincipleAdrEntry>,
    #[serde(default)]
    pub out_of_scope_adrs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PrincipleAdrEntry {
    pub id: u32,
    pub name: String,
    pub primary_adr: String,
    #[serde(default)]
    pub supporting_adrs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrStatus {
    pub id: String,
    pub path: String,
    pub status: String,
}

pub fn run_default() -> anyhow::Result<()> {
    run(Args {
        repo_root: PathBuf::from("."),
        // Blocking to match the CI invocation of this gate.
        mode: Mode::Blocking,
        map: None,
    })
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let map_path = args
        .map
        .unwrap_or_else(|| args.repo_root.join(".github/config/principle-adr-map.yaml"));
    let map: PrincipleAdrMap = fixtures::load_yaml(&map_path)
        .with_context(|| format!("failed to load {}", map_path.display()))?;
    let statuses = read_adr_statuses(&args.repo_root)?;
    let errors = validate_coverage(&map, &statuses);

    if errors.is_empty() {
        println!("ADR coverage map covers {} accepted ADR(s)", statuses.len());
        return Ok(());
    }

    let label = match args.mode {
        Mode::Informational => "warning",
        Mode::Blocking => "error",
    };
    for error in &errors {
        eprintln!("{label}: {error}");
    }
    if args.mode == Mode::Blocking {
        bail!("ADR coverage has {} error(s)", errors.len());
    }
    Ok(())
}

pub fn validate_coverage(map: &PrincipleAdrMap, statuses: &[AdrStatus]) -> Vec<String> {
    let mut errors = Vec::new();
    if map.version != 1 {
        errors.push(format!(
            "principle-adr-map.yaml version must be 1, got {}",
            map.version
        ));
    }

    let by_id = statuses
        .iter()
        .map(|status| (status.id.clone(), status))
        .collect::<BTreeMap<_, _>>();
    let mut cited = BTreeSet::new();
    let out_of_scope = map
        .out_of_scope_adrs
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    for principle in &map.principles {
        if principle.name.trim().is_empty() {
            errors.push(format!("principle {} has an empty name", principle.id));
        }
        if principle.primary_adr.trim().is_empty() {
            errors.push(format!(
                "principle {} `{}` has no primary ADR",
                principle.id, principle.name
            ));
        } else {
            check_cited_adr(
                &mut errors,
                &by_id,
                &mut cited,
                &principle.primary_adr,
                &format!("principle {} primary ADR", principle.id),
            );
        }
        for adr in &principle.supporting_adrs {
            check_cited_adr(
                &mut errors,
                &by_id,
                &mut cited,
                adr,
                &format!("principle {} supporting ADR", principle.id),
            );
        }
    }

    for status in statuses {
        if is_accepted_status(&status.status)
            && !cited.contains(&status.id)
            && !out_of_scope.contains(&status.id)
        {
            errors.push(format!(
                "accepted ADR {} ({}) is not mapped to any principle",
                status.id, status.path
            ));
        }
    }

    errors
}

fn check_cited_adr(
    errors: &mut Vec<String>,
    by_id: &BTreeMap<String, &AdrStatus>,
    cited: &mut BTreeSet<String>,
    adr: &str,
    context: &str,
) {
    cited.insert(adr.to_owned());
    let Some(status) = by_id.get(adr) else {
        errors.push(format!("{context} references missing ADR {adr}"));
        return;
    };
    if !is_accepted_status(&status.status) {
        errors.push(format!(
            "{context} references ADR {} with status `{}` instead of `Accepted`",
            status.id, status.status
        ));
    }
}

fn is_accepted_status(status: &str) -> bool {
    matches!(status.trim(), "Accepted" | "Accepted (amended)")
}

fn read_adr_statuses(repo_root: &Path) -> anyhow::Result<Vec<AdrStatus>> {
    let adr_root = repo_root.join("docs/adr");
    let mut statuses = Vec::new();
    for entry in
        fs::read_dir(&adr_root).with_context(|| format!("failed to read {}", adr_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(id) = name
            .get(0..4)
            .filter(|id| id.chars().all(|ch| ch.is_ascii_digit()))
        else {
            continue;
        };
        let text = workspace::read_to_string(&path)?;
        let status = parse_status(&text).unwrap_or_else(|| "Missing".to_owned());
        statuses.push(AdrStatus {
            id: id.to_owned(),
            path: workspace::relative_path(repo_root, &path),
            status,
        });
    }
    statuses.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(statuses)
}

fn parse_status(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let trimmed = line.trim().trim_start_matches("- ").trim();
        trimmed
            .strip_prefix("Status:")
            .map(str::trim)
            .map(str::to_owned)
    })
}
