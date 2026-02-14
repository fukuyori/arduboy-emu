#!/usr/bin/env bash
# ============================================================
#  Arduboy Emulator — Linux Package Builder
#  Builds .deb (Debian/Ubuntu) and .rpm (Fedora/RHEL) packages
#
#  Prerequisites:
#    - Rust toolchain (rustup)
#    - dpkg-deb (for .deb — pre-installed on Debian/Ubuntu)
#    - rpmbuild (for .rpm — install: sudo dnf install rpm-build)
#
#  Usage:  ./build-linux.sh [--deb] [--rpm] [--all]
#  Output: dist/linux/arduboy-emu_0.7.3_amd64.deb
#          dist/linux/arduboy-emu-0.7.3-1.x86_64.rpm
# ============================================================

set -euo pipefail

VERSION="0.7.3"
ARCH="amd64"
RPM_ARCH="x86_64"
APP_NAME="arduboy-emu"
BIN_NAME="arduboy-frontend"
DESCRIPTION="Cycle-accurate Arduboy/Gamebuino emulator with debug tools"
MAINTAINER="arduboy-emu contributors"
URL="https://github.com/example/arduboy-emu"
LICENSE="MIT"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist/linux"

BUILD_DEB=false
BUILD_RPM=false

# Parse arguments
if [ $# -eq 0 ]; then
    BUILD_DEB=true
    BUILD_RPM=true
else
    for arg in "$@"; do
        case "$arg" in
            --deb) BUILD_DEB=true ;;
            --rpm) BUILD_RPM=true ;;
            --all) BUILD_DEB=true; BUILD_RPM=true ;;
            --help|-h)
                echo "Usage: $0 [--deb] [--rpm] [--all]"
                echo "  --deb   Build .deb package (Debian/Ubuntu)"
                echo "  --rpm   Build .rpm package (Fedora/RHEL/openSUSE)"
                echo "  --all   Build both (default if no args)"
                exit 0
                ;;
            *) echo "Unknown option: $arg"; exit 1 ;;
        esac
    done
fi

echo "==================================="
echo " Arduboy Emulator v${VERSION}"
echo " Linux Package Builder"
echo "==================================="
echo ""

# --- Step 1: Build release binary ---
echo "[1/3] Building release binary..."
cd "$PROJECT_ROOT"
cargo build --release -p arduboy-frontend

BINARY="$PROJECT_ROOT/target/release/$BIN_NAME"
if [ ! -f "$BINARY" ]; then
    echo "ERROR: $BIN_NAME not found in target/release/"
    exit 1
fi

# Strip debug symbols for smaller package
strip "$BINARY" 2>/dev/null || true
BINARY_SIZE=$(stat -c%s "$BINARY" 2>/dev/null || stat -f%z "$BINARY")
echo "     Binary: target/release/$BIN_NAME ($BINARY_SIZE bytes)"
echo ""

mkdir -p "$DIST_DIR"

# --- Step 2: Build .deb ---
if $BUILD_DEB; then
    echo "[2/3] Building .deb package..."

    if ! command -v dpkg-deb &>/dev/null; then
        echo "WARNING: dpkg-deb not found. Skipping .deb build."
        echo "         Install: sudo apt install dpkg"
    else
        DEB_NAME="${APP_NAME}_${VERSION}_${ARCH}"
        DEB_ROOT="$DIST_DIR/deb-staging/$DEB_NAME"
        rm -rf "$DEB_ROOT"

        # Directory structure
        mkdir -p "$DEB_ROOT/DEBIAN"
        mkdir -p "$DEB_ROOT/usr/bin"
        mkdir -p "$DEB_ROOT/usr/share/applications"
        mkdir -p "$DEB_ROOT/usr/share/doc/$APP_NAME"
        mkdir -p "$DEB_ROOT/usr/share/metainfo"

        # Binary
        cp "$BINARY" "$DEB_ROOT/usr/bin/$APP_NAME"
        chmod 755 "$DEB_ROOT/usr/bin/$APP_NAME"

        # Calculate installed size (in KB)
        INSTALLED_SIZE=$(du -sk "$DEB_ROOT" | cut -f1)

        # Control file
        cat > "$DEB_ROOT/DEBIAN/control" <<CTRL
Package: ${APP_NAME}
Version: ${VERSION}
Section: games
Priority: optional
Architecture: ${ARCH}
Depends: libc6 (>= 2.31), libasound2 | libasound2t64, libx11-6, libxcursor1, libxrandr2, libxi6, libudev1
Installed-Size: ${INSTALLED_SIZE}
Maintainer: ${MAINTAINER}
Homepage: ${URL}
Description: ${DESCRIPTION}
 A cycle-accurate emulator for the Arduboy handheld game console and
 Gamebuino Classic. Features ATmega32u4/328P CPU cores, SSD1306/PCD8544
 displays, stereo audio with DSP pipeline, FX flash, GDB server,
 execution profiler, rewind, LCD effect, and portrait rotation.
CTRL

        # Desktop entry
        cat > "$DEB_ROOT/usr/share/applications/$APP_NAME.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Arduboy Emulator
GenericName=Game Console Emulator
Comment=${DESCRIPTION}
Exec=${APP_NAME} %f
Terminal=false
Categories=Game;Emulator;
MimeType=application/x-arduboy;
Keywords=arduboy;gamebuino;avr;emulator;retro;
StartupNotify=true
DESKTOP

        # AppStream metainfo
        cat > "$DEB_ROOT/usr/share/metainfo/$APP_NAME.metainfo.xml" <<META
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>${APP_NAME}</id>
  <name>Arduboy Emulator</name>
  <summary>${DESCRIPTION}</summary>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT AND Apache-2.0</project_license>
  <url type="homepage">${URL}</url>
  <provides>
    <binary>${APP_NAME}</binary>
  </provides>
  <releases>
    <release version="${VERSION}" date="2025-02-14"/>
  </releases>
</component>
META

        # MIME type for .arduboy files
        mkdir -p "$DEB_ROOT/usr/share/mime/packages"
        cat > "$DEB_ROOT/usr/share/mime/packages/$APP_NAME.xml" <<MIME
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-arduboy">
    <comment>Arduboy game archive</comment>
    <glob pattern="*.arduboy"/>
  </mime-type>
</mime-info>
MIME

        # Documentation
        cp "$PROJECT_ROOT/README.md" "$DEB_ROOT/usr/share/doc/$APP_NAME/"
        cp "$PROJECT_ROOT/CHANGELOG.md" "$DEB_ROOT/usr/share/doc/$APP_NAME/"
        cp "$PROJECT_ROOT/LICENSE-MIT" "$DEB_ROOT/usr/share/doc/$APP_NAME/copyright"

        # Post-install script (update MIME database)
        cat > "$DEB_ROOT/DEBIAN/postinst" <<'POSTINST'
#!/bin/sh
set -e
if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database /usr/share/mime || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi
POSTINST
        chmod 755 "$DEB_ROOT/DEBIAN/postinst"

        # Build .deb
        dpkg-deb --root-owner-group --build "$DEB_ROOT" "$DIST_DIR/$DEB_NAME.deb"
        echo "     Output: dist/linux/$DEB_NAME.deb"
        echo ""

        # Cleanup staging
        rm -rf "$DIST_DIR/deb-staging"
    fi
else
    echo "[2/3] Skipping .deb (not requested)"
fi

# --- Step 3: Build .rpm ---
if $BUILD_RPM; then
    echo "[3/3] Building .rpm package..."

    if ! command -v rpmbuild &>/dev/null; then
        echo "WARNING: rpmbuild not found. Skipping .rpm build."
        echo "         Install: sudo dnf install rpm-build  (Fedora)"
        echo "                  sudo zypper install rpm-build (openSUSE)"
    else
        RPM_TOPDIR="$DIST_DIR/rpm-staging"
        rm -rf "$RPM_TOPDIR"
        mkdir -p "$RPM_TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

        # Create tarball for rpmbuild
        TARBALL_DIR="${APP_NAME}-${VERSION}"
        TARBALL_STAGING="$RPM_TOPDIR/SOURCES"
        mkdir -p "$TARBALL_STAGING/$TARBALL_DIR"
        cp "$BINARY" "$TARBALL_STAGING/$TARBALL_DIR/$APP_NAME"
        cp "$PROJECT_ROOT/README.md" "$TARBALL_STAGING/$TARBALL_DIR/"
        cp "$PROJECT_ROOT/CHANGELOG.md" "$TARBALL_STAGING/$TARBALL_DIR/"
        cp "$PROJECT_ROOT/LICENSE-MIT" "$TARBALL_STAGING/$TARBALL_DIR/"
        cp "$PROJECT_ROOT/LICENSE-APACHE" "$TARBALL_STAGING/$TARBALL_DIR/"

        # Desktop file
        cat > "$TARBALL_STAGING/$TARBALL_DIR/$APP_NAME.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Arduboy Emulator
GenericName=Game Console Emulator
Comment=${DESCRIPTION}
Exec=${APP_NAME} %f
Terminal=false
Categories=Game;Emulator;
MimeType=application/x-arduboy;
Keywords=arduboy;gamebuino;avr;emulator;retro;
StartupNotify=true
DESKTOP

        cd "$TARBALL_STAGING"
        tar czf "${APP_NAME}-${VERSION}.tar.gz" "$TARBALL_DIR"
        rm -rf "$TARBALL_DIR"

        # RPM spec file
        cat > "$RPM_TOPDIR/SPECS/$APP_NAME.spec" <<SPEC
# Disable debuginfo subpackage (binary is pre-stripped)
%define debug_package %{nil}

Name:           ${APP_NAME}
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        ${DESCRIPTION}
License:        MIT AND Apache-2.0
URL:            ${URL}
Source0:        %{name}-%{version}.tar.gz
ExclusiveArch:  x86_64

Requires:       alsa-lib
Requires:       libX11
Requires:       libXcursor
Requires:       libXrandr
Requires:       libXi
Requires:       systemd-libs

%description
A cycle-accurate emulator for the Arduboy handheld game console and
Gamebuino Classic. Features ATmega32u4/328P CPU cores, SSD1306/PCD8544
displays, stereo audio with DSP pipeline, FX flash, GDB server,
execution profiler, rewind, LCD effect, and portrait rotation.

%prep
%setup -q

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications

install -m 755 %{name} %{buildroot}%{_bindir}/%{name}
install -m 644 %{name}.desktop %{buildroot}%{_datadir}/applications/%{name}.desktop

%files
%license LICENSE-MIT LICENSE-APACHE
%doc README.md CHANGELOG.md
%{_bindir}/%{name}
%{_datadir}/applications/%{name}.desktop

%changelog
* Fri Feb 14 2025 arduboy-emu contributors - 0.7.3-1
- PWM DAC audio for Gamebuino Classic
- Portrait rotation (V key)
- Timer2 prescaler table fix
SPEC

        # Build RPM
        rpmbuild -bb \
            --define "_topdir $RPM_TOPDIR" \
            "$RPM_TOPDIR/SPECS/$APP_NAME.spec"

        # Copy output
        find "$RPM_TOPDIR/RPMS" -name "*.rpm" -exec cp {} "$DIST_DIR/" \;
        RPM_FILE=$(ls "$DIST_DIR"/*.rpm 2>/dev/null | head -1)
        if [ -n "$RPM_FILE" ]; then
            echo "     Output: $RPM_FILE"
        fi
        echo ""

        # Cleanup staging
        rm -rf "$RPM_TOPDIR"
    fi
else
    echo "[3/3] Skipping .rpm (not requested)"
fi

echo ""
echo "==================================="
echo " Done. Packages in dist/linux/"
ls -lh "$DIST_DIR"/*.{deb,rpm} 2>/dev/null || echo " (no packages built)"
echo "==================================="
