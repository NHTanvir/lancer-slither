@echo off
setlocal enabledelayedexpansion
title lancer-slither windows dist

REM Windows release build + packaged dist.
REM Output: dist\lancer-slither-windows-x86_64\bin\lancer-slither.exe + every companion binary.

echo === Building lancer-slither (release) for Windows x86_64 ===
cargo build --release
if errorlevel 1 exit /b 1

echo === Packaging ===
set DIST=dist\lancer-slither-windows-x86_64
if exist "%DIST%" rmdir /s /q "%DIST%"
mkdir "%DIST%"
mkdir "%DIST%\bin"

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
    if exist "target\release\%%B.exe" copy /Y "target\release\%%B.exe" "%DIST%\bin\" >/dev/null
)

copy /Y LICENSE "%DIST%\" >/dev/null 2>&1
copy /Y README.md "%DIST%\" >/dev/null 2>&1

echo === Done. Binaries in %DIST%\bin ===
dir "%DIST%\bin"
exit /b 0
