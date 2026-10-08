#!/usr/bin/env sh
# az — multi-OS packager.
# Builds Linux .deb (amd64/arm64), generic .tar.gz (any Linux distro:
# Arch, Fedora, openSUSE, Alpine, NixOS...), and stages macOS/Windows
# artifacts produced by CI (.github/workflows/release.yml).
#
# Usage:
#   ./dist/package.sh                 # package host arch (deb + tar.gz)
#   ./dist/package.sh --arch arm64    # cross-package when the foreign
#                                     # binary already exists at
#                                     # dist/az-<ver>-linux-<arm64|amd64>
#   ./dist/package.sh --all           # package every binary present in dist/
#   VERSION=4.2.0 ./dist/package.sh
set -eu

cd "$(dirname "$0")/.."

VERSION="${VERSION:-$(sed -n 's/^version *= *"\([^"]*\)".*/\1/p' Cargo.toml | head -n1)}"
ARCH_ARG=""
DO_ALL=0
for a in "$@"; do
  case "$a" in
    --arch) shift2=1 ;;
    --arch=*) ARCH_ARG="${a#--arch=}" ;;
    --all) DO_ALL=1 ;;
    -h|--help)
      echo "Usage: ./dist/package.sh [--arch amd64|arm64] [--all]"
      echo "  Default packages the host arch. --all packages every"
      echo "  dist/az-<ver>-linux-* binary present."
      exit 0
      ;;
    *)
      if [ "${shift2:-0}" = "1" ]; then ARCH_ARG="$a"; shift2=0; fi
      ;;
  esac
done

host_arch() {
  case "$(uname -m)" in
    x86_64|amd64) echo "amd64" ;;
    aarch64|arm64) echo "arm64" ;;
    *) echo "$(uname -m)" ;;
  esac
}

DEB_ARCH() {
  case "$1" in
    amd64) echo "amd64" ;;
    arm64) echo "arm64" ;;
    *) echo "$1" ;;
  esac
}

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[az package] Error: '$1' is required but not installed." >&2
    exit 1
  fi
}

make_tarball() {
  # $1 = rust target triple suffix for logs, $2 = arch label (amd64/arm64),
  # $3 = os label (linux/darwin), $4 = path to binary
  label="$2"; os="$3"; bin="$4"
  need tar
  if [ "$os" = "darwin" ]; then
    out="dist/az-${VERSION}-macos-${label}.tar.gz"
  elif [ "$os" = "windows" ]; then
    return 0
  else
    out="dist/az-${VERSION}-linux-${label}.tar.gz"
  fi
  stage="$(mktemp -d)"
  cp "$bin" "$stage/az"
  chmod +x "$stage/az"
  [ -f logo.png ] && cp logo.png "$stage/" 2>/dev/null || true
  [ -f README.md ] && cp README.md "$stage/" 2>/dev/null || true
  [ -f USER_MANUAL.md ] && cp USER_MANUAL.md "$stage/" 2>/dev/null || true
  if [ -f dist/deb/az_${VERSION}_amd64/usr/share/applications/az.desktop ]; then
    cp dist/deb/az_${VERSION}_amd64/usr/share/applications/az.desktop "$stage/az.desktop"
  fi
  tar -czf "$out" -C "$stage" az logo.png README.md USER_MANUAL.md az.desktop 2>/dev/null \
    || tar -czf "$out" -C "$stage" az
  rm -rf "$stage"
  printf '[az package] Wrote %s\n' "$out"
}

make_deb() {
  # $1 = arch label (amd64/arm64), $2 = path to binary
  arch="$1"; bin="$2"
  need dpkg-deb
  deb_arch="$(DEB_ARCH "$arch")"
  work="dist/deb/az_${VERSION}_${deb_arch}"
  rm -rf "$work"
  mkdir -p "$work/DEBIAN" "$work/usr/bin" "$work/usr/share/applications" \
    "$work/usr/share/doc/az" \
    "$work/usr/share/icons/hicolor/48x48/apps" \
    "$work/usr/share/icons/hicolor/64x64/apps" \
    "$work/usr/share/icons/hicolor/128x128/apps" \
    "$work/usr/share/icons/hicolor/256x256/apps"
  cp "$bin" "$work/usr/bin/az"
  chmod 755 "$work/usr/bin/az"
  # Icons: reuse logo.png for every size (source is 1:1).
  for s in 48x48 64x64 128x128 256x256; do
    if [ -f logo.png ]; then cp logo.png "$work/usr/share/icons/hicolor/$s/apps/az.png"; fi
  done
  if [ -f "dist/deb/az_${VERSION}_${deb_arch}/usr/share/applications/az.desktop" ]; then
    cp "dist/deb/az_${VERSION}_${deb_arch}/usr/share/applications/az.desktop" "$work/usr/share/applications/az.desktop"
  else
    cat > "$work/usr/share/applications/az.desktop" <<EOF
[Desktop Entry]
Name=az
Comment=A fast, small & sane text editor
Exec=az
Icon=az
Terminal=true
Type=Application
Categories=Development;TextEditor;
Keywords=editor;text;code;
EOF
  fi
  if [ -f CHANGELOG.md ]; then cp CHANGELOG.md "$work/usr/share/doc/az/changelog"; fi
  cat > "$work/DEBIAN/control" <<EOF
Package: az
Version: ${VERSION}
Section: editors
Priority: optional
Architecture: ${deb_arch}
Maintainer: Araz Gray
Description: A fast, small & sane text editor
 Terminal text editor written in Rust with zero dependencies.
 Supports word wrap (Alt+Z) and RTL/LTR direction (Alt+R).
Homepage: https://github.com/arazgray/az
EOF
  cat > "$work/DEBIAN/postinst" <<EOF
#!/bin/sh
set -e
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database >/dev/null 2>&1 || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t /usr/share/icons/hicolor >/dev/null 2>&1 || true
exit 0
EOF
  chmod 755 "$work/DEBIAN/postinst"
  dpkg-deb --build "$work" "dist/az_${VERSION}_${deb_arch}.deb"
  printf '[az package] Wrote dist/az_%s_%s.deb\n' "$VERSION" "$deb_arch"
}

make_rpm() {
  # $1 = arch label (amd64/arm64 -> x86_64/aarch64), $2 = path to binary
  arch="$1"; bin="$2"
  case "$arch" in
    amd64) rpm_arch="x86_64" ;;
    arm64) rpm_arch="aarch64" ;;
    *) rpm_arch="$arch" ;;
  esac
  if ! command -v rpmbuild >/dev/null 2>&1; then
    echo "[az package] rpmbuild not found; skipping .rpm for $rpm_arch (install 'rpm' or 'rpm-build' to enable)."
    return 0
  fi
  top="$(mktemp -d)"
  mkdir -p "$top/BUILD" "$top/RPMS" "$top/SOURCES" "$top/SPECS" "$top/SRPMS" "$top/BUILDROOT"
  # Stage inputs under SOURCES. Do NOT pre-fill BUILDROOT: rpmbuild wipes
  # %{buildroot} before %install, so anything placed there beforehand is
  # deleted and the install step finds an empty directory (this broke the
  # 4.0 CI builds with `cp .../*: No such file or directory`).
  cp "$bin" "$top/SOURCES/az"
  [ -f logo.png ] && cp logo.png "$top/SOURCES/az.png" || true
  deb_arch="$(DEB_ARCH "$arch")"
  if [ -f "dist/deb/az_${VERSION}_${deb_arch}/usr/share/applications/az.desktop" ]; then
    cp "dist/deb/az_${VERSION}_${deb_arch}/usr/share/applications/az.desktop" \
      "$top/SOURCES/az.desktop"
  else
    cat > "$top/SOURCES/az.desktop" <<EOF
[Desktop Entry]
Name=az
Comment=A fast, small & sane text editor
Exec=az
Icon=az
Terminal=true
Type=Application
Categories=Development;TextEditor;
Keywords=editor;text;code;
EOF
  fi
  have_icon=0
  [ -f "$top/SOURCES/az.png" ] && have_icon=1
  files_icon=""
  if [ "$have_icon" = "1" ]; then
    files_icon="/usr/share/icons/hicolor/256x256/apps/az.png"
  fi
  cat > "$top/SPECS/az.spec" <<EOF
Name:           az
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        A fast, small & sane text editor
License:        WTFPL
URL:            https://github.com/arazgray/az
BuildArch:      ${rpm_arch}
Requires(post): desktop-file-utils
Requires(post): hicolor-icon-theme

%description
Terminal text editor written in Rust with zero dependencies.
Supports word wrap (Alt+Z) and RTL/LTR direction (Alt+R).

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/bin %{buildroot}/usr/share/applications
install -m755 %{_topdir}/SOURCES/az %{buildroot}/usr/bin/az
install -m644 %{_topdir}/SOURCES/az.desktop %{buildroot}/usr/share/applications/az.desktop
if [ -f %{_topdir}/SOURCES/az.png ]; then
  mkdir -p %{buildroot}/usr/share/icons/hicolor/256x256/apps
  install -m644 %{_topdir}/SOURCES/az.png %{buildroot}/usr/share/icons/hicolor/256x256/apps/az.png
fi

%files
/usr/bin/az
/usr/share/applications/az.desktop
$files_icon

%post
update-desktop-database >/dev/null 2>&1 || true
touch --no-create /usr/share/icons/hicolor >/dev/null 2>&1 || true
gtk-update-icon-cache -f -t /usr/share/icons/hicolor >/dev/null 2>&1 || true
exit 0
EOF
  rpmbuild --define "_topdir $top" -bb "$top/SPECS/az.spec" >/dev/null
  rpm="$(find "$top/RPMS" -name '*.rpm' | head -n1)"
  if [ -n "$rpm" ]; then
    cp "$rpm" "dist/az-${VERSION}-1.${rpm_arch}.rpm"
    printf '[az package] Wrote dist/az-%s-1.%s.rpm\n' "$VERSION" "$rpm_arch"
  fi
  rm -rf "$top"
}

package_arch() {
  arch="$1"
  bin="dist/az-${VERSION}-linux-${arch}"
  if [ ! -f "$bin" ]; then
    # Host build: compile and stage it under the canonical name.
    if [ "$arch" = "$(host_arch)" ]; then
      echo "[az package] Building release binary for $arch..."
      cargo build --release
      cp target/release/az "$bin"
    else
      echo "[az package] Missing $bin; cross-compile it first, e.g.:"
      echo "  rustup target add aarch64-unknown-linux-gnu"
      echo "  cargo build --release --target aarch64-unknown-linux-gnu"
      echo "  cp target/aarch64-unknown-linux-gnu/release/az $bin"
      return 1
    fi
  else
    echo "[az package] Using existing $bin"
  fi
  make_tarball "" "$arch" "linux" "$bin"
  make_deb "$arch" "$bin"
  make_rpm "$arch" "$bin"
}

mkdir -p dist
if [ "$DO_ALL" = "1" ]; then
  found=0
  for b in dist/az-"${VERSION}"-linux-*; do
    [ -f "$b" ] || continue
    case "$b" in
      *.tar.gz|*.deb|*.rpm) continue ;;
    esac
    a="${b##*-linux-}"
    package_arch "$a" || true
    found=1
  done
  if [ "$found" = "0" ]; then
    package_arch "$(host_arch)"
  fi
elif [ -n "$ARCH_ARG" ]; then
  package_arch "$ARCH_ARG"
else
  package_arch "$(host_arch)"
fi

# Checksums for the release page.
cd dist
sha256sum az-"${VERSION}"-linux-*.tar.gz az_"${VERSION}"_*.deb 2>/dev/null > "SHA256SUMS-${VERSION}.txt" || true
sha256sum az-"${VERSION}"-macos-*.tar.gz 2>/dev/null >> "SHA256SUMS-${VERSION}.txt" || true
sha256sum az-"${VERSION}"-*.rpm 2>/dev/null >> "SHA256SUMS-${VERSION}.txt" || true
sha256sum az-"${VERSION}"-windows-*.exe 2>/dev/null >> "SHA256SUMS-${VERSION}.txt" || true
if [ -f "SHA256SUMS-${VERSION}.txt" ]; then
  echo "[az package] Wrote dist/SHA256SUMS-${VERSION}.txt"
  cat "SHA256SUMS-${VERSION}.txt"
fi
