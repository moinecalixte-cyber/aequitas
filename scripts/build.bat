@echo off
REM ============================================
REM   Aequitas - Script de compilation Windows
REM ============================================

echo.
echo  ========================================
echo     AEQUITAS BUILD SCRIPT
echo  ========================================
echo.

REM Vérifier Rust
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [ERREUR] Rust n'est pas installé!
    echo Installez Rust depuis: https://rustup.rs
    pause
    exit /b 1
)

echo [1/4] Vérification de Rust...
cargo --version

echo.
echo [2/4] Nettoyage des anciens builds...
cargo clean

echo.
echo [3/4] Compilation en mode release...
echo      Cela peut prendre plusieurs minutes...
echo.

cargo build --release

if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERREUR] La compilation a échoué!
    pause
    exit /b 1
)

echo.
echo [4/4] Copie des binaires...

if not exist "bin" mkdir bin
copy /Y target\release\aequitas-miner.exe bin\ >nul 2>nul

echo.
echo  ========================================
echo     COMPILATION TERMINÉE !
echo  ========================================
echo.
echo  Binaires disponibles dans : bin\
echo.
echo  Pour démarrer le minage :
echo    1. Éditez miner.toml avec votre adresse
echo    2. Lancez: bin\aequitas-miner.exe mine
echo.

pause
