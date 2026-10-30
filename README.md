# reposcope

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#development)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

A high-performance, concurrent terminal dashboard for tracking status, sync state, and staleness across your local Git repositories.

---

## Overview

When actively working across multiple repositories, tracking uncommitted changes, divergent branches, and stale projects can quickly become chaotic. Shelling out commands to individual repositories is slow and tedious.

**reposcope** provides a unified bird's-eye view of your entire local workspace in seconds. Built with Rust, Ratatui, and direct `libgit2` bindings, it scans repositories in parallel and presents your workspace state without remote network calls or background daemons.

```text
reposcope  12 repos  3 dirty  2 stale  0 errors  [sort: attention (s)]  [details: on (d)]

> reposcope         main             synced    clean              7eb36b7 feat(tui): add... (2m ago)   clean
  typed-up          feature/parser   ↑1        1 modified         9a1f4b2 add ast parser (4h ago)       dirty
  scratchorm        main             synced    clean              b82c103 fix pooling bug (12d ago)     stale
  terminal.chat     develop          ↓3        2 untracked        e4a901f bump deps (1d ago)            dirty
```

---

## Key Features

- **Concurrent Multi-Repo Scans**: Scans multiple repositories in parallel using native scoped threads (`std::thread::scope`) and direct `libgit2` bindings—no subprocess overhead.
- **Divergence & Sync Tracking**: Instantly view whether your current branch is ahead, behind, diverged, or synced relative to its upstream tracking reference.
- **Uncommitted Change Monitoring**: Distinct counts for staged changes, unstaged modifications, and untracked files.
- **Smart Staleness Detection**: Flags inactive repositories based on configurable commit age thresholds, while smartly ignoring repos with active work in progress.
- **Interactive TUI Dashboard**:
  - **Live Filter (`/`)**: Instant, non-blocking case-insensitive search by project name, branch, commit message, or error status.
  - **Multi-Mode Sorting (`s`)**: Cycle sort order by attention required, repository name, recency, or dirty changes.
  - **Details Inspector (`d`)**: Toggle an expanded inspection panel displaying full file paths, full commit hashes, author timestamps, and sync metadata.
  - **Vim & Standard Navigation**: Full support for `hjkl`, arrow keys, `g`/`G`, and `Page Up`/`Page Down`.
- **Fault-Tolerant & Resilient**: Broken, non-existent, or unborn git repositories degrade gracefully into visible error rows rather than aborting the scan.
- **Fast Directory Jumping**: Press `Enter` on any selected repository to exit and immediately `cd` into that directory using a shell wrapper.
- **Zero Network Overhead**: All calculations rely purely on local state and tracking refs—zero latency, zero network traffic, and fully offline-capable.

---

## Installation

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (1.74+ recommended, Rust 2021 edition)
- C compiler and development headers for `libgit2` (automatically handled by `git2-rs` on most platforms)

### Build and Install from Source

```bash
# Clone the repository
git clone https://github.com/your-username/reposcope.git
cd reposcope

# Option A: Install directly via Cargo to ~/.cargo/bin
cargo install --path .

# Option B: Build optimized release binary manually
cargo build --release
install -Dm755 target/release/reposcope ~/.local/bin/reposcope
```

Ensure your installation directory (`~/.cargo/bin` or `~/.local/bin`) is included in your system's `PATH`.

---

## Usage

### Managing Tracked Repositories

Use the CLI subcommands to manage the list of projects monitored by reposcope:

```bash
# Track a new repository (accepts absolute, relative, or tilde paths)
reposcope add ~/dev/my-project
reposcope add ./another-project

# List registered projects in a script-friendly format
reposcope list

# Remove a repository by name or path
reposcope remove my-project
reposcope remove ~/dev/my-project
```

### Launching the Dashboard

Run `reposcope` without subcommands to launch the interactive dashboard:

```bash
reposcope
```

---

## Keybindings Reference

| Key | Action | Description |
| :--- | :--- | :--- |
| `↓` / `j` | **Select Next** | Move cursor to the next repository |
| `↑` / `k` | **Select Previous** | Move cursor to the previous repository |
| `g` / `Home` | **Jump to Top** | Move cursor to the first repository |
| `G` / `End` | **Jump to Bottom** | Move cursor to the last repository |
| `PgUp` / `PgDn` | **Page Jump** | Scroll through repositories 10 rows at a time |
| `/` | **Filter / Search** | Start live text search across names, branches, and commits |
| `Esc` | **Clear / Back** | Clear active filter query, exit search mode, or quit |
| `s` | **Cycle Sort** | Toggle sorting: `attention` &rarr; `name` &rarr; `recent` &rarr; `dirty` |
| `d` | **Toggle Details** | Show/hide the inspector pane with commit and path info |
| `r` | **Rescan** | Re-run concurrent scans across all repositories in place |
| `Enter` | **Select & Jump** | Exit dashboard and print selected repository path |
| `q` | **Quit** | Exit reposcope cleanly, restoring terminal state |

---

## Shell Integration (`cd` on Selection)

Because child processes cannot modify the working directory of the invoking parent shell, `reposcope` supports a `--print-cd-target` flag. When you highlight a repository and press `Enter`, `reposcope` prints its path to stdout upon clean terminal exit.

Add the appropriate wrapper function to your shell configuration file:

### Bash / Zsh (`~/.bashrc` or `~/.zshrc`)

```bash
reposcope() {
  local target
  target=$(command reposcope "$@" --print-cd-target)
  [ -n "$target" ] && cd "$target"
}
```

### Fish (`~/.config/fish/functions/reposcope.fish`)

```fish
function reposcope
  set -l target (command reposcope $argv --print-cd-target)
  if test -n "$target"
    cd "$target"
  end
end
```

---

## Configuration

`reposcope` stores configuration and registered project lists in accordance with the XDG Base Directory specification at:

```text
~/.config/reposcope/config.toml
```

### Example `config.toml`

```toml
# Inactivity threshold (in days) before clean repositories are marked as stale
stale_threshold_days = 7

[[projects]]
name = "scratchorm"
path = "/home/developer/dev/ScratchORM"

[[projects]]
name = "typed-up"
path = "/home/developer/dev/typed-up"
stale_threshold_days = 14  # Optional per-project threshold override
```

- **`stale_threshold_days`**: Inactivity period after which a clean project is flagged `stale`. Repositories with uncommitted changes are never flagged as stale regardless of age.
- **`projects`**: Array of tracked projects, each with a display `name`, filesystem `path`, and optional override settings.

---

## Architecture & Design Principles

- **Zero-Subprocess Git Access**: Uses Rust bindings to `libgit2` directly rather than invoking `git status` via subshells, eliminating fork/exec latency.
- **Lightweight Concurrency**: Employs `std::thread::scope` bounded parallel scans, avoiding the overhead of heavy async runtimes for synchronous I/O operations.
- **Fail-Safe Isolation**: Scan errors on individual repositories are isolated into a `RepoStatus.error` field and displayed directly in the dashboard UI rather than crashing the process.
- **Offline & Private**: Never touches remote network endpoints; queries operate entirely on local tracking references.

---

## Development

```bash
# Check formatting
cargo fmt --check

# Run linter checks
cargo clippy --all-targets -- -D warnings

# Execute test suite
cargo test

# Build debug binary
cargo build

# Build optimized release binary
cargo build --release
```

---

## License

This project is licensed under the [MIT License](LICENSE) (or Apache-2.0 at your option).
