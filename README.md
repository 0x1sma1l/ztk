# Zet

Zet is a local-first, keyboard-driven Markdown note manager for the terminal. It keeps notes as ordinary files, provides scriptable CLI commands, and includes a full-screen TUI for fast browsing.

The project is currently under active development. Its foundations—note storage, core use cases, CLI workflows, validation, and TUI navigation—are working, while search and full TUI editing are still being built.

## Why Zet?

- **Local first:** your notes stay on disk as readable Markdown.
- **Terminal first:** create, inspect, edit, lint, and browse without leaving the shell.
- **Editor friendly:** write note bodies using `$VISUAL`, `$EDITOR`, or the default `vi`.
- **Script aware:** commands use meaningful exit codes, including lint failures.
- **Shared core:** CLI and TUI behavior is built on the same domain and storage layers.

## Current features

### CLI

- Create notes with optional tags.
- Generate safe, unique slugs from titles.
- List readable notes and warn about malformed files without hiding healthy notes.
- Render a note's Markdown in the terminal.
- Edit notes safely using `$VISUAL`/`$EDITOR`, including quoted commands and editor flags.
- Update titles, tags, or bodies directly without opening an editor.
- Fuzzy-search note slugs, titles, and tags with deterministic ranking.
- Delete a note by slug.
- Lint note metadata and automatically remove duplicate tags.
- Show the current note count.

### TUI

- Browse real notes in a stateful list.
- Preview the selected note and its metadata.
- Navigate with Vim-style keys or arrow keys.
- Jump to the first or last note.
- Refresh notes from disk.
- Recover readable notes when some files are malformed.
- View load, refresh, and skipped-file status.
- Open an in-app keyboard-help overlay.
- Restore the terminal safely on normal exits, errors, and panic unwinding.

## Installation

Zet currently builds from source and requires a recent stable Rust toolchain.

```bash
git clone https://github.com/0x1sma1l/zet.git
cd zet
cargo build --release
```

The binary will be available at:

```text
target/release/zet
```

You can also install it directly from the local checkout:

```bash
cargo install --path .
```

> Zet currently stores notes in a `notes/` directory relative to the directory from which it is run. Run commands from the same project or notes directory to access the same collection.

## Quick start

During development, pass Zet's arguments after Cargo's `--` separator:

```bash
cargo run -- list
cargo run -- tui
```

Use `cargo run --quiet -- <command>` when you do not want Cargo's build output. Do not repeat the binary name after `--`.

Create a note:

```bash
zet new "Rust Ownership" --tags rust,learning
```

List notes:

```bash
zet list
```

View or edit a note:

```bash
zet view rust-ownership
zet edit rust-ownership
zet update rust-ownership --title "Ownership in Rust" --tags rust,learning
zet search ownership
```

Check the note collection:

```bash
zet lint
zet lint --fix
zet stats
```

Launch the TUI:

```bash
zet tui
```

Delete a note:

```bash
zet delete rust-ownership --force
```

Deletion is permanent. Zet asks for confirmation in an interactive terminal; scripts and redirected invocations must opt in explicitly with `--force`.

## Commands

| Command | Description |
| --- | --- |
| `zet new <title> [--tags <tags>]` | Create a note. Tags are comma separated. |
| `zet list` | List note slugs, dates, and tags. |
| `zet view <slug>` | Render a note's Markdown body. |
| `zet edit <slug>` | Open a note in `$VISUAL`/`$EDITOR` and update its modification date. |
| `zet update <slug> [--title ...] [--tags ...] [--body ...]` | Update selected note fields without opening an editor. |
| `zet search <query>` | Fuzzy-search note slugs, titles, and tags. |
| `zet lint [--fix]` | Validate notes and optionally apply supported repairs. |
| `zet stats` | Display the total number of readable notes. |
| `zet delete <slug> [--force]` | Permanently delete one note, with confirmation by default. |
| `zet tui` | Launch the full-screen terminal interface. |

Run `zet --help` or `zet <command> --help` for command-line help.

## TUI controls

| Key | Action |
| --- | --- |
| `j` / `Down` | Select the next note. |
| `k` / `Up` | Select the previous note. |
| `g` / `Home` | Jump to the first note. |
| `G` / `End` | Jump to the last note. |
| `r` | Reload notes from disk. |
| `h` / `?` | Toggle the help overlay. |
| `Esc` | Close help, or quit when help is closed. |
| `q` / `Ctrl-C` | Quit. |

Selection wraps at the beginning and end of the list.

## Note format

Each note is stored as `notes/<slug>.md` with YAML frontmatter and a Markdown body:

```markdown
---
title: Rust Ownership
date: 2026-07-27
tags:
- rust
- learning
updated_at: 2026-07-27
---

# Rust Ownership

Your note starts here.
```

Valid slugs contain ASCII letters, digits, and internal hyphens. Storage operations validate slugs before accessing the filesystem.

For compatibility, legacy notes without `updated_at` use their original `date` as the in-memory fallback. Newly saved notes always include `updated_at`.

Changing a title with `zet update` does not rename the note slug or file. Slugs are stable identifiers; automatic renames could break links and require collision handling. Pass `--tags=` or `--body=` to clear those fields. An update that changes nothing leaves the file and `updated_at` untouched.

## Lint behavior

The linter currently detects:

- Missing titles.
- Missing or invalid `YYYY-MM-DD` dates.
- Duplicate tags, compared case-insensitively.
- Missing or malformed frontmatter.

`zet lint --fix` currently repairs duplicate tags. It exits non-zero whenever issues remain after the fix pass, making it suitable for scripts and CI.

## Architecture

Zet uses a ports-and-adapters structure:

```text
CLI / TUI
    │
    ▼
Core use cases and validation
    │
    ▼
NoteRepository trait
    │
    ▼
LocalMarkdownRepo
    │
    ▼
notes/*.md
```

The main source areas are:

- `src/core/`: domain model, validation, repository contract, and use cases.
- `src/storage/`: Markdown/YAML parsing and local filesystem persistence.
- `src/cli/`: command adapters and terminal output.
- `src/tui/`: application state, event handling, and Ratatui rendering.
- `tests/`: repository and CLI integration coverage.

Business rules belong in core, filesystem and serialization behavior in storage, and presentation concerns in CLI/TUI adapters.

## Development

Run the complete local quality suite:

```bash
./scripts/check.sh
```

Or run its steps individually:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

The test suite includes validator and frontmatter unit tests, repository integration tests, CLI lint process tests, TUI state/event/render tests, and terminal-cleanup tests.

## Known limitations

- The TUI currently browses notes but does not create, edit, search, or delete them.
- Delete has confirmation but no trash/restore workflow yet.
- Long TUI previews do not scroll yet.
- The notes directory is tied to the current working directory; configurable repository discovery is planned.

## Roadmap

Near-term work is focused on:

1. Consistent partial-failure handling for malformed notes.
2. Strict and compatible frontmatter parsing.
3. Recoverable deletion with trash and restore operations.
4. Live TUI search backed by the shared core search use case.
5. Explicit note renaming and full TUI feature parity.
6. Broader CLI/TUI integration coverage, CI, and packaging.

## Project status

Zet is a learning-driven, pre-1.0 project. File formats and commands are being stabilized deliberately, with regression tests added before or alongside each behavior change.
