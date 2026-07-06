# reposcope

`reposcope` is a terminal dashboard for tracking the state of local git repositories.
It shows branch state, ahead/behind counts, staged/unstaged/untracked changes, last commit
info, and staleness for a curated list of repos stored in `~/.config/reposcope/config.toml`.

## Install

```bash
git clone <repo-url>
cd reposcope
cargo build --release
```

The release binary is written to `target/release/reposcope`.

## Usage

```bash
reposcope add /path/to/repo
reposcope remove repo-name-or-path
reposcope list
reposcope
```

`reposcope` with no subcommand opens the TUI dashboard. From there, use `up`/`down` to move,
`enter` to quit and print the selected repo path, `r` to refresh, and `q` to quit.

## Configuration

The config file lives at `~/.config/reposcope/config.toml` and uses TOML.
The global `stale_threshold_days` value defaults to 7 days, and individual projects may
override it.

## Shell wrapper pattern

Changing the parent shell's directory requires a wrapper function. The jump-to-directory
behavior is deferred to v1.1, but this is the intended pattern:

```bash
reposcope() {
  local dir
  dir=$(command reposcope "$@" --print-cd-target)
  [ -n "$dir" ] && cd "$dir"
}
```

## Development

```bash
cargo fmt
cargo clippy
cargo build
cargo build --release
```
