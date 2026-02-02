#!/bin/bash
# ============================================
#   Aequitas - Script de compilation Linux
# ============================================

set -e

echo ""
echo "========================================"
echo "    AEQUITAS BUILD SCRIPT"
echo "========================================"
echo ""

# Vérifier Rust
if ! command -v cargo &> /dev/null; then
    echo "[ERREUR] Rust n'est pas installé!"
    echo "Installez Rust avec: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "[1/4] Vérification de Rust..."
cargo --version

echo ""
echo "[2/4] Nettoyage des anciens builds..."
cargo clean

echo ""
echo "[3/4] Compilation en mode release..."
echo "     Cela peut prendre plusieurs minutes..."
echo ""

cargo build --release

echo ""
echo "[4/4] Copie des binaires..."

mkdir -p bin
cp -f target/release/aequitas-miner bin/ 2>/dev/null || true

echo ""
echo "========================================"
echo "    COMPILATION TERMINÉE !"
echo "========================================"
echo ""
echo "Binaires disponibles dans : bin/"
echo ""
echo "Pour démarrer le minage :"
echo "  1. Éditez miner.toml avec votre adresse"
echo "  2. Lancez: ./bin/aequitas-miner mine"
echo ""
