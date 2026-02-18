@echo off
rem ============================================================
rem  Arduboy Emulator - MSI Installer Builder v0.8.1
rem  Uses WiX Toolset v5/v6 (dotnet tool)
rem
rem  Prerequisites:
rem    - Rust toolchain (rustup)
rem    - .NET SDK 6+ (https://dotnet.microsoft.com/download)
rem    - WiX: dotnet tool install -g wix
rem
rem  Usage:  build-msi.bat
rem  Output: dist\windows\arduboy-emu-0.8.1-x64.msi
rem ============================================================

set VERSION=0.8.1
set PROJECT_ROOT=%~dp0..\..

echo ===================================
echo  Arduboy Emulator v%VERSION%
echo  MSI Installer Builder
echo ===================================
echo.

rem --- Find cargo ---
set "CARGO=cargo"
cargo --version >nul 2>&1 && goto cargo_ok
set "CARGO=%USERPROFILE%\.cargo\bin\cargo.exe"
if exist "%CARGO%" goto cargo_ok
echo ERROR: cargo not found. Install Rust from https://rustup.rs/
exit /b 1

:cargo_ok
echo      cargo: OK

rem --- Find wix ---
set "WIX=wix"
wix --version >nul 2>&1 && goto wix_ok
set "WIX=%USERPROFILE%\.dotnet\tools\wix.exe"
if exist "%WIX%" goto wix_ok
echo ERROR: WiX Toolset not found.
echo        Install: dotnet tool install -g wix
exit /b 1

:wix_ok
echo      wix:   OK
echo.

rem --- Step 1: Build release binary ---
echo [1/2] Building release binary...
pushd "%PROJECT_ROOT%"
"%CARGO%" build --release -p arduboy-frontend
if errorlevel 1 (
    echo ERROR: cargo build failed
    popd
    exit /b 1
)
popd

if not exist "%PROJECT_ROOT%\target\release\arduboy-emu.exe" (
    echo ERROR: arduboy-emu.exe not found
    exit /b 1
)
echo      OK: target\release\arduboy-emu.exe
echo.

rem --- Step 2: Build MSI ---
echo [2/2] Building MSI installer...
if not exist "%PROJECT_ROOT%\dist\windows" mkdir "%PROJECT_ROOT%\dist\windows"
pushd "%~dp0"
"%WIX%" build -o "%PROJECT_ROOT%\dist\windows\arduboy-emu-%VERSION%-x64.msi" arduboy-emu.wxs
if errorlevel 1 (
    echo ERROR: WiX build failed
    popd
    exit /b 1
)
popd

echo.
echo ===================================
echo  SUCCESS
echo  dist\windows\arduboy-emu-%VERSION%-x64.msi
echo ===================================
