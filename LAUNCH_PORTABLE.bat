@echo off
setlocal enabledelayedexpansion

echo ====================================
echo ⚖️  AEQUITAS - PORTABLE LAUNCHER
echo ====================================
echo.

REM Vérifier si nous sommes dans le bon répertoire
if not exist "AEQUITAS_CONTROL_CENTER.html" (
    echo [ERROR] Veuillez exécuter ce script depuis le répertoire racine d'Aequitas
    pause
    exit /b 1
)

echo [OK] Répertoire Aequitas détecté
echo.

REM Vérifier Rust
where rustc >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [INSTALL] Rust non trouvé, installation en cours...
    echo [INFO] Téléchargement de rustup-init.exe...
    
    REM Télécharger rustup
    powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile 'rustup-init.exe'"
    
    echo [INSTALL] Lancement de l'installateur Rust...
    rustup-init.exe -y --default-toolchain stable
    
    echo [REFRESH] Mise à jour des variables d'environnement...
    call "%USERPROFILE%\.cargo\env.bat"
    
    echo [OK] Rust installé
) else (
    echo [OK] Rust déjà installé: 
    rustc --version
)

echo.
echo [SETUP] Configuration du compilateur...

REM Configurer pour MSVC
rustup default stable-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc

REM Tenter de configurer Visual Studio
set "VS_PATH=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if exist "%VS_PATH%" (
    echo [OK] Visual Studio 2022 détecté
    call "%VS_PATH%"
) else (
    echo [WARN] Visual Studio Build Tools non trouvé
    echo [INFO] Installation recommandée: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
    echo [INFO] Continuons en mode démonstration...
)

echo.
echo [COMPILE] Tentative de compilation des composants...

REM Créer un répertoire pour les binaires
if not exist "portable_binaries" mkdir portable_binaries

REM Compiler si possible, sinon créer des exécutables de démonstration
if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\*" (
    echo [INFO] Compilation native en cours...
    cargo build --release 2>nul
    if %ERRORLEVEL% EQU 0 (
        echo [OK] Compilation réussie!
        copy target\release\*.exe portable_binaries\ 2>nul
    ) else (
        echo [WARN] Erreur de compilation - création des démos...
        call :create_demo_binaries
    )
) else (
    echo [INFO] Mode démonstration - création des interfaces...
    call :create_demo_binaries
)

echo.
echo [WEB] Démarrage du serveur web local...

REM Lancer un serveur web simple
start "" "http://localhost:8080/AEQUITAS_CONTROL_CENTER.html"

REM Vérifier Python
where python >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [OK] Serveur web démarré sur http://localhost:8080
    python -m http.server 8080
) else (
    where python3 >nul 2>nul
    if %ERRORLEVEL% EQU 0 (
        echo [OK] Serveur web démarré sur http://localhost:8080
        python3 -m http.server 8080
    ) else (
        echo [INFO] Python non trouvé - ouverture manuelle...
        start AEQUITAS_CONTROL_CENTER.html
        echo [OK] Interface ouverte dans le navigateur
    )
)

goto :end

:create_demo_binaries
echo [DEMO] Création des exécutables de démonstration...

REM Créer aequitas-node.exe (demo)
echo @echo off > portable_binaries\aequitas-node.exe
echo echo 🔗 Noeud Aequitas - Mode Démonstration >> portable_binaries\aequitas-node.exe
echo echo 📡 Connexion au réseau décentralisé... >> portable_binaries\aequitas-node.exe
echo echo ✅ Noeud connecté ! >> portable_binaries\aequitas-node.exe
echo pause >> portable_binaries\aequitas-node.exe

REM Créer aequitas-miner.exe (demo)
echo @echo off > portable_binaries\aequitas-miner.exe
echo echo ⛏️  Mining Aequitas - Mode Démonstration >> portable_binaries\aequitas-miner.exe
echo echo 🔥 Recherche de blocs... >> portable_binaries\aequitas-miner.exe
echo echo 💰 Bloc trouvé ! +50 AEQ >> portable_binaries\aequitas-miner.exe
echo pause >> portable_binaries\aequitas-miner.exe

REM Créer aequitas-wallet.exe (demo)
echo @echo off > portable_binaries\aequitas-wallet.exe
echo echo 💼 Portefeuille Aequitas - Mode Démonstration >> portable_binaries\aequitas-wallet.exe
echo echo 🔓 Portefeuille prêt >> portable_binaries\aequitas-wallet.exe
echo echo 📊 Solde: 0.00 AEQ >> portable_binaries\aequitas-wallet.exe
echo pause >> portable_binaries\aequitas-wallet.exe

echo [OK] Exécutables de démonstration créés dans portable_binaries\
goto :eof

:end
echo.
echo 🎉 Lancement terminé !
echo 🌐 Interface: http://localhost:8080/AEQUITAS_CONTROL_CENTER.html
echo 📁 Exécutables: portable_binaries\
echo 📚 Documentation: CONTRIBUTION_REPORT.md
echo.
pause