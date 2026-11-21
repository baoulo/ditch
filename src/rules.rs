use std::fs;
use std::path::Path;

use serde::Deserialize;

/// A rule describing build output for one kind of project.
///
/// A directory named in `dirs` is only considered build output when its parent
/// directory contains at least one of the `markers` (e.g. `Cargo.toml` next to
/// `target/`). Markers starting with `*` match by suffix (e.g. `*.csproj`).
/// An empty `markers` list means the directory is always safe to remove.
///
/// If `sentinels` is non-empty, the candidate directory must also contain at
/// least one of those entries, as an extra guard against false positives.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub kind: String,
    #[serde(default)]
    pub markers: Vec<String>,
    pub dirs: Vec<String>,
    #[serde(default)]
    pub sentinels: Vec<String>,
}

struct Builtin {
    kind: &'static str,
    markers: &'static [&'static str],
    dirs: &'static [&'static str],
    sentinels: &'static [&'static str],
}

const BUILTINS: &[Builtin] = &[
    Builtin {
        kind: "rust",
        markers: &["Cargo.toml"],
        dirs: &["target"],
        sentinels: &["CACHEDIR.TAG", ".rustc_info.json", "debug", "release"],
    },
    Builtin {
        kind: "elixir",
        markers: &["mix.exs"],
        dirs: &["_build", "cover", ".elixir_ls"],
        sentinels: &[],
    },
    Builtin {
        kind: "node",
        markers: &["package.json"],
        dirs: &[
            "node_modules",
            ".next",
            ".nuxt",
            ".svelte-kit",
            ".parcel-cache",
            ".turbo",
        ],
        sentinels: &[],
    },
    Builtin {
        kind: "python",
        markers: &[],
        dirs: &["__pycache__"],
        sentinels: &[],
    },
    Builtin {
        kind: "python",
        markers: &[
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
            "requirements.txt",
        ],
        dirs: &[
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            ".tox",
            ".nox",
        ],
        sentinels: &[],
    },
    Builtin {
        kind: "maven",
        markers: &["pom.xml"],
        dirs: &["target"],
        sentinels: &[],
    },
    Builtin {
        kind: "gradle",
        markers: &[
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
        ],
        dirs: &["build", ".gradle"],
        sentinels: &[],
    },
    Builtin {
        kind: "dotnet",
        markers: &["*.csproj", "*.fsproj", "*.vbproj"],
        dirs: &["bin", "obj"],
        sentinels: &[],
    },
    Builtin {
        kind: "haskell",
        markers: &["stack.yaml"],
        dirs: &[".stack-work"],
        sentinels: &[],
    },
    Builtin {
        kind: "haskell",
        markers: &["*.cabal", "cabal.project"],
        dirs: &["dist-newstyle"],
        sentinels: &[],
    },
    Builtin {
        kind: "zig",
        markers: &["build.zig"],
        dirs: &["zig-cache", ".zig-cache", "zig-out"],
        sentinels: &[],
    },
    Builtin {
        kind: "swift",
        markers: &["Package.swift"],
        dirs: &[".build"],
        sentinels: &[],
    },
    Builtin {
        kind: "dart",
        markers: &["pubspec.yaml"],
        dirs: &[".dart_tool", "build"],
        sentinels: &[],
    },
    Builtin {
        kind: "elm",
        markers: &["elm.json"],
        dirs: &["elm-stuff"],
        sentinels: &[],
    },
    Builtin {
        kind: "cmake",
        markers: &["CMakeLists.txt"],
        dirs: &["cmake-build-debug", "cmake-build-release"],
        sentinels: &[],
    },
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    #[serde(default)]
    rule: Vec<Rule>,
}

impl Rule {
    pub fn matches_dir(&self, name: &str) -> bool {
        self.dirs.iter().any(|dir| dir == name)
    }

    pub fn has_marker(&self, names: &[String]) -> bool {
        if self.markers.is_empty() {
            return true;
        }
        self.markers
            .iter()
            .any(|marker| match marker.strip_prefix('*') {
                Some(suffix) => names.iter().any(|name| name.ends_with(suffix)),
                None => names.iter().any(|name| name == marker),
            })
    }
}

pub fn builtin_rules() -> Vec<Rule> {
    BUILTINS.iter().map(Rule::from).collect()
}

/// Kind names in first-seen order. A repeated kind, including one added by a
/// user rule, appears once.
pub fn kinds_of(rules: &[Rule]) -> Vec<String> {
    let mut kinds = Vec::new();
    for rule in rules {
        if !kinds.iter().any(|kind| kind == &rule.kind) {
            kinds.push(rule.kind.clone());
        }
    }
    kinds
}

pub fn parse_rules(text: &str) -> Result<Vec<Rule>, String> {
    let config: ConfigFile = toml::from_str(text).map_err(|err| err.to_string())?;
    validate(&config.rule)?;
    Ok(config.rule)
}

pub fn load_rules(path: &Path) -> Result<Vec<Rule>, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    parse_rules(&text).map_err(|err| format!("{}: {err}", path.display()))
}

fn validate(rules: &[Rule]) -> Result<(), String> {
    for (index, rule) in rules.iter().enumerate() {
        let n = index + 1;
        if rule.kind.is_empty() {
            return Err(format!("rule {n}: kind must not be empty"));
        }
        if rule.dirs.is_empty() || rule.dirs.iter().any(|dir| dir.is_empty()) {
            return Err(format!("rule {n}: dirs must not be empty"));
        }
    }
    Ok(())
}

fn owned(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}

impl From<&Builtin> for Rule {
    fn from(builtin: &Builtin) -> Self {
        Self {
            kind: builtin.kind.to_string(),
            markers: owned(builtin.markers),
            dirs: owned(builtin.dirs),
            sentinels: owned(builtin.sentinels),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_file() {
        let rules = parse_rules(
            r#"
[[rule]]
kind = "docs"
markers = ["mkdocs.yml"]
dirs = ["site"]

[[rule]]
kind = "python"
markers = ["pyproject.toml"]
dirs = ["htmlcov"]
sentinels = [".gitignore"]
"#,
        )
        .unwrap();

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].kind, "docs");
        assert_eq!(rules[0].dirs, ["site"]);
        assert!(rules[0].sentinels.is_empty());
        assert_eq!(rules[1].sentinels, [".gitignore"]);
    }

    #[test]
    fn rejects_a_missing_required_field() {
        let err = parse_rules("[[rule]]\nkind = \"docs\"\n").unwrap_err();
        assert!(err.contains("dirs"), "{err}");
    }

    #[test]
    fn suffix_marker_matches_by_ending() {
        let rule = Rule {
            kind: "dotnet".into(),
            markers: vec!["*.csproj".into()],
            dirs: vec!["bin".into()],
            sentinels: vec![],
        };
        let names = ["App.csproj".into(), "Program.cs".into()];
        assert!(rule.has_marker(&names));
        assert!(!rule.has_marker(&["Program.cs".into()]));
    }

    #[test]
    fn empty_marker_list_matches_anywhere() {
        let rule = Rule {
            kind: "python".into(),
            markers: vec![],
            dirs: vec!["__pycache__".into()],
            sentinels: vec![],
        };
        assert!(rule.has_marker(&[]));
        assert!(rule.has_marker(&["unrelated.txt".into()]));
    }
}
