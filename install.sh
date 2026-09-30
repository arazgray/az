#!/usr/bin/env sh
set -eu

REPO_URL="${AZ_REPO_URL:-https://github.com/arazgholami/az.git}"
BIN_DIR="${AZ_BIN_DIR:-$HOME/.local/bin}"
TMP_DIR=""

cleanup() {
  if [ -n "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT INT TERM

run_from_source_tree() {
  printf '[az install] Building from source tree (BIN_DIR=%s)...\n' "$BIN_DIR"
  AZ_BIN_DIR="$BIN_DIR" ./build.sh
}

printf '[az install] Starting az install (BIN_DIR=%s)...\n' "$BIN_DIR"

if [ -f ./Cargo.toml ] && [ -f ./src/main.rs ] && [ -x ./build.sh ]; then
  printf '[az install] Detected local source checkout, skipping git clone.\n'
  run_from_source_tree
  printf '[az install] Install complete.\n'
  exit 0
fi

printf '[az install] Step 1/3: Checking for git...\n'
if ! command -v git >/dev/null 2>&1; then
  echo "[az install] Error: git is required for remote install."
  echo "[az install] Install git, or download the source zip and run ./build.sh inside it."
  exit 1
fi
printf '[az install] Found git: %s\n' "$(command -v git)"

TMP_DIR="$(mktemp -d)"
printf '[az install] Step 2/3: Cloning %s into %s...\n' "$REPO_URL" "$TMP_DIR/az"
git clone --depth 1 "$REPO_URL" "$TMP_DIR/az" >/dev/null 2>&1
printf '[az install] Clone complete.\n'
cd "$TMP_DIR/az"
printf '[az install] Step 3/3: Building and installing...\n'
run_from_source_tree
printf '[az install] Install complete.\n'
