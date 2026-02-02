@echo off
REM ============================================
REM   Publication sur GitHub
REM ============================================

echo.
echo  ========================================
echo     PUBLICATION AEQUITAS SUR GITHUB
echo  ========================================
echo.

REM Vérifier Git
where git >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [ERREUR] Git n'est pas installe!
    echo Installez Git depuis: https://git-scm.com
    pause
    exit /b 1
)

REM Initialiser le repo Git si nécessaire
if not exist ".git" (
    echo [INFO] Initialisation du repository Git...
    git init
    git branch -M main
)

REM Ajouter tous les fichiers
echo [INFO] Ajout des fichiers...
git add .

REM Commit
echo [INFO] Creation du commit...
git commit -m "Initial release - Aequitas v0.1.0"

REM Demander l'URL du repository
echo.
echo Entrez l'URL de votre repository GitHub:
echo (ex: https://github.com/votre-username/aequitas.git)
set /p REPO_URL=URL: 

if "%REPO_URL%"=="" (
    echo [ERREUR] URL requise!
    pause
    exit /b 1
)

REM Ajouter le remote
git remote remove origin 2>nul
git remote add origin %REPO_URL%

REM Push
echo.
echo [INFO] Publication sur GitHub...
git push -u origin main

if %ERRORLEVEL% equ 0 (
    echo.
    echo  ========================================
    echo     PUBLICATION REUSSIE!
    echo  ========================================
    echo.
    echo Votre projet est maintenant disponible sur GitHub.
    echo.
    echo Prochaines etapes:
    echo   1. Verifiez le repository sur GitHub
    echo   2. Activez GitHub Actions pour les builds automatiques
    echo   3. Creez une release avec: git tag v0.1.0 ^&^& git push --tags
    echo.
) else (
    echo.
    echo [ERREUR] La publication a echoue.
    echo Verifiez vos identifiants GitHub.
    echo.
)

pause
