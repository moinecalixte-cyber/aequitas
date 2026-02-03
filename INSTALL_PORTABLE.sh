#!/bin/bash

# Aequitas Portable Launcher
# Script d'installation et lancement automatique

echo "⚖️  AEQUITAS - Installation Portable"
echo "======================================"
echo ""

# Vérifier le système d'exploitation
OS="Unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="Linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macOS"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    OS="Windows"
fi

echo "🖥️  Système détecté: $OS"
echo ""

# Installer Rust si non présent
if ! command -v rustc &> /dev/null; then
    echo "📦 Installation de Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME/.cargo/env"
else
    echo "✅ Rust déjà installé: $(rustc --version)"
fi

# Installer les dépendances système
case $OS in
    "Linux")
        echo "📦 Installation des dépendances Linux..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y build-essential pkg-config libssl-dev
        elif command -v yum &> /dev/null; then
            sudo yum groupinstall -y "Development Tools"
            sudo yum install -y openssl-devel
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --needed base-devel openssl
        fi
        ;;
    "macOS")
        echo "📦 Installation des dépendances macOS..."
        if command -v brew &> /dev/null; then
            brew install openssl
        else
            echo "⚠️  Homebrew non trouvé. Veuillez l'installer: https://brew.sh"
        fi
        ;;
    "Windows")
        echo "📦 Configuration pour Windows..."
        echo "⚠️  Veuillez utiliser BUILD_FIX.bat pour Windows"
        ;;
esac

echo ""
echo "🔧 Compilation des composants Aequitas..."

# Compiler les différents modules
modules=("core" "consensus" "network" "wallet" "miner" "node")

for module in "${modules[@]}"; do
    echo "📚 Compilation du module: $module"
    cd "src/$module" 2>/dev/null || echo "⚠️  Module $module non trouvé"
    
    if [[ -f "Cargo.toml" ]]; then
        cargo build --release 2>/dev/null && echo "✅ $module compilé avec succès" || echo "❌ Erreur de compilation $module"
    fi
    
    cd ../../..
done

echo ""
echo "🚀 Lancement du centre de contrôle Aequitas..."

# Lancer l'interface web
if command -v python3 &> /dev/null; then
    python3 -m http.server 8080 --directory . &
    echo "🌐 Interface web démarrée sur: http://localhost:8080/AEQUITAS_CONTROL_CENTER.html"
elif command -v python &> /dev/null; then
    python -m http.server 8080 --directory . &
    echo "🌐 Interface web démarrée sur: http://localhost:8080/AEQUITAS_CONTROL_CENTER.html"
else
    echo "❌ Python non trouvé. Veuillez installer Python ou ouvrir manuellement AEQUITAS_CONTROL_CENTER.html"
fi

echo ""
echo "🎯 Installation terminée !"
echo "📖 Documentation: CONTRIBUTION_REPORT.md"
echo "💬 Support: https://github.com/moinecalixte-cyber/aequitas/issues"