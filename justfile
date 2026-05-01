set positional-arguments

default:
    @just --list

# Install platform-specific development dependencies.
bootstrap:
    ./script/bootstrap

# Build and run Warp locally. Pass app args after `--`, for example: `just run -- --help`.
run *args:
    ./script/run {{args}}

# Build and run Warp in release mode.
run-release *args:
    ./script/run --release {{args}}

# Build the workspace.
build:
    cargo build --workspace

# Build the Warp app crate only.
build-app:
    cargo build -p warp

# Install the Warpium app binary to Cargo's bin directory as `warpium`.
install:
    cargo build -p warp --bin warp-oss --release --features gui
    mkdir -p "${CARGO_INSTALL_ROOT:-$HOME/.cargo}/bin"
    cp target/release/warp-oss "${CARGO_INSTALL_ROOT:-$HOME/.cargo}/bin/warpium"

# Fast typecheck for the Warp app library.
check:
    cargo check -p warp --lib

# Run all workspace tests with Cargo.
test:
    cargo test --workspace

# Run tests for a single package, for example: `just test-pkg warp_completer`.
test-pkg pkg:
    cargo test -p {{pkg}}

# Run the repo's nextest suite.
nextest:
    cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2

# Format Rust code.
fmt:
    cargo fmt

# Check Rust formatting.
fmt-check:
    cargo fmt -- --check

# Run clippy with warnings denied for the workspace.
clippy:
    cargo clippy --workspace --exclude warp_completer --all-targets --tests -- -D warnings

# Run the repo presubmit script: fmt, clippy, formatters, nextest, and doctests.
presubmit:
    ./script/presubmit
