# ditch

`ditch` walks a directory tree, finds build output and other regenerable project junk, and deletes it. Source files, lockfiles, and git history stay put. Run it again after a rebuild and you get the disk space back.

It only removes a directory when a project file sits next to it (`Cargo.toml` beside `target/`, `package.json` beside `node_modules/`, and so on). The exception is `__pycache__`, which is removed wherever it appears.

## Install

From the repository:

```sh
cargo install --git https://github.com/baoulo/ditch.git
```

Or from a checkout:

```sh
cargo install --path .
```

This crate uses the 2024 edition, so you need Rust 1.85 or newer.

## Quick start

See what would be deleted under the current directory, and how much space it would free. Nothing is removed.

```sh
ditch -n
```

```text
Scanning...
     1.4 GB  rust     ./server/target
   612.8 MB  node     ./web/node_modules
    94.2 MB  node     ./web/.next
    18.6 MB  python   ./tools/.mypy_cache

     1.4 GB  rust     (1 dirs)
   707.0 MB  node     (2 dirs)
    18.6 MB  python   (1 dirs)
     2.1 GB  total    (4 dirs)

Dry run: nothing was deleted.
```

Drop the `-n` to delete. `ditch` prints the same report and then asks:

```text
Delete these 4 directories? [y/N]
```

`y`, `Y`, or `yes` deletes. Anything else, including Enter, aborts. Pass `-y` to skip the question.

## Usage

```text
ditch [OPTIONS] [PATHS]...
```

With no paths, `ditch` scans the current directory. Each path must be a directory. A file or a missing path is skipped with a warning.

| Flag | What it does |
| --- | --- |
| `-n`, `--dry-run` | Report matches and sizes. Delete nothing. |
| `-y`, `--yes` | Delete without asking. |
| `--only <KINDS>` | Only these project kinds, comma-separated. |
| `--skip <KINDS>` | Ignore these project kinds, comma-separated. |
| `--exclude <NAMES>` | Directory names never to descend into, comma-separated. |
| `--min-size-mb <N>` | Leave directories smaller than `N` megabytes alone. Default `0`. |
| `--list-kinds` | Print the supported kinds and exit. |
| `-h`, `--help` | Print help. |
| `-V`, `--version` | Print the version. |

`--only` and `--skip` accept the kind names below. An unknown name is an error, and nothing is scanned. If you pass both, a kind has to be listed in `--only` and absent from `--skip`.

## Examples

Scan a few trees and only print the large hits:

```sh
ditch -n --min-size-mb 100 ~/src ~/work
```

Clean Rust and Node projects, and leave everything else:

```sh
ditch -n --only rust,node ~/src
```

Clean everything except Node. `node_modules` is not walked, so a project buried inside one is left alone too:

```sh
ditch --skip node ~/src
```

Skip a vendored tree and a data directory by name, at any depth:

```sh
ditch -n --exclude vendor,datasets ~/src
```

Delete without a prompt, for a script or a cron job:

```sh
ditch -y --min-size-mb 50 ~/src
```

List every kind the binary knows about:

```sh
ditch --list-kinds
```

## What gets removed

A directory is a match when its name is in the right-hand column and its parent contains one of the markers.

| Kind | Marker in the parent | Directories removed |
| --- | --- | --- |
| rust | `Cargo.toml` | `target` |
| elixir | `mix.exs` | `_build`, `cover`, `.elixir_ls` |
| node | `package.json` | `node_modules`, `.next`, `.nuxt`, `.svelte-kit`, `.parcel-cache`, `.turbo` |
| python | none | `__pycache__` |
| python | `pyproject.toml`, `setup.py`, `setup.cfg`, `requirements.txt` | `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, `.tox`, `.nox` |
| maven | `pom.xml` | `target` |
| gradle | `build.gradle`, `build.gradle.kts`, `settings.gradle`, `settings.gradle.kts` | `build`, `.gradle` |
| dotnet | `*.csproj`, `*.fsproj`, `*.vbproj` | `bin`, `obj` |
| haskell | `stack.yaml` | `.stack-work` |
| haskell | `*.cabal`, `cabal.project` | `dist-newstyle` |
| zig | `build.zig` | `zig-cache`, `.zig-cache`, `zig-out` |
| swift | `Package.swift` | `.build` |
| dart | `pubspec.yaml` | `.dart_tool`, `build` |
| elm | `elm.json` | `elm-stuff` |
| cmake | `CMakeLists.txt` | `cmake-build-debug`, `cmake-build-release` |

A marker of `*` matches by suffix, so `*.csproj` matches `App.csproj`.

Rust `target/` has one extra check. It is removed only when it contains `CACHEDIR.TAG`, `.rustc_info.json`, `debug`, or `release`. A hand-made directory named `target` next to a `Cargo.toml` is left alone.

`ditch` does not remove global caches (`~/.cargo`, the npm cache, `~/Library/Caches`, and the rest). It does not remove in-source build directories from CMake or Make unless they use one of the names in the table. A generic `build/` is removed only for Gradle and Dart, and only when the matching project file is in the parent.

Most of these directories come back from a normal build. `node_modules` does not: reinstall dependencies after deleting it.

## How a scan works

`ditch` does not follow symlinks, so a symlinked directory is neither entered nor deleted. It never enters `.git`. It never enters a directory named in `--exclude`.

Once a directory matches, it is not scanned further. Nested projects inside `target/` or `node_modules` are not listed on their own. The whole matched directory is one row in the report.

Matches are sized in parallel, then printed largest first, with a per-kind summary. On Unix the size is allocated disk usage, and a hard-linked file is counted once. Cargo hard-links binaries inside `target/`, so the number tracks what you would get back. Elsewhere the size is the sum of file lengths.

`--min-size-mb` is applied before the prompt. Directories under the threshold are not listed and not deleted.

## Exit status

| Code | When |
| --- | --- |
| 0 | Nothing to remove, a dry run, a successful delete, or you answered no at the prompt. |
| 1 | An unknown kind was requested, or at least one directory could not be deleted. |

Directories that did delete are still removed when a later one fails. The `Freed` line counts only the ones that succeeded.
