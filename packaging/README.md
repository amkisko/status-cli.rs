# Packaging

Distribution artifacts and package descriptors for status-cli.

| Distribution | Path | Notes |
|-------------|------|--------|
| **Homebrew** | [homebrew/status-cli.rb](homebrew/status-cli.rb) | `make sync-packaging` updates the tag URL; fill `sha256` after the release tarball exists |
| **Nix** | [nix/](nix/) + repo root [../flake.nix](../flake.nix) | `nix build .#default` from repo root |
| **Flatpak** | [flatpak/io.github.amkisko.status-cli.yml](flatpak/io.github.amkisko.status-cli.yml) | May require Rust SDK; adjust base/SDK as needed |
| **Arch AUR** | [aur/PKGBUILD](aur/PKGBUILD) | Run `updpkgsums` after setting `pkgver`; submit to AUR |
| **FreeBSD** | [freebsd/](freebsd/) | Port template; or `cargo install --path status` |
| **Gentoo** | [gentoo/app-misc/status-cli/](gentoo/app-misc/status-cli/) | Ebuild template |

All packaging is best-effort; prefer `cargo install --path status` when in doubt.
