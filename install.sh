#!/usr/bin/env sh
set -eu

# az 4.0.0 — prebuilt package installer.
# Checks the OS and installs the ready package instead of building:
#   Linux   -> dist/az_4.0.0_amd64.deb (icons from logo.png inside)
#   Windows -> dist/az-4.0.0-windows-amd64.exe (+ az-icon.ico / logo.png alongside)
# To build from source instead, run ./compile-and-install.sh.
AZ_VERSION="4.0.0"
REPO_RAW="${AZ_REPO_URL_RAW:-https://raw.githubusercontent.com/arazgray/az/refs/heads/main}"
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

info() { printf '%s[az install]%s %s\n' "$CYAN" "$RESET" "$1"; }
ok() { printf '%s[az install]%s %s%s%s\n' "$CYAN" "$RESET" "$GREEN" "$1" "$RESET"; }
err() { printf '%s[az install]%s %s%s%s\n' "$CYAN" "$RESET" "$RED" "$1" "$RESET" >&2; }

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
  printf '  %sThis script installs the prebuilt v%s package for your OS.%s\n' "$DIM" "$AZ_VERSION" "$RESET"
  printf '  %s(No compilation. To build instead: ./compile-and-install.sh.)%s\n' "$DIM" "$RESET"
  printf '\n'
}

cleanup() {
  if [ -n "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT INT TERM

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      echo "Usage: ./install.sh [--compile]"
      echo ""
      echo "Installs the prebuilt az v$AZ_VERSION package for your OS:"
      echo "  Linux   -> dist/az_4.0.0_amd64.deb (or the raw binary fallback)"
      echo "  Windows -> dist/az-4.0.0-windows-amd64.exe (+ az-icon.ico, logo.png)"
      echo ""
      echo "Options:"
      echo "  --compile   Build from source via ./compile-and-install.sh"
      echo ""
      echo "Env:"
      echo "  AZ_BIN_DIR  Install dir (default: \$HOME/.local/bin; on Windows: \$HOME/bin/az.exe)"
      exit 0
      ;;
    --compile)
      info "Delegating to ./compile-and-install.sh..."
      exec ./compile-and-install.sh
      ;;
  esac
done

os_name() {
  uname_s="$(uname -s 2>/dev/null || echo unknown)"
  case "$uname_s" in
    MINGW*|MSYS*|CYGWIN*|Windows_NT) echo "windows"; return 0 ;;
  esac
  if [ "${OS:-}" = "Windows_NT" ] || [ -n "${WINDIR:-}" ]; then
    echo "windows"; return 0
  fi
  case "$uname_s" in
    Linux*) echo "linux"; return 0 ;;
    Darwin*) echo "macos"; return 0 ;;
    *) echo "$uname_s" | tr '[:upper:]' '[:lower:]'; return 0 ;;
  esac
}

download() {
  url="$1"; dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --max-time 60 "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$dest" "$url"
  else
    return 1
  fi
}

# First local file that exists, else "".
first_existing() {
  for p in "$@"; do
    if [ -f "$p" ]; then printf '%s' "$p"; return 0; fi
  done
  printf ''
}

install_windows() {
  dest_dir="${AZ_BIN_DIR:-$HOME/bin}"
  case "$dest_dir" in
    *.exe) dest_exe="$dest_dir"; dest_dir="$(dirname "$dest_exe")" ;;
    *) dest_exe="$dest_dir/az.exe" ;;
  esac
  info "Detected Windows. Installing az.exe v$AZ_VERSION..."
  exe_src="$(first_existing ./dist/az-4.0.0-windows-amd64.exe ./az-4.0.0-windows-amd64.exe ./dist/az.exe ./az.exe)"
  ico_src="$(first_existing ./dist/az-icon.ico ./az-icon.ico)"
  png_src="$(first_existing ./dist/logo.png ./logo.png ./logo.png)"
  if [ -z "$exe_src" ]; then
    TMP_DIR="$(mktemp -d)"
    info "Downloading az-4.0.0-windows-amd64.exe..."
    if ! download "$REPO_RAW/dist/az-4.0.0-windows-amd64.exe" "$TMP_DIR/az.exe"; then
      err "Download failed. Clone the repo (which has dist/) or run ./compile-and-install.sh."
      exit 1
    fi
    exe_src="$TMP_DIR/az.exe"
    download "$REPO_RAW/dist/az-icon.ico" "$TMP_DIR/az-icon.ico" || true
    download "$REPO_RAW/logo.png" "$TMP_DIR/logo.png" || true
    [ -f "$TMP_DIR/az-icon.ico" ] && ico_src="$TMP_DIR/az-icon.ico"
    [ -f "$TMP_DIR/logo.png" ] && png_src="$TMP_DIR/logo.png"
  else
    info "Using local package: $exe_src"
  fi
  mkdir -p "$dest_dir"
  cp "$exe_src" "$dest_exe"
  chmod +x "$dest_exe" 2>/dev/null || true
  [ -n "$ico_src" ] && cp "$ico_src" "$dest_dir/az-icon.ico" 2>/dev/null || true
  [ -n "$png_src" ] && [ -f "$png_src" ] && cp "$png_src" "$dest_dir/logo.png" 2>/dev/null || true
  ok "Installed $dest_exe"
  info "Icons: $dest_dir/az-icon.ico + $dest_dir/logo.png (from logo.png)"
  info "Add $dest_dir to PATH if 'az' is not found, then run: az --version"
}

install_linux_binary_fallback() {
  bin_src="$1"
  info "Installing binary to $BIN_DIR/az (fallback, no dpkg)..."
  mkdir -p "$BIN_DIR"
  # The target may be a running executable (Text file busy): unlink first.
  rm -f "$BIN_DIR/az" 2>/dev/null || true
  cp "$bin_src" "$BIN_DIR/az"
  chmod +x "$BIN_DIR/az"
  if [ -w /usr/local/bin ] && [ ! -e /usr/local/bin/az ]; then
    cp "$bin_src" /usr/local/bin/az && chmod 755 /usr/local/bin/az && info "Also installed /usr/local/bin/az for sudo."
  elif command -v sudo >/dev/null 2>&1; then
    sudo cp "$bin_src" /usr/local/bin/az && sudo chmod 755 /usr/local/bin/az && info "Also installed /usr/local/bin/az for sudo." || true
  fi
  # Desktop entry + icon from logo.png for launchers.
  png_src="$(first_existing ./dist/logo.png ./logo.png "$TMP_DIR/logo.png")"
  if [ -n "$png_src" ]; then
    mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps" "$HOME/.local/share/applications"
    cp "$png_src" "$HOME/.local/share/icons/hicolor/256x256/apps/az.png" 2>/dev/null || true
    cat > "$HOME/.local/share/applications/az.desktop" <<EOF
[Desktop Entry]
Name=az
Comment=A fast, small & sane text editor
Exec=$BIN_DIR/az
Icon=az
Terminal=true
Type=Application
Categories=Development;TextEditor;
EOF
    command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
  fi
  ok "Installed $BIN_DIR/az"
}

install_linux() {
  info "Detected Linux. Installing az v$AZ_VERSION (.deb with logo.png icons)..."
  deb_src="$(first_existing ./dist/az_4.0.0_amd64.deb ./az_4.0.0_amd64.deb)"
  bin_src="$(first_existing ./dist/az-4.0.0-linux-amd64 ./az-4.0.0-linux-amd64)"
  if [ -z "$deb_src" ] && [ -z "$bin_src" ]; then
    TMP_DIR="$(mktemp -d)"
    info "No local dist/ package; downloading..."
    if download "$REPO_RAW/dist/az_4.0.0_amd64.deb" "$TMP_DIR/az.deb"; then
      deb_src="$TMP_DIR/az.deb"
    elif download "$REPO_RAW/dist/az-4.0.0-linux-amd64" "$TMP_DIR/az"; then
      bin_src="$TMP_DIR/az"
      download "$REPO_RAW/logo.png" "$TMP_DIR/logo.png" || true
    else
      err "Download failed. Clone the repo (which has dist/) or run ./compile-and-install.sh."
      exit 1
    fi
  elif [ -n "$deb_src" ]; then
    info "Using local package: $deb_src"
  else
    info "Using local binary: $bin_src"
  fi
  if [ -n "$deb_src" ] && command -v dpkg-deb >/dev/null 2>&1; then
    if [ "$(id -u)" -eq 0 ]; then
      dpkg -i "$deb_src" && ok "Installed $deb_src via dpkg." && exit 0
      err "dpkg failed, falling back to binary install."
    elif command -v sudo >/dev/null 2>&1; then
      if sudo dpkg -i "$deb_src"; then ok "Installed $deb_src via sudo dpkg."; exit 0; fi
      err "sudo dpkg failed, falling back to binary install."
    else
      info "No root/sudo; extracting binary to $BIN_DIR without system install."
      TMPX="$(mktemp -d)"
      dpkg-deb -x "$deb_src" "$TMPX"
      install_linux_binary_fallback "$TMPX/usr/bin/az"
      rm -rf "$TMPX"
      exit 0
    fi
  fi
  # Fallback: raw binary (deb missing or dpkg missing/failed).
  if [ -z "$bin_src" ] && [ -n "$deb_src" ] && command -v dpkg-deb >/dev/null 2>&1; then
    TMPX="$(mktemp -d)"
    dpkg-deb -x "$deb_src" "$TMPX"
    bin_src="$TMPX/usr/bin/az"
  fi
  if [ -n "$bin_src" ]; then
    install_linux_binary_fallback "$bin_src"
    exit 0
  fi
  err "No .deb tool and no binary available. Run ./compile-and-install.sh instead."
  exit 1
}

print_banner
OS="$(os_name)"
info "OS detected: $OS"
case "$OS" in
  windows) install_windows ;;
  linux) install_linux ;;
  macos|darwin)
    err "No prebuilt macOS package in dist/ (Linux .deb + Windows .exe only)."
    err "Building from source instead..."
    exec ./compile-and-install.sh
    ;;
  *)
    err "Unknown OS '$OS'. Trying Linux .deb flow, then source build."
    if ! install_linux; then
      exec ./compile-and-install.sh
    fi
    ;;
esac
ok "Install complete."
