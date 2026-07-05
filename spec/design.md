# reposcope — Design

Derived from requirements.md. Covers architecture, data model, module breakdown, and key
technical decisions.

## 1. Tech Stack
| Concern              | Choice                          | Why |
|----------------------|----------------------------------|-----|
| Git operations        | `git2` (libgit2 bindings)        | No shelling out to `git`; structured data; fast for repeated status checks (FR2, NFR1) |
| TUI                   | `ratatui` + `crossterm`          | Already in your toolkit (DevDocs reader); consistent skillset |
| Config                | `toml` + `serde`                 | Human-editable, matches Rust ecosystem convention (NFR2) |
| Concurrency           | `tokio` (or `std::thread` + scoped threads) | Parallel repo scans (FR6) |
| Fuzzy actions (later) | `nucleo`                         | Only if repo list grows large enough to need filtering |
| CLI parsing           | `clap` (derive API)              | Standard, low-boilerplate subcommands (FR1) |

Decision: prefer `std::thread::scope` over pulling in full `tokio` if the only concurrency
need is "scan N repos in parallel and join" — avoids an async runtime dependency for a
fundamentally synchronous, CPU/IO-bound task. Use `tokio` only if you already plan async
features (e.g. live fetch) soon.

## 2. Data Model

```rust
struct Project {
    name: String,        // derived from folder name, override-able
    path: PathBuf,
    stale_threshold_days: Option<u32>, // per-project override, optional
}

struct RepoStatus {
    project: Project,
    branch: BranchState,          // Named(String) | Detached(String hash)
    ahead: u32,
    behind: u32,
    staged: u32,
    unstaged: u32,
    untracked: u32,
    last_commit: Option<CommitInfo>,
    is_stale: bool,
    error: Option<String>,        // populated instead of panicking (NFR3)
}

struct CommitInfo {
    short_hash: String,
    summary: String,
    author_date: DateTime<Utc>,
}
```

Config file (`~/.config/reposcope/config.toml`):
```toml
stale_threshold_days = 7

[[projects]]
name = "scratchorm"
path = "/home/babah/dev/ScratchORM"

[[projects]]
name = "typed-up"
path = "/home/babah/dev/typed-up"
stale_threshold_days = 14   # optional override
```

## 3. Module Breakdown

```
src/
  main.rs           — CLI entry, dispatches to subcommands or launches TUI
  cli.rs            — clap definitions: add / remove / list / (default = TUI)
  config.rs         — load/save config.toml, validate paths, dedupe on add
  scanner.rs         — git2-based repo inspection -> RepoStatus
  scanner/
    concurrency.rs   — fan-out scan across all registered projects
  model.rs           — Project, RepoStatus, CommitInfo, BranchState
  tui/
    mod.rs           — app state, event loop
    view.rs          — Ratatui rendering (table, color rules)
    input.rs          — keybinding handling (up/down/r/q, later: cd-on-quit)
  error.rs            — shared error type, ensures scan errors surface per-row, not as panics
```

## 4. Core Flow

1. `main.rs` parses args via `clap`.
2. Non-TUI subcommands (`add`/`remove`/`list`) mutate/read `config.toml` directly and exit.
3. Default (no subcommand) path:
   a. Load config → list of `Project`.
   b. `scanner::scan_all(projects)` spawns one thread per project (bounded pool if list is
      large, e.g. via a simple channel + worker pool — not one raw thread per repo beyond
      ~32, to avoid thread explosion on large lists).
   c. Each thread opens repo via `git2::Repository::open`, computes `RepoStatus`, catches
      errors into `RepoStatus.error` rather than propagating a panic (NFR3).
   d. Results joined into `Vec<RepoStatus>`, sorted per FR4 default sort.
   e. TUI event loop renders table, handles input, `r` re-triggers step (b)-(d).
4. On quit via the "jump" key (FR5), write selected repo's path to stdout *after* terminal
   is restored to normal mode, so a shell wrapper function can capture it:
   ```bash
   reposcope() {
     local dir
     dir=$(command reposcope "$@" --print-cd-target)
     [ -n "$dir" ] && cd "$dir"
   }
   ```

## 5. Ahead/Behind Computation (git2 specifics)
- Get current branch's upstream via `branch.upstream()`.
- If no upstream: ahead/behind = 0, display as "no upstream" rather than 0/0 (avoid implying
  it's in sync when it simply isn't tracked).
- Use `repo.graph_ahead_behind(local_oid, upstream_oid)` — this is the direct libgit2-provided
  computation, no manual walk needed.
- This reflects state as of last local `fetch` (per NFR4 / assumption in requirements.md) —
  no live network fetch performed.

## 6. Staleness Rule (from FR3)
```
is_stale = (now - last_commit.author_date > threshold)
           AND (staged == 0 AND unstaged == 0 AND untracked == 0)
```
A dirty repo is never marked stale, regardless of last commit age — it's "in progress," a
different state that should render distinctly (yellow) rather than grey.

## 7. Rendering / Color Rules (FR4)
| State | Condition | Color |
|---|---|---|
| Clean & synced | no dirty, ahead=0, behind=0 | green |
| Dirty | staged/unstaged/untracked > 0 | yellow |
| Ahead/behind/diverged | ahead>0 or behind>0, not dirty | blue/cyan, with ↑N ↓N indicator |
| Stale | per FR3 rule | dim grey |
| Error | scan failed | red, show `error` message in row |

Default sort: dirty repos first → diverged next → stale last → clean/synced in between,
tie-broken by most recent commit.

## 8. Error Handling Strategy (NFR3)
Every `git2` call in `scanner.rs` returns `Result`; scan function converts any `Err` into a
`RepoStatus { error: Some(msg), ..default-ish }` rather than bubbling up via `?` to the TUI
layer. The TUI always renders a full row per project, error or not — a missing/corrupted repo
should be visible, not silently dropped.

## 9. Deferred / v1.1 Ideas (explicitly out of v1 design)
- Live `fetch` on demand (`f` key) — network + auth complexity, isolated as its own feature.
- Aggregate stats bar (total commits today, longest streak).
- README/doc freshness flag.
- SQLite-backed cache if TOML+full-rescan ever becomes too slow (unlikely at expected repo
  counts — defer until FR6 is actually violated in practice).
