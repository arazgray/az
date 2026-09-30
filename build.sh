#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")"

BIN_DIR="${AZ_BIN_DIR:-$HOME/.local/bin}"
TARGET="$BIN_DIR/az"

add_bin_dir_to_path() {
  printf '[az build] Checking if %s is on PATH...\n' "$BIN_DIR"
  case ":$PATH:" in
    *":$BIN_DIR:"*) printf '[az build] %s is already on PATH.\n' "$BIN_DIR"; return 0 ;;
  esac

  PROFILE="$HOME/.profile"
  LINE='export PATH="$HOME/.local/bin:$PATH"'

  if [ "$BIN_DIR" = "$HOME/.local/bin" ]; then
    printf '[az build] Adding ~/.local/bin to PATH via ~/.profile...\n'
    touch "$PROFILE"
    if ! grep -F "$LINE" "$PROFILE" >/dev/null 2>&1; then
      printf '\n%s\n' "$LINE" >> "$PROFILE"
      echo "[az build] Added ~/.local/bin to PATH in ~/.profile."
      echo "[az build] Restart your terminal or run: . ~/.profile"
    else
      echo "[az build] ~/.local/bin already present in ~/.profile."
    fi
  else
    echo "[az build] Note: $BIN_DIR is not in PATH."
    echo "[az build] Add this to your shell profile: export PATH=\"$BIN_DIR:\$PATH\""
  fi
}

ensure_rust() {
  printf '[az build] Step 1/4: Checking for Rust toolchain...\n'
  if command -v cargo >/dev/null 2>&1; then
    printf '[az build] Found cargo: %s\n' "$(command -v cargo)"
    cargo --version
    return 0
  fi
  if command -v rustc >/dev/null 2>&1; then
    printf '[az build] Found rustc (no cargo): %s\n' "$(command -v rustc)"
    rustc --version
    return 0
  fi

  printf '[az build] Rust not found. Installing Rust via rustup...\n'
  if ! command -v curl >/dev/null 2>&1; then
    echo "[az build] Error: curl is required to install Rust automatically." >&2
    echo "[az build] Install curl, then install Rust manually:" >&2
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
    exit 1
  fi

  printf '[az build] Downloading and running https://sh.rustup.rs (non-interactive: sh -s -- -y)...\n'
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

  # rustup installs to $HOME/.cargo/bin by default
  export PATH="$HOME/.cargo/bin:$PATH"
  # shellcheck disable=SC1091
  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
    printf '[az build] Sourced $HOME/.cargo/env.\n'
  fi

  if command -v cargo >/dev/null 2>&1; then
    printf '[az build] Rust installed successfully: %s\n' "$(command -v cargo)"
    cargo --version
    rustc --version
    return 0
  fi

  echo "[az build] Error: Rust installation finished but 'cargo' is still not on PATH." >&2
  echo "[az build] Try restarting your shell or run: export PATH=\"\$HOME/.cargo/bin:\$PATH\"" >&2
  exit 1
}

ensure_rust

printf '[az build] Step 2/4: Compiling az...\n'
if command -v cargo >/dev/null 2>&1; then
  printf '[az build] Running: cargo build --release\n'
  cargo build --release
  printf '[az build] Copying target/release/az to ./az...\n'
  cp target/release/az ./az
elif command -v rustc >/dev/null 2>&1; then
  printf '[az build] cargo not found, falling back to single-file rustc build.\n'
  printf '[az build] Running: rustc --edition=2021 -O src/main.rs -o ./az\n'
  rustc --edition=2021 -O src/main.rs -o ./az
else
  echo "[az build] Rust is not installed. Install Rust, then run ./build.sh" >&2
  exit 1
fi

printf '[az build] Step 3/4: Installing binary...\n'

chmod +x ./az
printf '[az build] Built ./az\n'
mkdir -p "$BIN_DIR"
printf '[az build] Copying ./az to %s...\n' "$TARGET"
cp ./az "$TARGET"
chmod +x "$TARGET"
printf '[az build] Step 4/4: Ensuring %s is on PATH...\n' "$BIN_DIR"
add_bin_dir_to_path

printf '[az build] Done.\n'
printf '[az build] Installed %s\n' "$TARGET"
printf '[az build] Run it with: az\n'
