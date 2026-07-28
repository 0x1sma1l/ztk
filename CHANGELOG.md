# Changelog

All notable user-facing changes to Ztk are documented here.

The project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before 1.0, minor releases may contain workflow-breaking changes; those changes are called out explicitly.

## [Unreleased]

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

[Unreleased]: https://github.com/0x1sma1l/ztk/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/0x1sma1l/ztk/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/0x1sma1l/ztk/releases/tag/v0.1.0
