# FreeBSD port for status-cli
# Before first build: run "make cargo-crates" in the port directory.

PORTNAME=	status-cli
DISTVERSIONPREFIX=	v
DISTVERSION=	0.1.0
CATEGORIES=	sysutils
MAINTAINER=	contact@kiskolabs.com
COMMENT=	Status page CLI - search catalog and check live status
WWW=		https://github.com/amkisko/status-cli.rs
LICENSE=	MIT
LICENSE_FILE=	${WRKSRC}/LICENSE.md

USES=		cargo
USE_GITHUB=	yes
GH_ACCOUNT=	amkisko
GH_PROJECT=	status-cli.rs

CARGO_INSTALL_PATH=	status

.include <bsd.port.mk>
