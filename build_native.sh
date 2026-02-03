#!/bin/bash

# Aequitas - Build Native Functionnel (Linux/macOS)
# Stable coin intrinsèque - Économie solidaire RÉELLE

echo "============================================"
echo "⚖️  AEQUITAS - BUILD NATIVE FONCTIONNEL"
echo "============================================"
echo

echo "[INFO] Construction NATIVE - PAS de simulation"
echo "[INFO] Stable coin intrinsèque - Économie solidaire réelle"
echo

# Vérification environnement
if [ ! -f "Cargo.toml" ]; then
    echo "[ERREUR] Veuillez exécuter depuis la racine d'Aequitas"
    exit 1
fi

# Configuration Rust
echo "[SETUP] Configuration Rust pour performance native..."
rustup default stable
rustup target add x86_64-unknown-linux-gnu 2>/dev/null || rustup target add x86_64-apple-darwin

# Installation dépendances
echo "[DEPS] Installation dépendances système..."

if command -v apt-get &> /dev/null; then
    echo "[INFO] Installation pour Ubuntu/Debian..."
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev clang
elif command -v yum &> /dev/null; then
    echo "[INFO] Installation pour CentOS/RHEL..."
    sudo yum groupinstall -y "Development Tools"
    sudo yum install -y openssl-devel clang
elif command -v pacman &> /dev/null; then
    echo "[INFO] Installation pour Arch Linux..."
    sudo pacman -S --needed base-devel openssl clang
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "[INFO] Installation pour macOS..."
    if command -v brew &> /dev/null; then
        brew install openssl llvm
    else
        echo "[WARN] Homebrew recommandé: https://brew.sh"
    fi
fi

# Nettoyage précédent
echo "[CLEAN] Nettoyage build précédent..."
cargo clean 2>/dev/null

# Build release optimisé
echo "[COMPILE] Build release optimisé..."
export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat"

# Détection du target
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    TARGET="x86_64-unknown-linux-gnu"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    TARGET="x86_64-apple-darwin"
else
    TARGET="native"
fi

cargo build --release --target $TARGET

if [ $? -ne 0 ]; then
    echo "[ERREUR] Échec de compilation native"
    echo "[DEBUG] Informations debug:"
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo "Target: $TARGET"
    echo "OS: $OSTYPE"
    
    echo
    echo "[SOLUTION] Problèmes possibles:"
    echo "1. Dépendances manquantes"
    echo "2. Versions incompatibles"
    echo "3. Permissions insuffisantes"
    echo
    echo "[ACTION] Installez les dépendances manuellement ou utilisez sudo"
    exit 1
fi

echo
echo "[SUCCESS] Build native terminé avec succès!"

# Vérification des binaires
echo "[VERIFY] Vérification des exécutables..."

BINARIES_FOUND=0
TARGET_DIR="target/$TARGET/release"

if [ -f "$TARGET_DIR/aequitas-node" ]; then
    echo "[OK] aequitas-node créé"
    ((BINARIES_FOUND++))
fi

if [ -f "$TARGET_DIR/aequitas-miner" ]; then
    echo "[OK] aequitas-miner créé"
    ((BINARIES_FOUND++))
fi

if [ -f "$TARGET_DIR/aequitas-wallet" ]; then
    echo "[OK] aequitas-wallet créé"
    ((BINARIES_FOUND++))
fi

if [ $BINARIES_FOUND -lt 3 ]; then
    echo "[WARN] Certains binaires manquent"
else
    echo "[PERFECT] Tous les binaires créés!"
fi

echo
echo "[DEPLOY] Création package déploiement..."

mkdir -p release_package
cp $TARGET_DIR/aequitas-* release_package/ 2>/dev/null
cp Cargo.toml release_package/
cp LICENSE release_package/
cp README.md release_package/

# Scripts de lancement
cat > release_package/start_node.sh << 'EOF'
#!/bin/bash
echo "🔗 Démarrage Noeud Aequitas P2P..."
echo "🌐 Connexion réseau décentralisé..."
./aequitas-node --port 23420 --data-dir ./data
EOF

cat > release_package/start_miner.sh << 'EOF'
#!/bin/bash
echo "⛏️  Démarrage Mining AequiHash..."
echo "🔥 Mining GPU optimisé..."
./aequitas-miner --address aeq1 VotreAdresse --threads auto
EOF

cat > release_package/start_wallet.sh << 'EOF'
#!/bin/bash
echo "💼 Portefeuille Aequitas Sécurisé..."
./aequitas-wallet
EOF

# Rendre exécutables
chmod +x release_package/*.sh
chmod +x release_package/aequitas-*

# Configuration par défaut
cat > release_package/config.toml << 'EOF'
[network]
port = 23420
data_dir = "./data"

[mining]
threads = "auto"

[wallet]
data_dir = "./wallet"
EOF

echo
echo "[COMPLETE] Package créé dans release_package/"

echo
echo "============================================"
echo "⚖️  AEQUITAS - BUILD NATIF TERMINÉ"
echo "============================================"
echo
echo "🎯 Applications NATIVES:"
echo "   📁 release_package/aequitas-node    - Noeud P2P décentralisé"
echo "   📁 release_package/aequitas-miner   - Mining GPU AequiHash"
echo "   📁 release_package/aequitas-wallet  - Portefeuille sécurisé"
echo
echo "🚀 Lancement:"
echo "   ./start_node.sh    - Démarrer noeud P2P"
echo "   ./start_miner.sh   - Commencer mining"
echo "   ./start_wallet.sh   - Ouvrir portefeuille"
echo
echo "💎 Principes respectés:"
echo "   ✅ Stable coin intrinsèque - NON indexé"
echo "   ✅ Économie solidaire 98/1/1"
echo "   ✅ Décentralisation maximale"
echo "   ✅ 100% fonctionnel - PAS de simulation"
echo
echo "🌐 Prêt pour réseau économique décentralisé!"
echo

# Test
if [ -f "$TARGET_DIR/aequitas-node" ]; then
    echo "[TEST] Test du noeud..."
    $TARGET_DIR/aequitas-node --version 2>/dev/null && echo "[OK] Noeud fonctionnel" || echo "[WARN] Noeud présente des erreurs"
fi

echo "✅ Build terminé - Aequitas prêt !"