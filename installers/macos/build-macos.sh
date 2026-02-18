#!/usr/bin/env bash
# ============================================================
#  Arduboy Emulator — macOS Installer Builder
#  Creates .app bundle + .pkg installer + .dmg disk image
#
#  Prerequisites:
#    - Rust toolchain (rustup)
#    - Xcode Command Line Tools (pkgbuild, productbuild, hdiutil)
#    - Optional: codesign identity for signing
#
#  Usage:  ./build-macos.sh [--sign "Developer ID"] [--universal]
#  Output: dist/macos/ArduboyEmulator-0.8.1.pkg
#          dist/macos/ArduboyEmulator-0.8.1.dmg
# ============================================================

set -euo pipefail

VERSION="0.8.1"
APP_NAME="Arduboy Emulator"
BUNDLE_ID="com.arduboy-emu.emulator"
BIN_NAME="arduboy-emu"
INSTALL_BIN_NAME="arduboy-emu"
MIN_MACOS="11.0"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist/macos"

SIGN_IDENTITY=""
BUILD_UNIVERSAL=false

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --sign)
            SIGN_IDENTITY="$2"
            shift 2
            ;;
        --universal)
            BUILD_UNIVERSAL=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--sign \"Developer ID Application: ...\"] [--universal]"
            echo "  --sign ID     Code-sign with given identity"
            echo "  --universal   Build universal binary (x86_64 + aarch64)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "==================================="
echo " ${APP_NAME} v${VERSION}"
echo " macOS Installer Builder"
echo "==================================="
echo ""

# --- Step 1: Build release binary ---
echo "[1/5] Building release binary..."
cd "$PROJECT_ROOT"

if $BUILD_UNIVERSAL; then
    echo "     Building universal binary (x86_64 + aarch64)..."

    # Ensure both targets are installed
    rustup target add x86_64-apple-darwin aarch64-apple-darwin 2>/dev/null || true

    cargo build --release -p arduboy-frontend --target x86_64-apple-darwin
    cargo build --release -p arduboy-frontend --target aarch64-apple-darwin

    BINARY="$DIST_DIR/$BIN_NAME"
    mkdir -p "$DIST_DIR"
    lipo -create \
        "target/x86_64-apple-darwin/release/$BIN_NAME" \
        "target/aarch64-apple-darwin/release/$BIN_NAME" \
        -output "$BINARY"

    echo "     Universal binary created"
else
    cargo build --release -p arduboy-frontend
    BINARY="$PROJECT_ROOT/target/release/$BIN_NAME"
fi

if [ ! -f "$BINARY" ]; then
    echo "ERROR: $BIN_NAME not found"
    exit 1
fi

strip "$BINARY" 2>/dev/null || true
echo "     Binary: $BINARY ($(wc -c < "$BINARY" | tr -d ' ') bytes)"
echo ""

mkdir -p "$DIST_DIR"

# --- Step 2: Create .app bundle ---
echo "[2/5] Creating .app bundle..."

APP_DIR="$DIST_DIR/${APP_NAME}.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

cp "$BINARY" "$APP_DIR/Contents/MacOS/$INSTALL_BIN_NAME"
chmod 755 "$APP_DIR/Contents/MacOS/$INSTALL_BIN_NAME"

# Info.plist
cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>${INSTALL_BIN_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>ARDB</string>
    <key>LSMinimumSystemVersion</key>
    <string>${MIN_MACOS}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Arduboy HEX ROM</string>
            <key>CFBundleTypeExtensions</key>
            <array><string>hex</string></array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
        </dict>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Arduboy Game Archive</string>
            <key>CFBundleTypeExtensions</key>
            <array><string>arduboy</string></array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Owner</string>
        </dict>
        <dict>
            <key>CFBundleTypeName</key>
            <string>ELF Binary</string>
            <key>CFBundleTypeExtensions</key>
            <array><string>elf</string></array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
        </dict>
    </array>
</dict>
</plist>
PLIST

# PkgInfo
echo -n "APPLARDB" > "$APP_DIR/Contents/PkgInfo"

# Copy icon if exists
if [ -f "$PROJECT_ROOT/assets/arduboy-emu.icns" ]; then
    cp "$PROJECT_ROOT/assets/arduboy-emu.icns" "$APP_DIR/Contents/Resources/AppIcon.icns"
    # Add icon reference to Info.plist
    /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string AppIcon" "$APP_DIR/Contents/Info.plist" 2>/dev/null || true
fi

echo "     Bundle: $APP_DIR"
echo ""

# --- Step 3: Code sign (optional) ---
if [ -n "$SIGN_IDENTITY" ]; then
    echo "[3/5] Code signing..."
    codesign --force --options runtime --sign "$SIGN_IDENTITY" \
        --entitlements /dev/stdin "$APP_DIR" <<ENTITLEMENTS
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.device.audio-input</key>
    <false/>
</dict>
</plist>
ENTITLEMENTS
    echo "     Signed with: $SIGN_IDENTITY"
    codesign --verify --verbose "$APP_DIR"
    echo ""
else
    echo "[3/5] Skipping code sign (use --sign to enable)"
    echo ""
fi

# --- Step 4: Build .pkg installer ---
echo "[4/5] Building .pkg installer..."

PKG_FILE="$DIST_DIR/ArduboyEmulator-${VERSION}.pkg"
COMPONENT_PKG="$DIST_DIR/_component.pkg"

# Build component package (installs .app to /Applications)
pkgbuild \
    --root "$APP_DIR" \
    --install-location "/Applications/${APP_NAME}.app" \
    --identifier "$BUNDLE_ID" \
    --version "$VERSION" \
    "$COMPONENT_PKG"

# Build product archive with welcome/license
# Distribution XML
cat > "$DIST_DIR/_distribution.xml" <<DISTXML
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>${APP_NAME} ${VERSION}</title>
    <welcome language="en" mime-type="text/plain"><![CDATA[
Welcome to the Arduboy Emulator installer.

This will install ${APP_NAME} v${VERSION} to your Applications folder.

Features:
• Cycle-accurate ATmega32u4 / ATmega328P CPU cores
• SSD1306 OLED and PCD8544 Nokia LCD displays
• Stereo audio with DSP pipeline (PWM DAC support)
• FX flash, GDB server, profiler, rewind
• LCD effect, portrait rotation, GIF recording
    ]]></welcome>
    <license file="LICENSE-MIT" mime-type="text/plain"/>
    <options customize="never" require-scripts="false"/>
    <choices-outline>
        <line choice="default"/>
    </choices-outline>
    <choice id="default" title="${APP_NAME}">
        <pkg-ref id="${BUNDLE_ID}"/>
    </choice>
    <pkg-ref id="${BUNDLE_ID}" version="${VERSION}" onConclusion="none">_component.pkg</pkg-ref>
</installer-gui-script>
DISTXML

# Copy license for the installer to reference
cp "$PROJECT_ROOT/LICENSE-MIT" "$DIST_DIR/LICENSE-MIT"

productbuild \
    --distribution "$DIST_DIR/_distribution.xml" \
    --resources "$DIST_DIR" \
    --package-path "$DIST_DIR" \
    "$PKG_FILE"

# Sign the .pkg if identity provided
if [ -n "$SIGN_IDENTITY" ]; then
    INSTALLER_IDENTITY="${SIGN_IDENTITY/Application/Installer}"
    productsign --sign "$INSTALLER_IDENTITY" "$PKG_FILE" "${PKG_FILE}.signed" 2>/dev/null && \
        mv "${PKG_FILE}.signed" "$PKG_FILE" || \
        echo "     Note: productsign failed (installer identity may differ)"
fi

# Cleanup temp files
rm -f "$COMPONENT_PKG" "$DIST_DIR/_distribution.xml" "$DIST_DIR/LICENSE-MIT"

echo "     Output: $PKG_FILE"
echo ""

# --- Step 5: Build .dmg disk image ---
echo "[5/5] Building .dmg disk image..."

DMG_FILE="$DIST_DIR/ArduboyEmulator-${VERSION}.dmg"
DMG_STAGING="$DIST_DIR/dmg-staging"

rm -rf "$DMG_STAGING" "$DMG_FILE"
mkdir -p "$DMG_STAGING"

# Copy .app to staging
cp -R "$APP_DIR" "$DMG_STAGING/"

# Create symlink to /Applications for drag-and-drop
ln -s /Applications "$DMG_STAGING/Applications"

# Add README
cp "$PROJECT_ROOT/README.md" "$DMG_STAGING/README.md"

# Create DMG
hdiutil create \
    -volname "${APP_NAME} ${VERSION}" \
    -srcfolder "$DMG_STAGING" \
    -ov -format UDZO \
    "$DMG_FILE"

# Cleanup
rm -rf "$DMG_STAGING" "$APP_DIR"

echo "     Output: $DMG_FILE"
echo ""

echo "==================================="
echo " Done. Packages in dist/macos/"
ls -lh "$DIST_DIR"/*.{pkg,dmg} 2>/dev/null || echo " (no packages)"
echo "==================================="
