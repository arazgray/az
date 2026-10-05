#!/usr/bin/env sh
set -eu

REPO_URL="${AZ_REPO_URL:-https://github.com/arazgray/az.git}"
BIN_DIR="${AZ_BIN_DIR:-$HOME/.local/bin}"
TMP_DIR=""

# Tokyo Night, same blues as the welcome dialog. Off when stdout is not a
# terminal, TERM=dumb, or NO_COLOR is set (https://no-color.org/).
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ] && [ "${TERM:-}" != "dumb" ]; then
  RESET=$(printf '\033[0m')
  BOLD=$(printf '\033[1m')
  BLUE=$(printf '\033[38;2;122;162;247m')
  CYAN=$(printf '\033[38;2;125;207;255m')
  GREEN=$(printf '\033[38;2;158;206;106m')
  RED=$(printf '\033[38;2;247;118;142m')
  DIM=$(printf '\033[38;2;169;177;214m')
else
  RESET=''
  BOLD=''
  BLUE=''
  CYAN=''
  GREEN=''
  RED=''
  DIM=''
fi

info() { printf '%s[az compile-install]%s %s\n' "$CYAN" "$RESET" "$1"; }
ok() { printf '%s[az compile-install]%s %s%s%s\n' "$CYAN" "$RESET" "$GREEN" "$1" "$RESET"; }
err() { printf '%s[az compile-install]%s %s%s%s\n' "$CYAN" "$RESET" "$RED" "$1" "$RESET" >&2; }

# Same mark the welcome dialog paints from logo.txt: dark ink #578bfb,
# light ink and solid fills #7aa2f7, every other cell on #1f2335.
# Half-blocks need that background or the letters fall apart.
print_logo() {
  if [ -z "$BLUE" ]; then
    printf '%s\n' \
      '         ' \
      '█▀▀█ ▀▀▀█' \
      '▄▄▄▐ ▐▀▀▀' \
      '█▄▄▌ ▌▄▄█' \
      '   ▀     '
    return 0
  fi
  printf '  %b\n' \
    '\033[38;2;122;162;247m\033[48;2;31;35;53m         \033[0m' \
    '\033[38;2;122;162;247m\033[48;2;31;35;53m█▀▀█\033[38;2;122;162;247m\033[48;2;31;35;53m \033[38;2;122;162;247m\033[48;2;31;35;53m▀▀▀█\033[0m' \
    '\033[38;2;122;162;247m\033[48;2;31;35;53m▄▄▄\033[38;2;122;162;247m\033[48;2;122;162;247m▐\033[38;2;122;162;247m\033[48;2;31;35;53m \033[38;2;122;162;247m\033[48;2;122;162;247m▐\033[38;2;122;162;247m\033[48;2;31;35;53m▀▀▀\033[0m' \
    '\033[38;2;87;139;251m\033[48;2;31;35;53m█▄▄\033[38;2;87;139;251m\033[48;2;122;162;247m▌\033[38;2;122;162;247m\033[48;2;31;35;53m \033[38;2;87;139;251m\033[48;2;122;162;247m▌\033[38;2;87;139;251m\033[48;2;31;35;53m▄▄█\033[0m' \
    '\033[38;2;122;162;247m\033[48;2;31;35;53m   \033[38;2;87;139;251m\033[48;2;31;35;53m▀\033[38;2;122;162;247m\033[48;2;31;35;53m     \033[0m'
}

print_banner() {
  printf '\n'
  print_logo
  printf '\n'
  printf '  %s%sWelcome!%s\n' "$BOLD" "$BLUE" "$RESET"
  printf '  %saz is a fast, small & sane text editor.%s\n' "$DIM" "$RESET"
  printf '  %sThis script builds it from source and installs it%s\n' "$DIM" "$RESET"
  printf '  %s(%s/az, and /usr/local/bin/az when it can).%s\n' "$DIM" "$BIN_DIR" "$RESET"
  printf '  %sPrefer a prebuilt package? Use ./install.sh instead.%s\n' "$DIM" "$RESET"
  printf '\n'
}

cleanup() {
  if [ -n "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT INT TERM

run_from_source_tree() {
  info "Building from source tree (BIN_DIR=${BIN_DIR})..."
  AZ_BIN_DIR="$BIN_DIR" ./build.sh
}

print_banner

if [ -f ./Cargo.toml ] && [ -f ./src/main.rs ] && [ -x ./build.sh ]; then
  info "Detected local source checkout, skipping git clone."
  run_from_source_tree
  ok "Install complete."
  exit 0
fi

info "Step 1/3: Checking for git..."
if ! command -v git >/dev/null 2>&1; then
  err "Error: git is required for remote install."
  err "Install git, or download the source zip and run ./build.sh inside it."
  exit 1
fi
info "Found git: $(command -v git)"

TMP_DIR="$(mktemp -d)"
info "Step 2/3: Cloning ${REPO_URL} into ${TMP_DIR}/az..."
git clone --depth 1 "$REPO_URL" "$TMP_DIR/az" >/dev/null 2>&1
ok "Clone complete."
cd "$TMP_DIR/az"
info "Step 3/3: Building and installing..."
run_from_source_tree
ok "Install complete."
