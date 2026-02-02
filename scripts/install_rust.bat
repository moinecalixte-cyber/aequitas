@echo off
REM ============================================
REM   Installation de Rust pour Windows
REM ============================================

echo.
echo  ========================================
echo     INSTALLATION DE RUST
echo  ========================================
echo.

REM Vérifier si Rust est déjà installé
where rustc >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo [OK] Rust est déjà installé!
    rustc --version
    cargo --version
    echo.
    echo Vous pouvez maintenant compiler Aequitas avec:
    echo   scripts\build.bat
    pause
    exit /b 0
)

echo [INFO] Rust n'est pas détecté sur ce système.
echo.
echo Pour installer Rust:
echo.
echo 1. Téléchargez rustup depuis: https://rustup.rs
echo    Ou exécutez dans PowerShell:
echo    irm https://sh.rustup.rs | iex
echo.
echo 2. Suivez les instructions d'installation
echo.
echo 3. Redémarrez votre terminal
echo.
echo 4. Vérifiez avec: cargo --version
echo.
echo 5. Compilez Aequitas avec: scripts\build.bat
echo.

REM Ouvrir la page de téléchargement
echo Voulez-vous ouvrir la page de téléchargement de Rust? (O/N)
set /p choice=
if /i "%choice%"=="O" (
    start https://rustup.rs
)

pause
