/// A rule describing build output for one kind of project.
///
/// A directory named in `dirs` is only considered build output when its parent
/// directory contains at least one of the `markers` (e.g. `Cargo.toml` next to
/// `target/`). Markers starting with `*` match by suffix (e.g. `*.csproj`).
/// An empty `markers` list means the directory is always safe to remove.
///
/// If `sentinels` is non-empty, the candidate directory must also contain at
/// least one of those entries, as an extra guard against false positives.
pub struct Rule {
    pub kind: &'static str,
    pub markers: &'static [&'static str],
    pub dirs: &'static [&'static str],
    pub sentinels: &'static [&'static str],
}

pub const RULES: &[Rule] = &[
    Rule {
        kind: "rust",
        markers: &["Cargo.toml"],
        dirs: &["target"],
        sentinels: &["CACHEDIR.TAG", ".rustc_info.json", "debug", "release"],
    },
    Rule {
        kind: "elixir",
        markers: &["mix.exs"],
        dirs: &["_build", "cover", ".elixir_ls"],
        sentinels: &[],
    },
    Rule {
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
    Rule {
        kind: "python",
        markers: &[],
        dirs: &["__pycache__"],
        sentinels: &[],
    },
    Rule {
        kind: "python",
        markers: &["pyproject.toml", "setup.py", "setup.cfg", "requirements.txt"],
        dirs: &[".pytest_cache", ".mypy_cache", ".ruff_cache", ".tox", ".nox"],
        sentinels: &[],
    },
    Rule {
        kind: "maven",
        markers: &["pom.xml"],
        dirs: &["target"],
        sentinels: &[],
    },
    Rule {
        kind: "gradle",
        markers: &["build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts"],
        dirs: &["build", ".gradle"],
        sentinels: &[],
    },
    Rule {
        kind: "dotnet",
        markers: &["*.csproj", "*.fsproj", "*.vbproj"],
        dirs: &["bin", "obj"],
        sentinels: &[],
    },
    Rule {
        kind: "haskell",
        markers: &["stack.yaml"],
        dirs: &[".stack-work"],
        sentinels: &[],
    },
    Rule {
        kind: "haskell",
        markers: &["*.cabal", "cabal.project"],
        dirs: &["dist-newstyle"],
        sentinels: &[],
    },
    Rule {
        kind: "zig",
        markers: &["build.zig"],
        dirs: &["zig-cache", ".zig-cache", "zig-out"],
        sentinels: &[],
    },
    Rule {
        kind: "swift",
        markers: &["Package.swift"],
        dirs: &[".build"],
        sentinels: &[],
    },
    Rule {
        kind: "dart",
        markers: &["pubspec.yaml"],
        dirs: &[".dart_tool", "build"],
        sentinels: &[],
    },
    Rule {
        kind: "elm",
        markers: &["elm.json"],
        dirs: &["elm-stuff"],
        sentinels: &[],
    },
    Rule {
        kind: "cmake",
        markers: &["CMakeLists.txt"],
        dirs: &["cmake-build-debug", "cmake-build-release"],
        sentinels: &[],
    },
];

impl Rule {
    pub fn has_marker(&self, names: &[String]) -> bool {
        if self.markers.is_empty() {
            return true;
        }
        self.markers.iter().any(|m| match m.strip_prefix('*') {
            Some(suffix) => names.iter().any(|n| n.ends_with(suffix)),
            None => names.iter().any(|n| n == m),
        })
    }
}

pub fn all_kinds() -> Vec<&'static str> {
    let mut kinds: Vec<_> = RULES.iter().map(|r| r.kind).collect();
    kinds.dedup();
    kinds
}
