# Changelog

## Unreleased

## 0.2.0 — 2026-11-23

### Added

- Custom cleanup rules in `ditch.toml`, or in another file via `--config`. User rules are added to the built-in set and can reuse a built-in kind name.
- Homebrew, Scoop, asdf, and mise packaging under `packaging/`.

## 0.1.0 — 2026-11-20

### Added

- Scan a directory tree and delete regenerable build output for Rust, Node, Python, and other project kinds.
