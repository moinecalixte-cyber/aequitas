@echo off
echo Installing MinGW-w64 for Rust compilation...

REM Check if chocolatey is installed
where choco >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Installing Chocolatey...
    powershell -Command "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"
    call refreshenv
)

echo Installing MinGW-w64...
choco install mingw -y

echo Adding MinGW to PATH...
set PATH=C:\ProgramData\chocolatey\lib\mingw\tools\install\mingw64\bin;%PATH%

echo Verifying GCC installation...
gcc --version

echo MinGW installation complete!
echo Please restart your terminal and run: cargo build