# Building & Packaging

## Quick Build (Development)

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo run -- game.hex          # Run directly
```

## Creating Installers

### Automatic (detect OS)

```bash
./build-installers.sh
```

### Windows — Inno Setup (.exe)

**Prerequisites:**
- [Rust](https://rustup.rs/)
- [Inno Setup 6](https://jrsoftware.org/isinfo.php)

```cmd
cd installers\windows
build-windows.bat
```

**Output:** `dist\windows\arduboy-emu-0.7.1-setup-x64.exe`

The installer includes:
- Start Menu shortcut
- Optional desktop shortcut
- Optional `.hex` / `.arduboy` file associations
- Uninstaller

### Linux — .deb / .rpm

**Prerequisites:**
- Rust, plus build dependencies:
  ```bash
  # Debian/Ubuntu
  sudo apt install libasound2-dev libx11-dev libxcursor-dev \
    libxrandr-dev libxi-dev libudev-dev

  # Fedora
  sudo dnf install alsa-lib-devel libX11-devel libXcursor-devel \
    libXrandr-devel libXi-devel systemd-devel
  ```
- `dpkg-deb` for .deb (pre-installed on Debian/Ubuntu)
- `rpmbuild` for .rpm (`sudo dnf install rpm-build` / `sudo apt install rpm`)

```bash
./installers/linux/build-linux.sh             # Both .deb and .rpm
./installers/linux/build-linux.sh --deb       # .deb only
./installers/linux/build-linux.sh --rpm       # .rpm only
```

**Output:**
- `dist/linux/arduboy-emu_0.7.1_amd64.deb`
- `dist/linux/arduboy-emu-0.7.1-1.x86_64.rpm`

**Install:**
```bash
sudo dpkg -i dist/linux/arduboy-emu_0.7.1_amd64.deb     # Debian/Ubuntu
sudo rpm -i dist/linux/arduboy-emu-0.7.1-1.x86_64.rpm   # Fedora/RHEL
```

Packages install `arduboy-emu` to `/usr/bin/` with a `.desktop` entry, MIME type for `.arduboy` files, and AppStream metadata.

### macOS — .pkg / .dmg

**Prerequisites:**
- Rust
- Xcode Command Line Tools (`xcode-select --install`)

```bash
./installers/macos/build-macos.sh                        # Native arch
./installers/macos/build-macos.sh --universal             # x86_64 + aarch64
./installers/macos/build-macos.sh --sign "Developer ID"   # Code-signed
```

**Output:**
- `dist/macos/ArduboyEmulator-0.7.1.pkg` — Standard macOS installer
- `dist/macos/ArduboyEmulator-0.7.1.dmg` — Drag-and-drop disk image

The `.app` bundle includes `Info.plist` with file type associations for `.hex`, `.arduboy`, and `.elf` files.

## CI/CD (GitHub Actions)

Push a tag to trigger automatic builds on all platforms:

```bash
git tag v0.7.1
git push origin v0.7.1
```

The workflow (`.github/workflows/release.yml`) builds all packages and creates a draft GitHub Release with all artifacts attached. See the workflow file for details.

## Adding an Application Icon

Place icon files in `assets/`:
- `assets/arduboy-emu.ico` — Windows (used by Inno Setup)
- `assets/arduboy-emu.icns` — macOS (bundled into .app)
- `assets/arduboy-emu.png` — Linux (128×128 recommended, for .desktop)

Then uncomment the `SetupIconFile` line in `installers/windows/arduboy-emu.iss`.

## File Structure

```
installers/
├── windows/
│   ├── arduboy-emu.iss       # Inno Setup script
│   └── build-windows.bat     # Build automation
├── linux/
│   └── build-linux.sh        # .deb + .rpm builder
└── macos/
    └── build-macos.sh        # .app + .pkg + .dmg builder
build-installers.sh            # Cross-platform entry point
.github/workflows/release.yml  # CI/CD pipeline
```
