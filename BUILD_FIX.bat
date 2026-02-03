@echo off
echo ====================================
echo Aequitas Build Environment Setup
echo ====================================
echo.

REM Check for Visual Studio Build Tools
where cl.exe >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [OK] Visual Studio compiler found
) else (
    echo [ERROR] Visual Studio Build Tools not found!
    echo Please install Visual Studio Build Tools 2022 with "C++ build tools" workload
    echo Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
    echo.
    echo After installation, run this script from "Developer Command Prompt for VS 2022"
    pause
    exit /b 1
)

echo.
echo Setting up Rust environment...
rustup default stable-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc

echo.
echo Building Aequitas...
cargo build --release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo [SUCCESS] Build completed successfully!
    echo Executables are in: target\release\
    echo.
    echo Available binaries:
    echo - aequitas-node.exe    : Run a full node
    echo - aequitas-miner.exe   : Start mining
    echo - aequitas-wallet.exe  : Wallet interface
) else (
    echo.
    echo [ERROR] Build failed!
    echo Check the error messages above for details.
)

echo.
pause