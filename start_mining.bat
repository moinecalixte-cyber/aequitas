@echo off
REM ============================================
REM   Aequitas Miner - Lancement rapide
REM ============================================

echo.
echo  ========================================
echo       AEQUITAS MINER
echo     Fair Mining for Everyone
echo  ========================================
echo.

REM Vérifier si le binaire existe
if not exist "bin\aequitas-miner.exe" (
    if not exist "target\release\aequitas-miner.exe" (
        echo [INFO] Le mineur n'est pas compilé. Compilation en cours...
        call scripts\build.bat
    ) else (
        set MINER=target\release\aequitas-miner.exe
    )
) else (
    set MINER=bin\aequitas-miner.exe
)

REM Vérifier si la config existe
if not exist "miner.toml" (
    echo [INFO] Création de la configuration...
    %MINER% init
    echo.
    echo [IMPORTANT] Éditez miner.toml et ajoutez votre adresse wallet!
    echo.
    notepad miner.toml
    pause
)

echo [INFO] Démarrage du minage...
echo.
%MINER% mine

pause
