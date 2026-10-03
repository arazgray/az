#!/usr/bin/env sh
set -eu

BIN_DIR="${AZ_BIN_DIR:-$HOME/.local/bin}"
TARGET="$BIN_DIR/az"
STATE_DIR="${XDG_STATE_HOME:-$HOME/.local/state}/az-rust"
PROFILE="$HOME/.profile"
PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'

REMOVE_STATE=0
REMOVE_PATH=0

for arg in "$@"; do
  case "$arg" in
    --remove-state) REMOVE_STATE=1 ;;
    --remove-path) REMOVE_PATH=1 ;;
    -h|--help)
      echo "Usage: ./uninstall.sh [--remove-state] [--remove-path]"
      echo ""
      echo "Removes the installed 'az' binary (default: \$HOME/.local/bin/az,"
      echo "override with AZ_BIN_DIR). By default keeps your sessions/recovery"
      echo "files and your shell PATH untouched."
      echo ""
      echo "Options:"
      echo "  --remove-state  Also delete the state dir (sessions + recovery"
      echo "                  files, default: \$HOME/.local/state/az-rust,"
      echo "                  override with XDG_STATE_HOME)."
      echo "  --remove-path   Also remove the PATH line ./build.sh added to"
      echo "                  ~/.profile (exact line match only)."
      exit 0
      ;;
    *)
      echo "[az uninstall] Error: unknown option '$arg'. Use --help." >&2
      exit 1
      ;;
  esac
done

printf '[az uninstall] Uninstalling az (BIN_DIR=%s)...\n' "$BIN_DIR"

if [ -f "$TARGET" ] || [ -L "$TARGET" ]; then
  rm -f "$TARGET"
  printf '[az uninstall] Removed %s.\n' "$TARGET"
else
  printf '[az uninstall] %s not found, nothing to remove.\n' "$TARGET"
fi

SYS_TARGET="/usr/local/bin/az"
if [ -f "$SYS_TARGET" ] || [ -L "$SYS_TARGET" ]; then
  if [ -w "$SYS_TARGET" ] || [ -w /usr/local/bin ]; then
    rm -f "$SYS_TARGET"
    printf '[az uninstall] Removed %s.\n' "$SYS_TARGET"
  elif command -v sudo >/dev/null 2>&1 && sudo rm -f "$SYS_TARGET"; then
    printf '[az uninstall] Removed %s.\n' "$SYS_TARGET"
  else
    printf '[az uninstall] Could not remove %s (try: sudo rm -f %s).\n' "$SYS_TARGET" "$SYS_TARGET"
  fi
fi

if [ "$REMOVE_STATE" -eq 1 ]; then
  if [ -d "$STATE_DIR" ]; then
    rm -rf "$STATE_DIR"
    printf '[az uninstall] Removed state dir %s.\n' "$STATE_DIR"
  else
    printf '[az uninstall] State dir %s not found, nothing to remove.\n' "$STATE_DIR"
  fi
else
  printf '[az uninstall] Keeping state dir %s (sessions/recovery). Use --remove-state to delete it.\n' "$STATE_DIR"
fi

if [ "$REMOVE_PATH" -eq 1 ]; then
  if [ -f "$PROFILE" ] && grep -F "$PATH_LINE" "$PROFILE" >/dev/null 2>&1; then
    tmp="$(mktemp)"
    # '|| true': grep exits 1 when every line is filtered out.
    grep -v -F "$PATH_LINE" "$PROFILE" > "$tmp" || true
    cat "$tmp" > "$PROFILE"
    rm -f "$tmp"
    printf '[az uninstall] Removed PATH line from %s.\n' "$PROFILE"
  else
    printf '[az uninstall] No build.sh PATH line in %s, nothing to remove.\n' "$PROFILE"
  fi
fi

printf '[az uninstall] Done.\n'
