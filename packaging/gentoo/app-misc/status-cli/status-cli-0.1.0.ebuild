# Gentoo ebuild for status-cli
# Copy to a local overlay under app-misc/status-cli/

EAPI=8

inherit cargo

DESCRIPTION="Status page CLI - search catalog and check live status"
HOMEPAGE="https://github.com/amkisko/status-cli.rs"
SRC_URI="https://github.com/amkisko/status-cli.rs/archive/refs/tags/v${PV}.tar.gz -> ${P}.tar.gz"
S="${WORKDIR}/status-cli.rs-${PV}"

LICENSE="MIT"
SLOT="0"
KEYWORDS="~amd64 ~arm64"

CARGO_INSTALL_PATH=status

src_install() {
	cargo_src_install
	einstalldocs
	dodoc LICENSE.md
}
