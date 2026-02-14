@echo off
rem ============================================================
rem  Arduboy Emulator - Windows Installer Builder v0.7.3
rem ============================================================

set VERSION=0.7.3
set PROJECT_ROOT=%~dp0..\..

echo ===================================
echo  Arduboy Emulator v%VERSION%
echo  Windows Installer Builder
echo ===================================
echo.

rem --- Find cargo executable ---
set "CARGO=cargo"
cargo --version >nul 2>&1 && goto cargo_ok

rem Try common install locations
set "CARGO=%USERPROFILE%\.cargo\bin\cargo.exe"
if exist "%CARGO%" goto cargo_ok

set "CARGO=%CARGO_HOME%\bin\cargo.exe"
if exist "%CARGO%" goto cargo_ok

set "CARGO=%LOCALAPPDATA%\Programs\Rust\bin\cargo.exe"
if exist "%CARGO%" goto cargo_ok

echo ERROR: cargo not found.
echo.
echo Searched:
echo   - PATH
echo   - %%USERPROFILE%%\.cargo\bin\cargo.exe
echo   - %%CARGO_HOME%%\bin\cargo.exe
echo   - %%LOCALAPPDATA%%\Programs\Rust\bin\cargo.exe
echo.
echo Install Rust from https://rustup.rs/ then reopen this terminal.
exit /b 1

:cargo_ok
echo      cargo: OK
echo.

rem --- Step 1: Build release binary ---
echo [1/3] Building release binary...
pushd "%PROJECT_ROOT%"
"%CARGO%" build --release -p arduboy-frontend
if errorlevel 1 (
    echo ERROR: cargo build failed
    popd
    exit /b 1
)
popd

if not exist "%PROJECT_ROOT%\target\release\arduboy-frontend.exe" (
    echo ERROR: arduboy-frontend.exe not found
    exit /b 1
)
echo      OK: target\release\arduboy-frontend.exe
echo.

rem --- Step 2: Find Inno Setup ---
echo [2/3] Locating Inno Setup compiler...
set "ISCC=iscc"
iscc /? >nul 2>&1 && goto iscc_found

set "ISCC=C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
if exist "%ISCC%" goto iscc_found

set "ISCC=C:\Program Files\Inno Setup 6\ISCC.exe"
if exist "%ISCC%" goto iscc_found

echo WARNING: Inno Setup 6 not found. Skipping installer creation.
echo          Install from https://jrsoftware.org/isinfo.php
echo.
echo Binary is ready at: target\release\arduboy-frontend.exe
exit /b 0

:iscc_found
echo      ISCC: %ISCC%
echo.

rem --- Step 3: Build installer ---
echo [3/3] Building installer...
if not exist "%PROJECT_ROOT%\dist\windows" mkdir "%PROJECT_ROOT%\dist\windows"
"%ISCC%" "%~dp0arduboy-emu.iss"
if errorlevel 1 (
    echo ERROR: Inno Setup compilation failed
    exit /b 1
)

echo.
echo ===================================
echo  SUCCESS
echo  dist\windows\arduboy-emu-%VERSION%-setup-x64.exe
echo ===================================
