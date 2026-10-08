#!/usr/bin/env sh
set -eu

# az 4.4.0 — prebuilt package installer.
# Detects OS + CPU arch and installs the ready package instead of building:
#   Debian/Ubuntu/Mint/Pop!_OS -> dist/az_<ver>_<debarch>.deb
#   Fedora/RHEL/openSUSE       -> dist/az-<ver>-1.<rpmarch>.rpm
#   Other Linux (Arch, Alpine, NixOS, ...) -> dist/az-<ver>-linux-<arch>.tar.gz
#   macOS (Intel + Apple Silicon) -> dist/az-<ver>-macos-<arch>.tar.gz
#   Windows (x86_64 + ARM64)   -> dist/az-<ver>-windows-<arch>.exe
# To build from source instead, run ./compile-and-install.sh.
AZ_VERSION="4.4.0"
REPO_RAW="${AZ_REPO_URL_RAW:-https://raw.githubusercontent.com/arazgray/az/refs/heads/main}"
# Release tag carrying this version's packages. Tags are usually short
# ("4.4.0" -> "4.4"); override with AZ_RELEASE_TAG when they are not.
RELEASE_TAG="${AZ_RELEASE_TAG:-${AZ_VERSION%.0}}"
RELEASE_BASE="https://github.com/arazgray/az/releases/download/${RELEASE_TAG}"
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
  printf '  %s%sThe TUI text editor you'"'"'ve always wanted%s\n' "$BOLD" "$BLUE" "$RESET"
  printf '  %sA ridiculously fast, lightweight & sane terminal text editor built in Rust. Batteries included.%s\n' "$DIM" "$RESET"
  printf '  %sKeyboard-first but mouse-supported, zero-configuration, and designed to stay out of your way%s\n' "$DIM" "$RESET"
  printf '  %sA perfect alternative to Vim and Nano%s\n' "$DIM" "$RESET"
  printf '  %sWith more than 150 language/syntax support.%s\n' "$DIM" "$RESET"
  printf '  %sVersion: v%s%s\n' "$DIM" "$AZ_VERSION" "$RESET"
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
      echo "Installs the prebuilt az v$AZ_VERSION package for your OS + CPU:"
      echo "  Debian/Ubuntu  -> dist/az_${AZ_VERSION}_<arch>.deb"
      echo "  Fedora/RHEL    -> dist/az-${AZ_VERSION}-1.<arch>.rpm"
      echo "  Other Linux    -> dist/az-${AZ_VERSION}-linux-<arch>.tar.gz"
      echo "  macOS          -> dist/az-${AZ_VERSION}-macos-<arch>.tar.gz"
      echo "  Windows        -> dist/az-${AZ_VERSION}-windows-<arch>.exe"
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
    FreeBSD*) echo "linux"; return 0 ;;
    *) echo "$uname_s" | tr '[:upper:]' '[:lower:]'; return 0 ;;
  esac
}

# Canonical CPU labels used by dist/ asset names: amd64 / arm64.
arch_name() {
  m="$(uname -m 2>/dev/null || echo unknown)"
  case "$m" in
    x86_64|amd64) echo "amd64" ;;
    aarch64|arm64) echo "arm64" ;;
    *) echo "$m" ;;
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

# Fetch a versioned dist/ artifact: the GitHub release first (source of
# truth published by CI), then raw main as fallback for older layouts.
fetch_dist() {
  if download "$RELEASE_BASE/$1" "$2"; then return 0; fi
  download "$REPO_RAW/dist/$1" "$2"
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
  ARCH="$(arch_name)"
  info "Detected Windows ($ARCH). Installing az.exe v$AZ_VERSION..."
  exe_src="$(first_existing \
    ./dist/az-${AZ_VERSION}-windows-${ARCH}.exe \
    ./az-${AZ_VERSION}-windows-${ARCH}.exe \
    ./dist/az-4.0.0-windows-amd64.exe ./az-4.0.0-windows-amd64.exe \
    ./dist/az.exe ./az.exe)"
  ico_src="$(first_existing ./dist/az-icon.ico ./az-icon.ico)"
  png_src="$(first_existing ./dist/logo.png ./logo.png ./logo.png)"
  if [ -z "$exe_src" ]; then
    TMP_DIR="$(mktemp -d)"
    info "Downloading az-${AZ_VERSION}-windows-${ARCH}.exe..."
    if ! fetch_dist "az-${AZ_VERSION}-windows-${ARCH}.exe" "$TMP_DIR/az.exe"; then
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

# Shared tarball extractor for macOS and generic-Linux installs.
# $1 = tarball path, $2 = install dir, $3 = os label (for messages)
install_tarball_to_bindir() {
  tarball="$1"; dest="$2"
  TMPX="$(mktemp -d)"
  if ! tar -xzf "$tarball" -C "$TMPX"; then
    err "Could not extract $tarball."
    rm -rf "$TMPX"
    exit 1
  fi
  # Tarball root holds ./az (see dist/package.sh); accept a nested dir too.
  bin_src="$(first_existing "$TMPX/az" "$TMPX/bin/az")"
  if [ -z "$bin_src" ]; then
    bin_src="$(find "$TMPX" -name az -type f 2>/dev/null | head -n1)"
  fi
  if [ -z "$bin_src" ]; then
    err "No 'az' binary inside $tarball."
    rm -rf "$TMPX"
    exit 1
  fi
  mkdir -p "$dest"
  rm -f "$dest/az" 2>/dev/null || true
  cp "$bin_src" "$dest/az"
  chmod +x "$dest/az"
  rm -rf "$TMPX"
}

install_macos() {
  ARCH="$(arch_name)"
  dest="${AZ_BIN_DIR:-$HOME/.local/bin}"
  info "Detected macOS ($ARCH). Installing az v$AZ_VERSION..."
  tar_src="$(first_existing \
    ./dist/az-${AZ_VERSION}-macos-${ARCH}.tar.gz \
    ./az-${AZ_VERSION}-macos-${ARCH}.tar.gz)"
  if [ -z "$tar_src" ]; then
    TMP_DIR="$(mktemp -d)"
    info "No local dist/ package; downloading az-${AZ_VERSION}-macos-${ARCH}.tar.gz..."
    if fetch_dist "az-${AZ_VERSION}-macos-${ARCH}.tar.gz" "$TMP_DIR/az.tar.gz"; then
      tar_src="$TMP_DIR/az.tar.gz"
    else
      err "Download failed. Building from source instead..."
      exec ./compile-and-install.sh
    fi
  else
    info "Using local package: $tar_src"
  fi
  install_tarball_to_bindir "$tar_src" "$dest" "macos"
  ok "Installed $dest/az"
  case ":$PATH:" in
    *":$dest:"*) ;;
    *) info "Add $dest to PATH: export PATH=\"$dest:\$PATH\" (or: brew --prefix + ln -s)" ;;
  esac
  info "Homebrew tap: brew install arazgray/tap/az"
  info "Run it with: $dest/az --version"
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
Comment=The TUI text editor you've always wanted
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
  ARCH="$(arch_name)"
  case "$ARCH" in
    amd64) DEB_ARCH="amd64"; RPM_ARCH="x86_64" ;;
    arm64) DEB_ARCH="arm64"; RPM_ARCH="aarch64" ;;
    *) DEB_ARCH="$ARCH"; RPM_ARCH="$ARCH" ;;
  esac
  info "Detected Linux ($ARCH). Installing az v$AZ_VERSION..."
  deb_src="$(first_existing \
    ./dist/az_${AZ_VERSION}_${DEB_ARCH}.deb \
    ./az_${AZ_VERSION}_${DEB_ARCH}.deb \
    ./dist/az_4.0.0_amd64.deb ./az_4.0.0_amd64.deb)"
  rpm_src="$(first_existing \
    ./dist/az-${AZ_VERSION}-1.${RPM_ARCH}.rpm \
    ./az-${AZ_VERSION}-1.${RPM_ARCH}.rpm)"
  tar_src="$(first_existing \
    ./dist/az-${AZ_VERSION}-linux-${ARCH}.tar.gz \
    ./az-${AZ_VERSION}-linux-${ARCH}.tar.gz)"
  bin_src="$(first_existing \
    ./dist/az-${AZ_VERSION}-linux-${ARCH} \
    ./az-${AZ_VERSION}-linux-${ARCH} \
    ./dist/az-4.0.0-linux-amd64 ./az-4.0.0-linux-amd64)"
  if [ -z "$deb_src" ] && [ -z "$rpm_src" ] && [ -z "$tar_src" ] && [ -z "$bin_src" ]; then
    TMP_DIR="$(mktemp -d)"
    info "No local dist/ package; downloading..."
    if [ -f /etc/debian_version ] || command -v dpkg >/dev/null 2>&1; then
      if fetch_dist "az_${AZ_VERSION}_${DEB_ARCH}.deb" "$TMP_DIR/az.deb"; then
        deb_src="$TMP_DIR/az.deb"
      fi
    fi
    if [ -z "$deb_src" ] && { [ -f /etc/redhat-release ] || [ -f /etc/SuSE-release ] || command -v rpm >/dev/null 2>&1; }; then
      if fetch_dist "az-${AZ_VERSION}-1.${RPM_ARCH}.rpm" "$TMP_DIR/az.rpm"; then
        rpm_src="$TMP_DIR/az.rpm"
      fi
    fi
    if [ -z "$deb_src" ] && [ -z "$rpm_src" ]; then
      if fetch_dist "az-${AZ_VERSION}-linux-${ARCH}.tar.gz" "$TMP_DIR/az.tar.gz"; then
        tar_src="$TMP_DIR/az.tar.gz"
      elif fetch_dist "az-${AZ_VERSION}-linux-${ARCH}" "$TMP_DIR/az"; then
        bin_src="$TMP_DIR/az"
        download "$REPO_RAW/logo.png" "$TMP_DIR/logo.png" || true
      else
        err "Download failed. Clone the repo (which has dist/) or run ./compile-and-install.sh."
        exit 1
      fi
    fi
  elif [ -n "$deb_src" ]; then
    info "Using local package: $deb_src"
  elif [ -n "$rpm_src" ]; then
    info "Using local package: $rpm_src"
  elif [ -n "$tar_src" ]; then
    info "Using local tarball: $tar_src"
  else
    info "Using local binary: $bin_src"
  fi
  # 1) Native .deb path (Debian/Ubuntu/Mint/Pop!_OS/...).
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
  # 2) Native .rpm path (Fedora/RHEL/CentOS/openSUSE/...).
  if [ -n "$rpm_src" ] && command -v rpm >/dev/null 2>&1; then
    if [ "$(id -u)" -eq 0 ]; then
      if rpm -i "$rpm_src"; then ok "Installed $rpm_src via rpm."; exit 0; fi
      err "rpm failed, falling back to binary install."
    elif command -v sudo >/dev/null 2>&1; then
      if sudo rpm -i "$rpm_src"; then ok "Installed $rpm_src via sudo rpm."; exit 0; fi
      if sudo dnf install -y "$rpm_src"; then ok "Installed $rpm_src via sudo dnf."; exit 0; fi
      if sudo zypper --non-interactive install "$rpm_src"; then ok "Installed via sudo zypper."; exit 0; fi
      err "sudo rpm/dnf/zypper failed, falling back to binary install."
    else
      info "No root/sudo; installing tarball-equivalent binary to $BIN_DIR."
    fi
  fi
  # 3) Generic tarball (Arch, Alpine, Gentoo, NixOS, any other distro).
  if [ -n "$tar_src" ]; then
    install_tarball_to_bindir "$tar_src" "$BIN_DIR" "linux"
    # Desktop entry + icon from the tarball (or repo logo.png).
    png_src="$(first_existing ./dist/logo.png ./logo.png "$TMP_DIR/logo.png")"
    if [ -n "$png_src" ]; then
      mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps" "$HOME/.local/share/applications"
      cp "$png_src" "$HOME/.local/share/icons/hicolor/256x256/apps/az.png" 2>/dev/null || true
      cat > "$HOME/.local/share/applications/az.desktop" <<EOF
[Desktop Entry]
Name=az
Comment=The TUI text editor you've always wanted
Exec=$BIN_DIR/az
Icon=az
Terminal=true
Type=Application
Categories=Development;TextEditor;
EOF
      command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
    fi
    if [ -w /usr/local/bin ] && [ ! -e /usr/local/bin/az ]; then
      cp "$BIN_DIR/az" /usr/local/bin/az && chmod 755 /usr/local/bin/az && info "Also installed /usr/local/bin/az for sudo."
    elif command -v sudo >/dev/null 2>&1; then
      sudo cp "$BIN_DIR/az" /usr/local/bin/az && sudo chmod 755 /usr/local/bin/az && info "Also installed /usr/local/bin/az for sudo." || true
    fi
    ok "Installed $BIN_DIR/az (from tarball)"
    exit 0
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
  err "No package tool and no binary available. Run ./compile-and-install.sh instead."
  exit 1
}

print_banner
OS="$(os_name)"
ARCH_DETECTED="$(arch_name)"
info "OS detected: $OS ($ARCH_DETECTED)"
case "$OS" in
  windows) install_windows ;;
  linux) install_linux ;;
  macos|darwin) install_macos ;;
  *)
    err "Unknown OS '$OS'. Trying Linux .deb flow, then source build."
    if ! install_linux; then
      exec ./compile-and-install.sh
    fi
    ;;
esac
ok "Install complete."
