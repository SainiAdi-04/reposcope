# reposcope — Agent Guide

Pre-implementation spec. No code exists yet. All source of truth is in `spec/`.

## Repo state

- **Pre-build**: no `Cargo.toml`, no `src/`, no lockfiles. `cargo` commands will fail.
- The entire project is documented in four spec files under `spec/`:
  - `requirements.md` — FR/NFR, success criteria
  - `design.md` — tech stack, data model, module layout, color rules, staleness logic
  - `tasks.md` — phased implementation checklist (Phase 0–7)
  - `instructions.md` — dos/dont's for Rust correctness, git hygiene, scope discipline, AI pairing

## Architecture facts (from spec, not yet in code)

- **Tech stack**: Rust, `ratatui` + `crossterm` for TUI, `git2` (libgit2 bindings) for git ops, `clap` for CLI, `toml`+`serde` for config, `chrono` for time.
- **Concurrency**: `std::thread::scope` (not tokio) for parallel repo scans.
- **Config location**: `~/.config/reposcope/config.toml` (XDG).
- **No network calls in v1** — ahead/behind uses local tracking refs only.
- **No panics on bad repos** — every scan error becomes `RepoStatus.error`, never a crash.
- **Staleness**: `is_stale = (last_commit_age > threshold) AND (no dirty files)`.
- **Build target**: single static binary, Linux primary target.

## Commands (will exist after implementation)

- `reposcope add <path>` — register a repo
- `reposcope remove <name-or-path>` — unregister
- `reposcope list` — plain-text listing (scriptable)
- `reposcope` (no subcommand) — TUI dashboard
- `cargo clippy` before `cargo build` — clippy catches idiomatic issues the compiler wont
- `cargo fmt` before every commit
- `cargo build --release` for a static-ish binary

## Key constraints from `instructions.md`

- No `.clone()`/`Rc<RefCell<>>` as first reaction to borrow checker — understand the error first.
- No blanket `#[allow(...)]` for clippy warnings.
- No `unwrap()`/`expect()` in `scanner.rs` — use the error-into-RepoStatus pattern.
- No tokio unless async runtime is actually needed.
- No SQLite cache preemptively — defer until TOML+rescan is measured too slow.
- No TUI before Phase 2 scanner correctness is verified against real repos.
- Commit per task checkbox, not per phase — small bisectable commits.
- Error repos must render as visible rows, not be silently dropped.
- Always verify scanner output against `git status`/`git log -1` ground truth.

## Build quirks (anticipated)

- `git2` may need `--features vendored-openssl` or similar on some platforms for static linking.

## Spec files order of authority

`design.md` > `requirements.md` > `tasks.md` > `instructions.md` when resolving ambiguity.

## Spec-driven development
This project follows spec/requirements.md, spec/design.md, spec/tasks.md, and
spec/instructions.md. Always check tasks.md for current phase before making changes.
Follow the phase acceptance criteria exactly; don't implement future phases early.