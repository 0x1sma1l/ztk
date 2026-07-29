# Ztk

Ztk is a local-first, keyboard-driven Markdown note manager for the terminal. It keeps notes as ordinary files, provides scriptable CLI commands, and includes a full-screen TUI for fast browsing.

The project is currently under active development. Its note storage, shared core use cases, CLI v1 workflows, and keyboard-driven TUI mutations are working.

## Why Ztk?

- **Local first:** your notes stay on disk as readable Markdown.
- **Terminal first:** create, inspect, edit, lint, and browse without leaving the shell.
- **Editor friendly:** write note bodies using `$VISUAL`, `$EDITOR`, or an automatically detected terminal editor.
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
- Interactively fuzzy-find notes with `fzf`, then open the selection in your editor.
- Move notes to recoverable repository-local trash and restore or purge them explicitly.
- Lint note metadata and automatically remove duplicate tags.
- Show the current note count.

### TUI

- Browse real notes in a stateful list.
- Read the selected note in a focused surface or edit it with `$VISUAL`/`$EDITOR` directly inside the TUI.
- Navigate with Vim-style keys or arrow keys.
- Jump to the first or last note.
- Scroll long previews by line or page, including wrapped Unicode content.
- Switch to a stacked list/preview layout on narrow terminals.
- Create notes and launch a centered, two-pane `fzf` search without leaving the TUI workflow.
- Edit the selected note's title, tags, or body through validated core operations.
- Delete the selected note through an explicit confirmation mode.
- Refresh notes from disk.
- Recover readable notes when some files are malformed.
- View load, refresh, and skipped-file status.
- Open an in-app keyboard-help overlay.
- Restore the terminal safely on normal exits, errors, and panic unwinding.

## Installation

Ztk currently builds from source and requires Rust 1.85 or newer. Interactive search also requires [`fzf`](https://github.com/junegunn/fzf).

Install `fzf` with your platform package manager:

```bash
brew install fzf                    # macOS
sudo apt install fzf                # Ubuntu/Debian
winget install junegunn.fzf         # Windows
```

Package-manager distributions of Ztk should declare `fzf` as a runtime dependency. Cargo cannot install external system executables, so users installing with `cargo install` must install `fzf` separately.

```bash
git clone https://github.com/0x1sma1l/ztk.git
cd ztk
cargo build --release
```

The binary will be available at:

```text
target/release/ztk
```

You can also install it directly from the local checkout:

```bash
cargo install --path .
```

After it is published on crates.io, install it by package name:

```bash
cargo install ztk
```

By default, Ztk stores notes in a stable per-user data directory and creates it automatically on first run. You can choose another location with `--notes-dir`, `ZTK_NOTES_DIR`, or the config file described below.

## Quick start

During development, pass Ztk's arguments after Cargo's `--` separator:

```bash
cargo run -- list
cargo run -- tui
```

Use `cargo run --quiet -- <command>` when you do not want Cargo's build output. Do not repeat the binary name after `--`.

Create a note and open it immediately in `$VISUAL`, `$EDITOR`, or the first available `nvim`, `vim`, `vi`, or `nano`:

```bash
ztk new "Rust Ownership" --tags rust,learning
```

List notes:

```bash
ztk list
```

View or edit a note:

```bash
ztk view rust-ownership
ztk edit rust-ownership
ztk update rust-ownership --title "Ownership in Rust" --tags rust,learning
ztk search
```

Check the note collection:

```bash
ztk lint
ztk lint --fix
ztk stats
```

Launch the TUI:

```bash
ztk tui
```

Delete a note:

```bash
ztk delete rust-ownership --force
```

Deletion moves the note into `<notes-dir>/.trash` and is recoverable. Ztk asks for confirmation in an interactive terminal; scripts and redirected invocations must opt in with `--force`.

List and restore deleted notes, or explicitly purge one forever:

```bash
ztk trash list
ztk trash restore <trash-id>
ztk purge <trash-id>
ztk purge --all
```

## Commands

| Command | Description |
| --- | --- |
| `ztk new <title> [--tags <tags>]` | Create a note, then open it in the configured editor. Tags are comma separated. |
| `ztk list` | List note slugs, dates, and tags. |
| `ztk view <slug>` | Render a note's Markdown body. |
| `ztk edit <slug>` | Open a note in `$VISUAL`/`$EDITOR` and update its modification date. |
| `ztk update <slug> [--title ...] [--tags ...] [--body ...]` | Update selected note fields without opening an editor. |
| `ztk search [query]` | Launch `fzf` over the configured notes repository, optionally seeded with a query, then edit the selected note. |
| `ztk lint [--fix]` | Validate notes and optionally apply supported repairs. |
| `ztk stats` | Display the total number of readable notes. |
| `ztk delete <slug> [--force]` | Move one note to recoverable trash, with confirmation by default. |
| `ztk trash list` | List recoverable trash entries and their deletion times. |
| `ztk trash restore <id>` | Restore an entry if its original slug is free. |
| `ztk purge <id>` | Permanently remove one trash entry after interactive confirmation. |
| `ztk purge --all` | Permanently remove all trash after interactive confirmation. |
| `ztk tui` | Launch the full-screen terminal interface. |

Run `ztk --help` or `ztk <command> --help` for command-line help.

## Notes directory configuration

Ztk resolves one notes directory at startup and passes it to both CLI and TUI operations. Sources have this precedence:

1. `--notes-dir <path>`
2. `ZTK_NOTES_DIR`
3. `notes_dir` in the config file
4. `$XDG_DATA_HOME/ztk/notes`, `%LOCALAPPDATA%/ztk/notes`, or `$HOME/.local/share/ztk/notes`

Set `ZTK_CONFIG` to use a specific config file. Otherwise Ztk checks `$XDG_CONFIG_HOME/ztk/config.toml`, `%APPDATA%/ztk/config.toml`, or `$HOME/.config/ztk/config.toml`, as appropriate. A config file contains:

```toml
notes_dir = "/absolute/path/to/notes"
```

Relative command-line and environment paths resolve from the working directory. Relative config values resolve from the directory containing the config file. Ztk creates the resolved notes directory automatically when it starts. The TUI header displays the active repository path.

### CLI v1 contract

CLI v1 consists of the commands in the table above. Successful commands exit with status `0`; runtime or data errors exit with status `1`; command-line parsing errors exit with status `2`. Normal results go to stdout, while errors and warnings go to stderr. Collection commands may return healthy results with status `0` while warning that malformed notes were skipped.

Human output follows a stable stream convention: requested records and successful mutation summaries go to stdout; recoverable diagnostics begin with `warning:` on stderr; fatal diagnostics begin with `error:` on stderr and use a non-zero status. Interactive output uses a restrained Vesper-derived palette and borderless aligned tables, while redirected output contains no ANSI color sequences. Ztk does not currently expose JSON: machine-readable output will be added only with a versioned schema shared across commands, rather than freezing inconsistent ad hoc shapes.

Bulk deletion is not part of v1; single-entry trash operations keep collision and partial-failure behavior understandable before bulk workflows are designed. Tag and date filters are also deferred: search already retrieves notes by tags, while exact filter syntax should be designed together with stable machine-readable output instead of becoming an ad hoc argument set.

## TUI controls

| Key | Action |
| --- | --- |
| `j` / `Down` | Select the next note. |
| `k` / `Up` | Select the previous note. |
| `g` / `Home` | Jump to the first note. |
| `G` / `End` | Jump to the last note. |
| `Enter` | Focus the note surface in read mode. |
| `j` / `k`, `[` / `]` | Scroll the focused note one visual line. |
| `Page Up` / `Page Down` | Scroll the focused note one viewport. |
| `n` | Create a note by title. |
| `/` | Open the centered fuzzy-search panel for slugs, titles, and tags, with live note preview. |
| `e` | Open or reattach `$VISUAL`/`$EDITOR` inside the note surface. |
| `F6` | Detach without applying saved editor changes; they apply after the editor exits. |
| `d` | Request deletion of the selected note; `y` confirms and `n`/`Esc` cancels. |
| `r` | Reload notes from disk. |
| `h` / `?` | Toggle the help overlay. |
| `Esc` | Close help, or quit when help is closed. |
| `q` / `Ctrl-C` | Quit. |

Selection wraps at the beginning and end of the list.

Editor selection uses `$VISUAL` first, then `$EDITOR`. When neither is set, Ztk uses the first available `nvim`, `vim`, `vi`, or `nano` found on `PATH`; if none is installed, it reports how to configure or install one. An explicitly configured editor that cannot be launched remains an error instead of being silently replaced.

When Ztk launches Vi, Vim, or Neovim from either the CLI or TUI, it enables absolute and relative line numbers for easier navigation. When the embedded editor is attached, it owns every ordinary key, including `Esc`, `Ctrl-C`, and Vim commands. `:w` saves only the editor's temporary working copy. Ztk validates and applies the last saved copy after the editor exits successfully with `:q`, `:wq`, or another normal editor command. `F6` only detaches: it returns to read mode while leaving the editor running, so the read surface continues to show the last applied note until you reattach and exit the editor. The configured `$VISUAL`/`$EDITOR` command must be a terminal editor such as `nvim`, `vim`, `vi`, or `nano`.

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

Both fields are validated calendar dates serialized as `YYYY-MM-DD`. `date` is the note's creation day in the machine's local timezone. `updated_at` is the local calendar day of the last content or metadata change; Ztk intentionally stores day precision rather than a timestamp. Invalid legacy values never enter the typed domain model and are reported by `ztk lint` with the affected field and value.

Changing a title with `ztk update` does not rename the note slug or file. Slugs are stable identifiers; automatic renames could break links and require collision handling. Pass `--tags=` or `--body=` to clear those fields. An update that changes nothing leaves the file and `updated_at` untouched.

## Lint behavior

The linter currently detects:

- Missing titles.
- Missing or invalid `YYYY-MM-DD` dates.
- Duplicate tags, compared case-insensitively.
- Missing or malformed frontmatter.

`ztk lint --fix` currently repairs duplicate tags. It exits non-zero whenever issues remain after the fix pass, making it suitable for scripts and CI.

## Architecture

Ztk uses a ports-and-adapters structure:

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

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow, architecture boundaries, test expectations, and bug-report guidance. Pull requests run formatting, Clippy, packaging, and the full test suite on Linux, macOS, and Windows, including the declared Rust 1.85 minimum.

## Packaging and releases

The crate includes crates.io metadata and can be validated locally with `cargo package --locked`. Release builds use thin LTO and strip symbols. The current supported installation path is `cargo install --path .`; publishing to crates.io or attaching downloadable binaries requires an intentional release decision and is not automated yet.

Ztk is available under the [MIT License](LICENSE).

## Known limitations

- TUI body input is currently a single-line prompt; use `ztk edit <slug>` for comfortable multiline authoring.
- Interactive search depends on the external `fzf` executable when Ztk is installed through Cargo or a standalone binary.

## Roadmap

Near-term work is focused on:

1. Consistent partial-failure handling for malformed notes.
2. Strict and compatible frontmatter parsing.
3. Bulk trash management after single-entry workflows stabilize.
4. Package-manager releases that install the `fzf` runtime dependency automatically.
5. Explicit note renaming and full TUI feature parity.
6. Broader CLI/TUI integration coverage, CI, and packaging.

## Project status

Ztk is a learning-driven, pre-1.0 project. File formats and commands are being stabilized deliberately, with regression tests added before or alongside each behavior change.
