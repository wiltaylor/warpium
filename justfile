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

# Install the Warpium app binary and a user desktop entry.
install:
    cargo build -p warp --bin warp-oss --release --features gui
    install_root="${CARGO_INSTALL_ROOT:-$HOME/.cargo}"; install -Dm755 target/release/warp-oss "$install_root/bin/warpium"
    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"; install -Dm644 app/channels/oss/icon/no-padding/512x512.png "$data_home/icons/hicolor/512x512/apps/warpium.png"
    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"; install_root="${CARGO_INSTALL_ROOT:-$HOME/.cargo}"; install -d "$data_home/applications"; printf '%s\n' '[Desktop Entry]' 'Version=1.0' 'Type=Application' 'Name=Warpium' 'GenericName=Terminal Emulator' "Exec=$install_root/bin/warpium %U" 'StartupWMClass=dev.warp.Warpium' 'Keywords=shell;prompt;command;commandline;cmd;' 'Icon=warpium' 'Categories=System;TerminalEmulator;' 'Terminal=false' 'MimeType=x-scheme-handler/warposs;' > "$data_home/applications/warpium.desktop"
    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"; command -v update-desktop-database >/dev/null && update-desktop-database "$data_home/applications" || true
    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"; command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q "$data_home/icons/hicolor" || true

# Remove the Warpium app binary and user desktop entry.
uninstall:
    install_root="${CARGO_INSTALL_ROOT:-$HOME/.cargo}"; rm -f "$install_root/bin/warpium"
    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"; rm -f "$data_home/applications/warpium.desktop"
    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"; rm -f "$data_home/icons/hicolor/512x512/apps/warpium.png"
    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"; command -v update-desktop-database >/dev/null && update-desktop-database "$data_home/applications" || true
    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"; command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q "$data_home/icons/hicolor" || true

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
