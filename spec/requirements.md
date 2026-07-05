# reposcope — Requirements

## 1. Problem Statement
Multiple local git repositories (ScratchORM, Typed Up, terminal.chat, wc-notify, Oblivio, etc.)
are worked on in parallel. There is no single place to see, at a glance, which repos have
uncommitted work, which are behind/ahead of origin, and which have gone stale. This tool
provides a terminal dashboard over a user-curated list of local git repos.

## 2. Goals (v1)
- G1: Register/unregister local git repo paths persistently.
- G2: Show, per registered repo: current branch, ahead/behind count vs upstream,
  uncommitted change counts (staged/unstaged/untracked), last commit summary + relative time.
- G3: Flag "stale" repos (no commits in N days, configurable).
- G4: Refresh all repos in parallel without noticeable UI lag for ~20 repos.
- G5: Single static binary, no external runtime dependencies (no shelling out to `git` required).

## 3. Non-Goals (v1)
- NG1: No remote git operations (push/pull/fetch triggered from the tool) — read-only.
- NG2: No multi-user / team dashboard, single local user only.
- NG3: No GitHub/GitLab API integration (PRs, issues) — pure local git state.
- NG4: No file-level diff viewer — counts only, not content.
- NG5: No daemon/background process in v1 — on-demand scan when the TUI opens (may revisit post-v1).

## 4. Users & Usage Context
Single user (you), invoked as a CLI command (`reposcope`) from any shell, typically opened in a
dedicated tmux window per your existing per-task window layout. Expected to run interactively;
session length: seconds to a couple minutes per open.

## 5. Functional Requirements

### FR1 — Project registration
- `reposcope add <path>` — adds a path as a tracked project. Validates it is a git repo
  (contains `.git`) before adding. Errors clearly if not.
- `reposcope remove <name-or-path>` — removes a tracked project.
- `reposcope list` — plain-text listing of tracked projects (non-TUI, scriptable/pipeable).
- Duplicate paths are rejected with a clear message, not silently ignored.

### FR2 — Repo status computation
For each tracked repo, compute:
- Current branch name (or "detached HEAD" state).
- Ahead/behind counts relative to the configured upstream (if any upstream exists).
- Counts of: staged changes, unstaged modifications, untracked files.
- Last commit: short hash, first line of message, author date (relative, e.g. "3h ago").
- Total commit count on current branch (optional/secondary — nice-to-have, not blocking).

### FR3 — Staleness detection
- Configurable threshold (default: 7 days) — repo flagged "stale" if last commit is older
  than threshold AND there are no uncommitted changes either (a repo mid-edit isn't "stale",
  it's "in progress").

### FR4 — Dashboard TUI
- Table/list view of all tracked repos with the FR2 fields as columns.
- Color coding: clean (green), uncommitted changes (yellow), stale (grey/dim), ahead-only
  vs behind vs diverged (distinct indicators).
- Keyboard navigation: up/down to select a repo, `r` to refresh all, `q` to quit.
- Sorting: default sort = dirty repos first, then most-recently-active.

### FR5 — Quick actions (stretch for v1, required for v1.1)
- Press a key on a selected repo to print its path to stdout on quit (so a shell wrapper
  function can `cd` there) — since a child process cannot change the parent shell's directory.
- Document the shell-function wrapper pattern in the README (this is a known constraint of
  any CLI tool wanting to affect `cd`, not a limitation specific to this design).

### FR6 — Performance
- Scanning 20 repos should complete and render within ~1–2 seconds on typical local SSDs.
- Repo scans run concurrently, not sequentially.

## 6. Non-Functional Requirements
- NFR1: Single static binary via `cargo build --release`, cross-compilable to at least Linux
  (primary target, matches your Ubuntu setup).
- NFR2: Config and cache stored under `~/.config/reposcope/` (XDG-compliant).
- NFR3: No panics on malformed/missing repos — degrade gracefully, show an error row instead
  of crashing the whole dashboard.
- NFR4: No network calls in v1 (ahead/behind uses local tracking refs, not live fetch —
  freshness of ahead/behind is only as good as the last manual/automatic `git fetch`).

## 7. Open Questions / Assumptions
- Assumption: "ahead/behind" is computed against the existing local tracking ref, not a live
  `fetch` — a live fetch would violate NFR4 (no network) and add latency/failure surface.
  If you want live-fetch behavior, that's a clearly separable v1.1 feature (explicit `f` key
  to fetch a selected repo).
- Assumption: config format is TOML (matches your existing Rust/tooling conventions).
- Open: should stale threshold be global-only or per-project override? (Recommend: global
  default + optional per-project override in config, deferred unless you want it in v1.)

## 8. Success Criteria
- You can register all your active repos in under a minute total.
- Opening the dashboard answers "what needs my attention today" without running `git status`
  manually in each repo.
