mod rules;

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use rayon::prelude::*;
use walkdir::WalkDir;

use rules::Rule;

/// Recursively find development projects and delete their build output.
#[derive(Parser)]
#[command(version, after_help = "Supported kinds: run with --list-kinds")]
struct Cli {
    /// Directories to scan (defaults to the current directory)
    paths: Vec<PathBuf>,

    /// Only report what would be deleted and how much space it would free
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Delete without asking for confirmation
    #[arg(short, long)]
    yes: bool,

    /// Only clean these project kinds (comma separated, e.g. rust,elixir)
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,

    /// Skip these project kinds (comma separated, e.g. node)
    #[arg(long, value_delimiter = ',')]
    skip: Vec<String>,

    /// Directory names to never descend into (comma separated)
    #[arg(long, value_delimiter = ',')]
    exclude: Vec<String>,

    /// Hide entries smaller than this many megabytes
    #[arg(long, default_value_t = 0.0)]
    min_size_mb: f64,

    /// List the supported project kinds and what they clean, then exit
    #[arg(long)]
    list_kinds: bool,

    /// Load extra rules from this file instead of ./ditch.toml
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

struct Target {
    path: PathBuf,
    kind: String,
    size: u64,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let user_rules = match user_rules(cli.config.as_deref()) {
        Ok(rules) => rules,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };
    let mut rules = rules::builtin_rules();
    rules.extend(user_rules);

    if cli.list_kinds {
        for rule in &rules {
            let markers = if rule.markers.is_empty() {
                "(anywhere)".to_string()
            } else {
                rule.markers.join(", ")
            };
            println!(
                "{:<8} {:<45} -> {}",
                rule.kind,
                markers,
                rule.dirs.join(", ")
            );
        }
        return ExitCode::SUCCESS;
    }

    let known = rules::kinds_of(&rules);
    for k in cli.only.iter().chain(&cli.skip) {
        if !known.iter().any(|kind| kind == k) {
            eprintln!("error: unknown kind '{k}' (known: {})", known.join(", "));
            return ExitCode::FAILURE;
        }
    }
    let rules: Vec<&Rule> = rules
        .iter()
        .filter(|r| cli.only.is_empty() || cli.only.iter().any(|k| k == &r.kind))
        .filter(|r| !cli.skip.iter().any(|k| k == &r.kind))
        .collect();

    let roots = if cli.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cli.paths.clone()
    };

    eprintln!("Scanning...");
    let mut found = Vec::new();
    for root in &roots {
        if !root.is_dir() {
            eprintln!("warning: {} is not a directory, skipping", root.display());
            continue;
        }
        scan(root, &rules, &cli.exclude, &mut found);
    }

    let min_size = (cli.min_size_mb * 1024.0 * 1024.0) as u64;
    let mut targets: Vec<Target> = found
        .into_par_iter()
        .map(|(path, kind)| Target {
            size: dir_size(&path),
            path,
            kind,
        })
        .filter(|t| t.size >= min_size)
        .collect();
    targets.sort_by_key(|t| std::cmp::Reverse(t.size));

    if targets.is_empty() {
        println!("Nothing to clean.");
        return ExitCode::SUCCESS;
    }

    for t in &targets {
        println!("{:>10}  {:<8} {}", human(t.size), t.kind, t.path.display());
    }
    print_summary(&targets);

    if cli.dry_run {
        println!("\nDry run: nothing was deleted.");
        return ExitCode::SUCCESS;
    }

    if !cli.yes && !confirm(targets.len()) {
        println!("Aborted.");
        return ExitCode::SUCCESS;
    }

    let results: Vec<(&Target, io::Result<()>)> = targets
        .par_iter()
        .map(|t| (t, fs::remove_dir_all(&t.path)))
        .collect();

    let mut freed = 0;
    let mut failures = 0;
    for (t, res) in results {
        match res {
            Ok(()) => freed += t.size,
            Err(e) => {
                failures += 1;
                eprintln!("failed to delete {}: {e}", t.path.display());
            }
        }
    }
    println!("\nFreed {}.", human(freed));
    if failures > 0 {
        eprintln!(
            "{failures} director{} could not be deleted.",
            if failures == 1 { "y" } else { "ies" }
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn user_rules(config: Option<&Path>) -> Result<Vec<Rule>, String> {
    let path = match config {
        Some(path) => {
            if !path.is_file() {
                return Err(format!("config file not found: {}", path.display()));
            }
            path.to_path_buf()
        }
        None => {
            let default = PathBuf::from("ditch.toml");
            if !default.is_file() {
                return Ok(Vec::new());
            }
            default
        }
    };
    rules::load_rules(&path)
}

/// Walks `dir` looking for build output directories. Does not follow symlinks
/// and does not descend into directories it has marked for deletion.
fn scan(dir: &Path, rules: &[&Rule], exclude: &[String], out: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let entries: Vec<_> = entries.flatten().collect();
    let names: Vec<String> = entries
        .iter()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    for (entry, name) in entries.iter().zip(&names) {
        // file_type() does not follow symlinks, so symlinked dirs are skipped.
        let Ok(ft) = entry.file_type() else { continue };
        if !ft.is_dir() || name == ".git" || exclude.contains(name) {
            continue;
        }
        let path = entry.path();

        let matched = rules
            .iter()
            .find(|r| r.matches_dir(name) && r.has_marker(&names) && has_sentinel(r, &path));
        match matched {
            Some(rule) => out.push((path, rule.kind.clone())),
            // node_modules can be enormous; never walk into one we're not deleting.
            None if name == "node_modules" => {}
            None => scan(&path, rules, exclude, out),
        }
    }
}

fn has_sentinel(rule: &Rule, dir: &Path) -> bool {
    rule.sentinels.is_empty() || rule.sentinels.iter().any(|s| dir.join(s).exists())
}

/// Disk usage of a directory tree, counting hard-linked files once
/// (cargo hard-links binaries between `target/*/deps` and `target/*`).
fn dir_size(dir: &Path) -> u64 {
    let mut seen = HashSet::new();
    WalkDir::new(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| !m.is_dir())
        .map(|m| disk_usage(&m, &mut seen))
        .sum()
}

#[cfg(unix)]
fn disk_usage(m: &fs::Metadata, seen: &mut HashSet<(u64, u64)>) -> u64 {
    use std::os::unix::fs::MetadataExt;
    if m.nlink() > 1 && !seen.insert((m.dev(), m.ino())) {
        return 0;
    }
    m.blocks() * 512
}

#[cfg(not(unix))]
fn disk_usage(m: &fs::Metadata, _seen: &mut HashSet<(u64, u64)>) -> u64 {
    m.len()
}

fn print_summary(targets: &[Target]) {
    let mut by_kind: BTreeMap<&str, (usize, u64)> = BTreeMap::new();
    for t in targets {
        let e = by_kind.entry(t.kind.as_str()).or_default();
        e.0 += 1;
        e.1 += t.size;
    }
    println!();
    for (kind, (count, size)) in &by_kind {
        println!("{:>10}  {:<8} ({count} dirs)", human(*size), kind);
    }
    let total: u64 = targets.iter().map(|t| t.size).sum();
    println!("{:>10}  total    ({} dirs)", human(total), targets.len());
}

fn confirm(count: usize) -> bool {
    print!("\nDelete these {count} directories? [y/N] ");
    let _ = io::stdout().flush();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).is_ok() && matches!(answer.trim(), "y" | "Y" | "yes")
}

fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}
