# Repository Guidelines

## Project Structure & Module Organization

Warp is a Rust Cargo workspace. The main desktop client lives in `app/`, with source under `app/src`, assets in `app/assets`, and app-level tests in `app/tests`. Shared libraries are in `crates/*`; important crates include `crates/warpui` and `crates/warpui_core` for the custom UI framework, `crates/warp_terminal` for terminal behavior, `crates/editor` for editing primitives, and `crates/integration` for end-to-end tests. Contribution specs live in `specs/GH<issue-number>/`. Agent and repo workflow guidance lives in `.agents/skills/`, while broader engineering notes are in `WARP.md`.

## Build, Test, and Development Commands

- `./script/bootstrap`: install platform-specific build prerequisites.
- `./script/run` or `cargo run`: build and run Warp locally.
- `cargo run --features with_local_server`: run the client against a local server; set `SERVER_ROOT_URL` and `WS_SERVER_URL` for non-default ports.
- `./script/presubmit`: run the expected local gate: formatting, clippy, clang-format, and tests.
- `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2`: run workspace tests.
- `cargo nextest run -p warp_completer --features v2`: run completer tests with v2 enabled.
- `cargo test --doc`: run Rust documentation tests.

## Coding Style & Naming Conventions

Use `cargo fmt`; Rust formatting is configured in `.rustfmt.toml` with edition 2018. Clippy warnings are errors in presubmit. Prefer concise imports over long path qualifiers, inline format args such as `format!("{value}")`, and exhaustive `match` arms instead of catch-all `_` where possible. Context parameters named `ctx` should usually be last. For C/C++/Objective-C touched under `crates/warpui/src` or `app/src`, run `./script/run-clang-format.py -r --extensions 'c,h,cpp,m' ./crates/warpui/src/ ./app/src/`.

## Testing Guidelines

Add regression tests for bug fixes and unit tests for non-trivial logic. Rust unit test files usually use `${filename}_tests.rs` or `mod_test.rs`, included from the owning module with `#[cfg(test)]`. User-facing flows should prefer integration coverage in `crates/integration/` when practical. Run `./script/presubmit` before opening or updating a PR.

## Commit & Pull Request Guidelines

Branch from `master`; contributor branches should be prefixed with a handle, for example `alice/fix-parser`. Commit messages should explain what changed and why. Feature requests need a ready label and often a spec PR before code; triaged bugs are ready to implement. Use `.github/pull_request_template.md`, include a clear description and testing notes, link the issue, add screenshots for UI changes, and add changelog lines such as `CHANGELOG-BUG-FIX:` when user-visible.

## Security & Configuration Tips

Do not open public issues for vulnerabilities; follow `SECURITY.md`. Avoid committing local server URLs, credentials, tokens, logs with secrets, or machine-specific config.
