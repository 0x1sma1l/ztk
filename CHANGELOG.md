# Changelog

All notable user-facing changes to Ztk are documented here.

The project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before 1.0, minor releases may contain workflow-breaking changes; those changes are called out explicitly.

## [Unreleased]

## [0.5.0] - 2026-07-29

### Changed

- Added a restrained Vesper orange-and-neutral TUI palette with consistent focus, selection, hierarchy, and overlay styling while preserving the terminal background.
- Refined interactive CLI output with the same restrained Vesper palette, aligned collection and lint output, and consistent action, warning, and error hierarchy while keeping redirected output plain.
- Clarified that `F6` only detaches the embedded editor and that saved changes are validated and applied after the editor exits.

## [0.4.0] - 2026-07-29

### Added

- Added focused browse and read modes to the TUI note surface.
- Added an embedded PTY-backed `$VISUAL`/`$EDITOR` session inside the note pane, with ordinary editor key ownership, live resizing, safe temporary-file validation, and `F6` detach/reattach support.

### Changed

- Renamed the TUI's default `NORMAL` mode to `BROWSE` to distinguish application navigation from Vim's normal mode.
- Removed the TUI's separate title, tag, and body controls; note metadata remains editable through the embedded editor and CLI.
- Made `Enter` the sole shortcut for read mode, narrowed the notes list to leave more room for the note surface, and added a visible `q`/`Esc` exit hint.

## [0.3.0] - 2026-07-28

### Added

- Added an embedded, centered fuzzy-search modal to the TUI with live `fzf`-ranked results and a side-by-side note preview.
- Added keyboard-driven search navigation, selection, cancellation, and query clearing without leaving the TUI.

### Changed

- Kept the underlying notes interface visible around the search modal instead of allowing `fzf` to take over full terminal rows.
- Reduced the search modal footprint and made its results and preview stack vertically on narrow terminals.
- Simplified the TUI chrome by removing excess borders, adopting the terminal background, and using a consistent neutral text palette.
- Simplified the main preview heading by removing its scroll-position counter.

## [0.2.0] - 2026-07-28

### Breaking changes

- Renamed permanent deletion from `ztk trash purge <id> --force` to `ztk purge <id>`. Purging one entry or all entries now requires confirmation from an interactive terminal.
- Changed `ztk new <title>` to open the newly created note immediately in `$VISUAL`, `$EDITOR`, or `vi`. The note remains saved if the editor fails.
- Replaced printed search results with interactive `fzf` selection. `ztk search [query]` now opens the selected note in the configured editor, and `fzf` is a required external dependency for search.

### Added

- Added `ztk purge --all` to permanently empty repository trash after interactive confirmation.
- Added `fzf` search to the TUI. Pressing `/` temporarily yields the terminal to `fzf`, then returns with the chosen note selected and previewed.
- Added platform-specific guidance when `fzf` is unavailable.

### Changed

- Restricted interactive search candidates to readable notes in the active configured notes directory.
- Made the search query optional so `ztk search` starts with an empty live query while `ztk search <query>` seeds the initial filter.

## [0.1.0] - 2026-07-27

### Added

- Added local-first Markdown note creation, listing, viewing, editor-based editing, structured updates, fuzzy search, linting, statistics, safe deletion, recoverable trash, restore, and permanent purge workflows.
- Added the full-screen keyboard-driven TUI with navigation, responsive previews, note creation and updates, deletion confirmation, refresh, help, and partial recovery from malformed notes.
- Added configurable notes-directory discovery through CLI options, environment variables, config files, and stable platform data directories.
- Added strict slug and storage-boundary validation, compatible frontmatter parsing, typed calendar dates, and recoverable handling of malformed notes.
- Added cross-platform CI, release packaging, CLI integration tests, TUI tests, and Rust 1.85 compatibility.

[Unreleased]: https://github.com/0x1sma1l/ztk/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/0x1sma1l/ztk/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/0x1sma1l/ztk/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/0x1sma1l/ztk/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/0x1sma1l/ztk/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/0x1sma1l/ztk/releases/tag/v0.1.0
