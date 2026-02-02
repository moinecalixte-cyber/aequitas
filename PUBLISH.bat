@echo off
REM ============================================
REM   PUBLICATION AEQUITAS SUR GITHUB
REM ============================================
title Aequitas - Publication GitHub

echo.
echo  ========================================
echo     PUBLICATION AEQUITAS SUR GITHUB
echo  ========================================
echo.

REM Vérifier Git
where git >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [ERREUR] Git n'est pas installe!
    echo.
    echo Installez Git depuis: https://git-scm.com/download/win
    echo.
    echo Ou avec winget:
    echo   winget install Git.Git
    echo.
    pause
    exit /b 1
)

echo [OK] Git detecte: 
git --version
echo.

REM Initialiser le repo si nécessaire
if not exist ".git" (
    echo [INFO] Initialisation du repository Git...
    git init
    git branch -M main
    echo.
)

REM Configuration Git si nécessaire
git config user.name >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [INFO] Configuration Git requise...
    set /p GIT_NAME="Votre nom: "
    set /p GIT_EMAIL="Votre email: "
    git config user.name "%GIT_NAME%"
    git config user.email "%GIT_EMAIL%"
    echo.
)

REM Ajouter tous les fichiers
echo [INFO] Ajout des fichiers...
git add .

REM Vérifier s'il y a des changements
git diff --cached --quiet 2>nul
if %ERRORLEVEL% equ 0 (
    echo [INFO] Aucun nouveau changement a commiter.
) else (
    echo [INFO] Creation du commit...
    git commit -m "Aequitas v0.1.0 - Initial release"
)

echo.
echo  ----------------------------------------
echo    CREATION DU REPOSITORY GITHUB
echo  ----------------------------------------
echo.
echo  Pour publier sur GitHub:
echo.
echo  1. Allez sur https://github.com/new
echo.
echo  2. Creez un nouveau repository:
echo     - Nom: aequitas
echo     - Description: Aequitas - Fair Mining Cryptocurrency
echo     - Public (recommande)
echo     - NE PAS initialiser avec README
echo.
echo  3. Copiez l'URL du repository
echo     (ex: https://github.com/VOTRE_USERNAME/aequitas.git)
echo.

set /p REPO_URL="Collez l'URL du repository (ou appuyez sur Entree pour annuler): "

if "%REPO_URL%"=="" (
    echo.
    echo [INFO] Publication annulee.
    echo Vous pouvez publier plus tard avec:
    echo   git remote add origin VOTRE_URL
    echo   git push -u origin main
    pause
    exit /b 0
)

REM Ajouter le remote
git remote remove origin 2>nul
git remote add origin %REPO_URL%

echo.
echo [INFO] Publication sur GitHub...
echo.
echo Si demande, entrez vos identifiants GitHub.
echo (Utilisez un Personal Access Token comme mot de passe)
echo.

git push -u origin main

if %ERRORLEVEL% equ 0 (
    echo.
    echo  ========================================
    echo      PUBLICATION REUSSIE!
    echo  ========================================
    echo.
    echo  Votre projet est maintenant sur GitHub!
    echo.
    echo  URL: %REPO_URL%
    echo.
    echo  Prochaines etapes:
    echo    1. Activez GitHub Actions (Settings ^> Actions)
    echo    2. Creez une release: git tag v0.1.0 ^&^& git push --tags
    echo    3. Partagez le lien avec la communaute!
    echo.
) else (
    echo.
    echo [ATTENTION] Probleme lors de la publication.
    echo.
    echo Solutions:
    echo   1. Verifiez vos identifiants GitHub
    echo   2. Utilisez un Personal Access Token
    echo      (GitHub ^> Settings ^> Developer settings ^> Tokens)
    echo   3. Verifiez l'URL du repository
    echo.
)

pause
