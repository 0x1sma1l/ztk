# Contributing to Zet

Thanks for helping improve Zet. Keep changes focused, preserve the local-first file format, and put business rules in `src/core/` rather than duplicating them in CLI or TUI adapters.

## Development workflow

1. Install stable Rust with the `rustfmt` and `clippy` components.
2. Create a focused branch and make one coherent change.
3. Add unit tests for core policies and integration tests for observable CLI, storage, or TUI behavior.
4. Run the same gate used by CI:

   ```bash
   ./scripts/check.sh
   ```

5. Update the README when commands, keybindings, file formats, or limitations change.

Do not commit generated `target/` artifacts, personal notes, or local planning documents. Avoid changing note serialization without round-trip and compatibility tests.

## Commit and review expectations

- Explain the user-visible behavior and why the change belongs in its chosen layer.
- Include success, failure, and relevant edge cases.
- Keep stdout for requested data and stderr for warnings or errors.
- Preserve terminal restoration on every TUI exit path.

Bug reports should include the command or key sequence, expected behavior, actual behavior, operating system, and `rustc --version` output. Security-sensitive reports should not include private note contents.
