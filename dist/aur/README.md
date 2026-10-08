# Publishing `az-bin` to the AUR

`dist/aur/az-bin/` holds the package (`PKGBUILD` + `.SRCINFO`).
`az-bin` and `az` are both currently free on the AUR (checked 2026-10-06).

## First publish (needs an AUR account + SSH key)

```sh
# 1. Register at https://aur.archlinux.org/register, add your SSH pubkey.
# 2. Clone the (empty) AUR repo and push these files:
git clone ssh://aur@aur.archlinux.org/az-bin.git /tmp/opencode/aur-az-bin
cp dist/aur/az-bin/PKGBUILD dist/aur/az-bin/.SRCINFO /tmp/opencode/aur-az-bin/
cd /tmp/opencode/aur-az-bin
git add PKGBUILD .SRCINFO
git commit -m 'az-bin 4.3.0-1'
git push
```

Users then install with `yay -S az-bin` (or `paru -S az-bin`, or
`git clone https://aur.archlinux.org/az-bin.git && cd az-bin && makepkg -si`).

## Bumping to a new release

```sh
# Edit pkgver (+ reset pkgrel=1), refresh sums, regenerate .SRCINFO:
cd /tmp/opencode/aur-az-bin
updpkgsums          # needs pacman-contrib
makepkg --printsrcinfo > .SRCINFO
makepkg -si         # sanity: builds + installs locally
git commit -am 'az-bin <ver>-1' && git push
```

`updpkgsums` downloads the new release assets, so the sums always match
what GitHub serves — never copy sums out of `README.md` by hand.

## ARM64

The `PKGBUILD` has a commented `aarch64` stanza. Uncomment it (and drop the
comment) once a release ships `az-<ver>-linux-arm64`, fill
`sha256sums_aarch64` via `updpkgsums -a aarch64`, and add `arch = aarch64`
to `.SRCINFO`.
