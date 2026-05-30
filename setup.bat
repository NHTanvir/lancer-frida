@echo off
setlocal enabledelayedexpansion
title lancer-frida Setup

echo.
echo  ================================================
echo    lancer-frida  Setup
echo    Installs all required build dependencies
echo  ================================================
echo.

REM Check for winget
where winget >nul 2>&1
if errorlevel 1 (
    echo  ERROR: winget not found.
    echo  Please install App Installer from the Microsoft Store, then re-run.
    pause
    exit /b 1
)

REM ---- 1. Rust / Cargo ----
where cargo >nul 2>&1
if errorlevel 1 (
    set "CARGO_BIN=%USERPROFILE%\.cargo\bin\cargo.exe"
    if not exist "!CARGO_BIN!" (
        echo  [1/3]  Installing Rust via rustup...
        winget install --id Rustlang.Rustup -e --silent
        if errorlevel 1 (
            echo  ERROR: Failed to install Rust. Visit https://rustup.rs
            pause
            exit /b 1
        )
        echo  [1/3]  Rust installed. Restart this terminal after setup to use cargo.
    ) else (
        echo  [1/3]  Rust already installed.
    )
) else (
    echo  [1/3]  Rust already installed.
)

REM ---- 2. Frida tools (runtime dependency — the CLI delegates to them) ----
where frida >nul 2>&1
if errorlevel 1 (
    echo  [2/3]  Installing Frida tools (frida, frida-ps, frida-trace, ...)...
    where pip >nul 2>&1
    if errorlevel 1 (
        echo         pip not found. Install Python 3.10+ then run: pip install frida-tools
    ) else (
        pip install --user frida-tools
        if errorlevel 1 (
            echo         WARNING: pip install frida-tools failed. Install manually: pip install frida-tools
        ) else (
            echo  [2/3]  Frida tools installed.
        )
    )
) else (
    echo  [2/3]  Frida tools already installed.
)

REM ---- 3. Visual C++ Build Tools ----
echo  [3/3]  Checking Visual C++ Build Tools...
where cl >nul 2>&1
if errorlevel 1 (
    echo         Not found in PATH.
    echo         If not installed, run:
    echo           winget install --id Microsoft.VisualStudio.2022.BuildTools -e
    echo         Then reopen a "Developer Command Prompt for VS 2022".
) else (
    echo  [3/3]  Visual C++ Build Tools found.
)

echo.
echo  ================================================
echo    Setup complete!
echo.
echo    IMPORTANT: Open a NEW terminal window so all
echo    PATH changes take effect, then run:
echo.
echo      build.bat
echo  ================================================
echo.
pause
endlocal
