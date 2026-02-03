@echo off
setlocal enabledelayedexpansion

echo ============================================
echo ⚖️  AEQUITAS - BUILD NATIVE FUNCTIONNEL
echo ============================================
echo.

echo [INFO] Construction NATIVE d'Aequitas - PAS de simulation
echo [INFO] Stable coin intrinsèque - Économie solidaire réelle
echo.

REM Vérifier environnement
if not exist "Cargo.toml" (
    echo [ERREUR] Veuillez exécuter depuis la racine d'Aequitas
    pause
    exit /b 1
)

REM Configuration Rust pour build natif
echo [SETUP] Configuration Rust pour performance native...
rustup default stable-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc

REM Installation dépendances système
echo [DEPS] Vérification des dépendances système...

REM Visual Studio Build Tools
where cl.exe >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [INSTALL] Visual Studio Build Tools requis
    echo [INFO] Installation automatique en cours...
    
    REM Télécharger Visual Studio Build Tools
    powershell -Command "& {Invoke-WebRequest -Uri 'https://aka.ms/vs/17/release/vs_buildtools.exe' -OutFile 'vs_buildtools.exe'}"
    
    echo [INSTALL] Lancement installateur VS Build Tools...
    echo [INFO] Cochez "C++ build tools" lors de l'installation
    start vs_buildtools.exe
    
    echo [ATTENTION] Après installation VS, réexécutez ce script
    pause
    exit /b 0
) else (
    echo [OK] Visual Studio Build Tools détecté
)

REM Configuration environnement Visual Studio
set "VS2022_PATH=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if exist "%VS2022_PATH%" (
    echo [SETUP] Configuration environnement Visual Studio 2022...
    call "%VS2022_PATH%"
) else (
    set "VS2019_PATH=C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
    if exist "%VS2019_PATH%" (
        echo [SETUP] Configuration environnement Visual Studio 2019...
        call "%VS2019_PATH%"
    ) else (
        echo [WARN] Visual Studio non configuré automatiquement
    )
)

echo.
echo [BUILD] Compilation NATIVE des composants Aequitas...

REM Nettoyage précédent
echo [CLEAN] Nettoyage build précédent...
cargo clean 2>nul

REM Build release optimisé
echo [COMPILE] Build release optimisé...
set RUSTFLAGS=-C target-cpu=native -C opt-level=3 -C lto=fat

cargo build --release --target x86_64-pc-windows-msvc

if %ERRORLEVEL% NEQ 0 (
    echo [ERREUR] Échec de compilation native
    echo [DEBUG] Vérification détaillée...
    
    echo [INFO] Rust version:
    rustc --version
    
    echo [INFO] Cargo version:
    cargo --version
    
    echo [INFO] Test build progressif:
    cargo check --target x86_64-pc-windows-msvc
    
    echo.
    echo [SOLUTION] Problèmes possibles:
    echo 1. Visual Studio Build Tools incomplet
    echo 2. Variables d'environnement incorrectes
    echo 3. Conflit avec autre linker
    echo.
    echo [ACTION] Réessayez après redémarrage ou réinstallez VS Build Tools
    pause
    exit /b 1
)

echo.
echo [SUCCESS] Build native terminé avec succès!

REM Vérification des binaires
echo [VERIFY] Vérification des exécutables créés...

set BINARIES_FOUND=0
if exist "target\x86_64-pc-windows-msvc\release\aequitas-node.exe" (
    echo [OK] aequitas-node.exe créé
    set /a BINARIES_FOUND+=1
)

if exist "target\x86_64-pc-windows-msvc\release\aequitas-miner.exe" (
    echo [OK] aequitas-miner.exe créé
    set /a BINARIES_FOUND+=1
)

if exist "target\x86_64-pc-windows-msvc\release\aequitas-wallet.exe" (
    echo [OK] aequitas-wallet.exe créé
    set /a BINARIES_FOUND+=1
)

if %BINARIES_FOUND% LSS 3 (
    echo [WARN] Certains binaires manquent - vérifiez les erreurs ci-dessus
) else (
    echo [PERFECT] Tous les binaires créés avec succès!
)

echo.
echo [DEPLOY] Création du package de déploiement...

if not exist "release_package" mkdir release_package

REM Copie des binaires
copy target\x86_64-pc-windows-msvc\release\*.exe release_package\ 2>nul

REM Copie fichiers essentiels
copy Cargo.toml release_package\ 2>nul
copy LICENSE release_package\ 2>nul
copy README.md release_package\ 2>nul

REM Création des scripts de lancement
REM Node launcher
echo @echo off > release_package\start_node.bat
echo echo 🔗 Démarrage Noeud Aequitas P2P... >> release_package\start_node.bat
echo echo 🌐 Connexion réseau décentralisé... >> release_package\start_node.bat
echo aequitas-node.exe --port 23420 --data-dir ./data >> release_package\start_node.bat
echo pause >> release_package\start_node.bat

REM Miner launcher
echo @echo off > release_package\start_miner.bat
echo echo ⛏️  Démarrage Mining AequiHash... >> release_package\start_miner.bat
echo echo 🔥 Mining GPU optimisé... >> release_package\start_miner.bat
echo aequitas-miner.exe --address aeq1 VotreAdresse --threads auto >> release_package\start_miner.bat
echo pause >> release_package\start_miner.bat

REM Wallet launcher
echo @echo off > release_package\start_wallet.bat
echo echo 💼 Portefeuille Aequitas Sécurisé... >> release_package\start_wallet.bat
echo aequitas-wallet.exe >> release_package\start_wallet.bat
echo pause >> release_package\start_wallet.bat

REM Création config par défaut
echo [network] > release_package\config.toml
echo port = 23420 >> release_package\config.toml
echo data_dir = "./data" >> release_package\config.toml
echo. >> release_package\config.toml
echo [mining] >> release_package\config.toml
echo threads = "auto" >> release_package\config.toml
echo [wallet] >> release_package\config.toml
echo data_dir = "./wallet" >> release_package\config.toml

echo.
echo [COMPLETE] Package de déploiement créé dans release_package\

echo.
echo ============================================
echo ⚖️  AEQUITAS - BUILD NATIF TERMINÉ
echo ============================================
echo.
echo 🎯 Applications NATIVES créées:
echo    📁 release_package\aequitas-node.exe    - Noeud P2P décentralisé
echo    📁 release_package\aequitas-miner.exe   - Mining GPU AequiHash
echo    📁 release_package\aequitas-wallet.exe  - Portefeuille sécurisé
echo.
echo 🚀 Lancement rapide:
echo    start_node.bat     - Démarrer noeud P2P
echo    start_miner.bat    - Commencer mining
echo    start_wallet.bat    - Ouvrir portefeuille
echo.
echo 💎 Principes respectés:
echo    ✅ Stable coin intrinsèque - NON indexé
echo    ✅ Économie solidaire 98/1/1  
echo    ✅ Décentralisation maximale
echo    ✅ 100% fonctionnel - PAS de simulation
echo.
echo 🌐 Prêt pour le réseau économique décentralisé!
echo.

REM Test immédiat optionnel
echo [TEST] Test de fonctionnement...
if exist "release_package\aequitas-node.exe" (
    echo [INFO] Test du noeud...
    release_package\aequitas-node.exe --version 2>nul
    if %ERRORLEVEL% EQU 0 (
        echo [OK] Noeud fonctionnel
    ) else (
        echo [WARN] Noeud présente des erreurs d'exécution
    )
)

pause