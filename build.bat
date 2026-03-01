@echo off
setlocal enabledelayedexpansion
title lancer-slither Build

REM ============================================================
REM  build.bat -- Build lancer-slither binaries for Windows.
REM  Outputs (release): build\lancer-slither.exe plus every companion.
REM
REM  Requirements:
REM    - Rust / Cargo   https://rustup.rs
REM    - Visual C++ Build Tools (2019/2022)
REM ============================================================

set "REPO=%~dp0"
set "REPO=%REPO:~0,-1%"
set "BUILD=%REPO%\build"

echo.
echo  ================================================
echo    lancer-slither  Build
echo  ================================================
echo.
echo  Repo   : %REPO%
echo  Output : %BUILD%\
echo.

REM Locate cargo
where cargo >/dev/null 2>&1
if errorlevel 1 (
    set "CARGO=%USERPROFILE%\.cargo\bin\cargo.exe"
    if not exist "!CARGO!" (
        echo  ERROR: cargo not found.
        echo  Install Rust from https://rustup.rs then re-run.
        pause
        exit /b 1
    )
) else (
    set "CARGO=cargo"
)

echo  [1/2]  Building release workspace...
"!CARGO!" build --release
if errorlevel 1 (
    echo  ERROR: cargo build failed
    pause
    exit /b 1
)

echo  [2/2]  Copying to build\...
if not exist "%BUILD%" mkdir "%BUILD%"
for %%B in (
    lancer-slither
    lancer-slither-cli
    lancer-slither-mutate
    lancer-slither-prop
    lancer-slither-check-upgradeability
    lancer-slither-read-storage
    lancer-slither-find-paths
    lancer-slither-format
    lancer-slither-flat
    lancer-slither-check-erc
    lancer-slither-interface
    lancer-slither-doctor
    lancer-slither-documentation
) do (
    if exist "target\release\%%B.exe" copy /Y "target\release\%%B.exe" "%BUILD%\" >/dev/null
)

echo.
echo  ================================================
echo    Done!  Binaries in %BUILD%\
echo  ================================================
dir "%BUILD%"
echo.
pause
endlocal
