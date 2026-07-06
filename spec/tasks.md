# reposcope — Implementation Tasks

Phased so each milestone is independently runnable/demoable. Maps back to requirements.md
(FR/NFR references) and design.md modules.

## Phase 0 — Scaffolding

- [x] `cargo new reposcope`, add deps: `clap`, `serde`, `toml`, `git2`, `ratatui`, `crossterm`,
      `chrono` (for relative time formatting).
- [x] `error.rs` shared error enum (config errors, git errors, io errors).
- **Acceptance**: `cargo build` succeeds with empty subcommand stubs.

## Phase 1 — Config & Registration (FR1)

- [x] `config.rs`: load/save `~/.config/reposcope/config.toml`, create dir/file if missing.
- [x] `cli.rs`: `add <path>`, `remove <name-or-path>`, `list` subcommands via `clap`.
- [x] Validate path is a git repo on `add` (check `.git` exists or `git2::Repository::open`
      succeeds) — reject with clear error otherwise.
- [x] Reject duplicate paths on `add`.
- **Acceptance**: Can add 3 real repos from disk, `reposcope list` prints them, `remove` works,
  re-running `add` on a duplicate errors clearly. No TUI yet.

## Phase 2 — Single-repo status (FR2, subset of design.md §5)

- [x] `model.rs`: `RepoStatus`, `CommitInfo`, `BranchState`.
- [x] `scanner.rs`: implement scan for ONE repo — branch, ahead/behind, dirty counts, last
      commit. No concurrency yet, no TUI — just a `reposcope debug-scan <path>` hidden
      subcommand that prints the computed `RepoStatus` as debug output.
- **Acceptance**: Run against ScratchORM and Typed Up, manually verify counts match `git
status` / `git log -1` output for both.

## Phase 3 — Concurrent multi-repo scan (FR6, design.md §4b-d)

- [x] `scanner/concurrency.rs`: fan out scan across all registered projects using
      `std::thread::scope` (or bounded worker pool if project count could exceed ~30).
- [x] Errors per-repo captured into `RepoStatus.error`, never panic the whole scan (NFR3).
- **Acceptance**: Add 10+ repos (including one intentionally broken/missing path), scan
  completes without panicking, broken repo shows as an error entry, total time is close to
  the slowest single repo, not the sum of all repos.

## Phase 4 — Staleness (FR3)

- [x] Implement `is_stale` per design.md §6 rule.
- [x] Global `stale_threshold_days` in config, optional per-project override.
- **Acceptance**: A repo with an old last-commit and no dirty files is flagged stale; the same
  repo with an uncommitted change is NOT flagged stale.

## Phase 5 — TUI dashboard (FR4)

- [x] `tui/view.rs`: render table with columns from RepoStatus.
- [x] Color rules per design.md §7.
- [x] Default sort per design.md §7.
- [x] `tui/input.rs`: up/down navigation, `r` to re-scan, `q` to quit.
- **Acceptance**: Opening `reposcope` with no subcommand shows the live dashboard for all
  registered repos; `r` re-scans and updates in place; `q` exits cleanly with terminal state
  restored (no leftover raw-mode artifacts in the shell).

## Phase 6 — Polish & packaging (NFR1, NFR2)

- [x] `cargo build --release`, confirm single static-ish binary (note any dynamic linking
      caveats from `git2`'s libgit2 dependency — may need `--features vendored-openssl` or
      similar depending on platform).
- [x] README with install steps + the shell-wrapper snippet from design.md §4 (even if the
      jump-to-directory key itself is deferred to v1.1, document the pattern now).
- **Acceptance**: Fresh clone → `cargo build --release` → binary runs standalone.

## Phase 7 (v1.1, optional/stretch) — Jump-to-directory (FR5)

- [x] `--print-cd-target` flag / dedicated key that, on quit, prints selected repo path to
      stdout after terminal teardown.
- [ ] Shell function wrapper documented and tested in your actual `.bashrc`/`.zshrc`.
- **Acceptance**: Typing your shell alias, selecting a repo, pressing the jump key, and
  landing in that directory in the same shell session.

## Explicit Non-Tasks (matches requirements.md Non-Goals)

- No fetch/push/pull automation.
- No GitHub/GitLab API calls.
- No diff/content viewer.
- No background daemon.

## Suggested Order Rationale

Phases 1–2 are deliberately TUI-free so git2 correctness is validated against ground truth
(`git status`/`git log`) before any UI complexity is introduced — cheaper to debug data
correctness in a print statement than inside a Ratatui render loop.
