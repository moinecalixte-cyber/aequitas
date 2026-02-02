@echo off
REM ============================================
REM   INSTALLATION COMPLETE AEQUITAS
REM ============================================
title Aequitas - Installation

echo.
echo  ========================================
echo      AEQUITAS CRYPTOCURRENCY
echo      Installation Complete
echo  ========================================
echo.

REM Vérifier si Rust est installé
where rustc >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo [OK] Rust est deja installe!
    rustc --version
    goto :BUILD
)

echo [INFO] Rust n'est pas installe.
echo.
echo Cette installation va:
echo   1. Telecharger Rust (rustup)
echo   2. Installer Rust et Cargo
echo   3. Compiler Aequitas
echo   4. Creer les binaires prets a l'emploi
echo.

set /p INSTALL="Voulez-vous continuer? (O/N): "
if /i not "%INSTALL%"=="O" (
    echo Installation annulee.
    pause
    exit /b 0
)

echo.
echo [1/4] Telechargement de Rust...
echo.

REM Télécharger rustup-init
powershell -Command "& {Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile 'rustup-init.exe'}"

if not exist "rustup-init.exe" (
    echo [ERREUR] Echec du telechargement de Rust.
    echo Installez manuellement depuis: https://rustup.rs
    pause
    exit /b 1
)

echo.
echo [2/4] Installation de Rust...
echo.
echo Suivez les instructions de l'installateur Rust.
echo Choisissez l'option 1 (installation par defaut).
echo.

rustup-init.exe -y

REM Recharger le PATH
set PATH=%USERPROFILE%\.cargo\bin;%PATH%

del rustup-init.exe 2>nul

echo.
echo [OK] Rust installe!

:BUILD
echo.
echo [3/4] Compilation d'Aequitas...
echo      Cela peut prendre 5-10 minutes...
echo.

cargo build --release

if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERREUR] La compilation a echoue.
    echo Verifiez les erreurs ci-dessus.
    pause
    exit /b 1
)

echo.
echo [4/4] Preparation des binaires...
echo.

if not exist "bin" mkdir bin

copy /Y target\release\aequitas-miner.exe bin\ >nul 2>nul
copy /Y target\release\aequitas-node.exe bin\ >nul 2>nul
copy /Y target\release\aequitas-wallet.exe bin\ >nul 2>nul
copy /Y miner.toml.example bin\miner.toml >nul 2>nul

echo.
echo  ========================================
echo      INSTALLATION TERMINEE!
echo  ========================================
echo.
echo  Les binaires sont dans le dossier: bin\
echo.
echo  PROCHAINES ETAPES:
echo.
echo  1. Creer un wallet:
echo     bin\aequitas-wallet new --password "VotreMotDePasse"
echo.
echo  2. Configurer le mineur:
echo     Editez bin\miner.toml et ajoutez votre adresse
echo.
echo  3. Lancer le node:
echo     bin\aequitas-node run
echo.
echo  4. Lancer le mineur:
echo     bin\aequitas-miner mine
echo.
echo  Documentation: docs\MINING_GUIDE.md
echo.

pause
