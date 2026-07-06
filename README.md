# reposcope

`reposcope` is a simple terminal dashboard for your local git projects.
It shows which repos have changes, which are ahead or behind their upstream, and which look stale.

## Install globally

```bash
git clone <repo-url>
cd reposcope
cargo build --release
cp target/release/reposcope ~/.local/bin/
```

Make sure `~/.local/bin` is on your `PATH` so you can run `reposcope` from anywhere.

## Use it

```bash
reposcope add /path/to/project
reposcope add /path/to/another-project
reposcope list
reposcope
```

`reposcope add` registers a local git repo for tracking.
`reposcope list` shows the projects you already added.
`reposcope remove <name-or-path>` removes one.

Running `reposcope` with no subcommand opens the dashboard.
Use `up`/`down` to move, `r` to refresh, `q` to quit, and `enter` to quit while printing the
selected repo path.

## Configuration

Your tracked projects are stored in `~/.config/reposcope/config.toml`.
The default stale threshold is 7 days.

## Shell wrapper pattern

If you want to use the selected repo path to `cd` in your shell, add a wrapper like this:

```bash
reposcope() {
  local dir
  dir=$(command reposcope "$@" --print-cd-target)
  [ -n "$dir" ] && cd "$dir"
}
```

## Development

If you change the code, these are the useful commands:

- `cargo fmt` formats the code
- `cargo clippy` checks for common mistakes
- `cargo build` builds a normal debug version
- `cargo build --release` builds the optimized version
