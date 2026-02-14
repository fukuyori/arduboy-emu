#!/usr/bin/env bash
# ============================================================
#  Arduboy Emulator — Cross-Platform Installer Builder
#  Detects the current OS and runs the appropriate build script.
#
#  Usage:  ./build-installers.sh [--deb|--rpm|--all|--sign ID|--universal]
#  Output: dist/<platform>/
# ============================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Detecting platform..."
case "$(uname -s)" in
    Linux*)
        echo "Platform: Linux"
        echo ""
        exec "$SCRIPT_DIR/installers/linux/build-linux.sh" "$@"
        ;;
    Darwin*)
        echo "Platform: macOS"
        echo ""
        exec "$SCRIPT_DIR/installers/macos/build-macos.sh" "$@"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        echo "Platform: Windows"
        echo ""
        echo "Run installers\\windows\\build-windows.bat from a Windows command prompt."
        echo "(Bash wrappers cannot invoke Inno Setup directly.)"
        exit 1
        ;;
    *)
        echo "ERROR: Unsupported platform: $(uname -s)"
        exit 1
        ;;
esac
