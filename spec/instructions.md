# reposcope — Dos and Don'ts

Practical working guidelines for building this project well, not just building it fast.
Written against the phased plan in tasks.md.

## Do

**Compiler & correctness**
- Do read every compiler error fully before reacting — Rust's borrow checker errors usually
  tell you the actual fix in the message itself (suggested lifetimes, `&mut` vs `&`, etc.).
  Fix the root cause it names, don't just sprinkle `.clone()` until it compiles.
- Do run `cargo clippy` regularly, not just `cargo build` — clippy catches idiomatic issues
  (needless clones, wrong iterator usage) the compiler won't flag.
- Do run `cargo fmt` before every commit so diffs stay clean and reviewable.
- Do write a quick manual test against real repos (per Phase 2's acceptance criteria) before
  trusting scanner output — verify against `git status`/`git log -1` ground truth, not
  assumption.

**Git hygiene (practice what the tool preaches)**
- Do commit at the end of each completed task-list checkbox, not at the end of a whole phase —
  smaller commits make it easy to bisect later if Phase 5's TUI breaks something Phase 3 relied on.
- Do write commit messages that say *why*, not just *what* — e.g. "Use thread::scope instead of
  tokio to avoid async runtime for a sync IO-bound scan" beats "update scanner.rs".
- Do keep `error.rs` changes and feature changes in separate commits when reasonably possible —
  makes the history easier to read later.

**Scope discipline**
- Do resist adding "just one more field" to `RepoStatus` mid-phase — if it's not in the current
  phase's task list, note it as a future idea and keep moving (the non-goals section exists for
  a reason).
- Do keep Phase 6 (packaging/README) honest — write the README as if a stranger has to build this
  with zero context, since future-you in 3 months is basically that stranger.

**Error handling**
- Do surface scan errors as a visible row/state (per NFR3), never swallow them silently — a
  repo that fails to open should be *obviously* broken in the dashboard, not just missing.

## Don't

**Compiler & correctness**
- Don't fight the borrow checker with `.clone()`/`Rc<RefCell<>>` as a first instinct — reach for
  those only after you understand *why* the checker is complaining, otherwise you're papering
  over a design issue that will resurface later (usually in the TUI event loop, which is where
  borrow issues tend to compound).
- Don't ignore `cargo clippy` warnings by blanket-adding `#[allow(...)]` — read what it's
  flagging first; suppress only once you've decided it's genuinely a false positive for your case.
- Don't let `unwrap()`/`expect()` creep into `scanner.rs` — this is exactly the module NFR3 exists
  for. A `.unwrap()` on a git2 call defeats the "never panic on a bad repo" requirement in one line.

**Git hygiene**
- Don't batch unrelated changes (e.g. "add staleness logic" + "fix TUI color bug") into one
  commit — it makes `git bisect` useless and makes code review (even self-review) harder.
- Don't commit `target/` or local `config.toml` test data — add a `.gitignore` in Phase 0, not
  as an afterthought in Phase 6.
- Don't force-push over shared history if you ever push this to a shared remote — irrelevant
  for a solo project but worth the habit given you're also doing this across ScratchORM/Typed Up.

**Scope & architecture**
- Don't reach for `tokio` "just in case" if `std::thread::scope` covers the actual need (per
  design.md's stated rationale) — an unused async runtime is dead weight and complicates the
  mental model for no payoff at this project's scale.
- Don't build the SQLite cache mentioned in design.md §9 preemptively — it's explicitly deferred
  until TOML+full-rescan is *measured* to be too slow, not assumed to be.
- Don't let Phase 5 (TUI) start before Phase 2's acceptance criteria are actually verified —
  debugging incorrect git data through a render loop is much slower than debugging it via a
  plain debug-print subcommand.

**Testing/validation**
- Don't trust a scan result on your own repos alone — include at least one path that's NOT a
  git repo and one that no longer exists on disk in your test set, so Phase 3's acceptance
  criteria (graceful per-repo errors) gets exercised honestly, not just assumed correct.

## Working efficiently with an AI pair-programmer (vibe-coding this spec)

**Do**
- Do work one phase (or even one checkbox) at a time when prompting — reference only the
  file(s) that phase touches, not the whole spec or whole codebase, every turn.
- Do ask for a diff/patch against an existing file instead of a full file regeneration when
  only a function or two changed — cheaper and easier to review.
- Do summarize prior context yourself in a sentence ("Phase 2 done, scanner.rs verified against
  ScratchORM/Typed Up") instead of re-pasting full previous outputs to "remind" the AI.
- Do paste compiler errors as-is (not paraphrased) when debugging — the exact message is the
  efficient path to a fix; a paraphrase often costs an extra round-trip.

**Don't**
- Don't paste entire multi-hundred-line files into a prompt when only asking about one function —
  point to the function/module by name and paste just that section.
- Don't ask for the same file to be regenerated in full repeatedly across small edits — batch
  changes and request them together, or ask for targeted patches.
- Don't re-justify settled decisions every prompt (e.g. re-explaining "why git2, not shelling
  out to git" each time) — that's already in design.md; reference it, don't restate it.
- Don't ask for speculative future-phase code ("also add the fetch feature while you're at it")
  in the middle of an earlier phase — it inflates the response and violates the phased scope
  discipline from the section above anyway.

