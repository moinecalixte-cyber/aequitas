# 🌟 AEQUITAS (AEQ)

> **"Fair Mining for Everyone"** - Une cryptomonnaie décentralisée conçue pour une redistribution équitable de la création monétaire.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)

---

## 🎯 Vision

**Aequitas** (latin pour "équité") est une cryptomonnaie:
- ✅ **Résistante aux ASIC** - Optimisée pour GPU grand public (RTX 3060+)
- ✅ **Distribution équitable** - Zéro pré-minage, 98% pour les mineurs
- ✅ **Décentralisée** - Aucun point de contrôle central
- ✅ **Résiliente** - Non indexée sur les monnaies fiat

---

## 📊 Spécifications

| Paramètre | Valeur |
|-----------|--------|
| **Algorithme** | AequiHash (GPU-friendly) |
| **Temps de bloc** | 30 secondes |
| **Récompense initiale** | 50 AEQ |
| **Halving** | ~2 ans (2,100,000 blocs) |
| **Supply maximum** | 210,000,000 AEQ |
| **Trésorerie** | 2% par bloc |
| **VRAM minimum** | 6 GB |

---

## 🚀 Démarrage rapide

### Prérequis
- [Rust](https://rustup.rs) 1.75+
- GPU avec 6+ GB VRAM (recommandé)

### Installation

```bash
# Cloner le projet
git clone https://github.com/aequitas-coin/aequitas.git
cd aequitas

# Compiler
cargo build --release

# Les binaires sont dans target/release/
```

### Lancer un nœud

```bash
# Initialiser la configuration
./target/release/aequitas-node init

# Démarrer le nœud
./target/release/aequitas-node run
```

### Créer un wallet

```bash
# Créer un nouveau wallet
./target/release/aequitas-wallet new --password "VotreMotDePasse"

# Lister les adresses
./target/release/aequitas-wallet list
```

### Commencer à miner

```bash
# Copier la configuration exemple
cp miner.toml.example miner.toml

# Éditer et ajouter votre adresse
# address = "aeq1VotreAdresse"

# Lancer le mineur
./target/release/aequitas-miner mine
```

---

## 📁 Structure du projet

```
aequitas/
├── src/
│   ├── core/       # Blockchain core (blocs, transactions, UTXO)
│   ├── consensus/  # Algorithme AequiHash
│   ├── network/    # Réseau P2P
│   ├── wallet/     # Portefeuille
│   ├── miner/      # Mineur GPU/CPU
│   └── node/       # Nœud complet
├── docs/           # Documentation
└── scripts/        # Scripts utilitaires
```

---

## � Documentation

- [Guide de minage](docs/MINING_GUIDE.md) - Configuration RTX 3060
- [Spécifications techniques](docs/SPECIFICATIONS.md) - Détails de l'algorithme
- [Instructions d'installation](INSTALL.md) - Installation complète

---

## 🔧 Configuration minimale

| Composant | Minimum | Recommandé |
|-----------|---------|------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 16 GB |
| GPU VRAM | 6 GB | 8+ GB |
| Stockage | 50 GB SSD | 100+ GB NVMe |

### GPU supportés
- NVIDIA: RTX 3060+, RTX 20xx+, GTX 16xx
- AMD: RX 6600+, RX 5000+

---

## 🤝 Contribuer

Aequitas est un projet communautaire ouvert!

1. Fork le projet
2. Créez votre branche (`git checkout -b feature/amazing`)
3. Commit (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing`)
5. Ouvrez une Pull Request

---

## 📜 Licence

[MIT License](LICENSE) - Libre d'utilisation, modification et distribution.

---

## 🌐 Communauté

- **GitHub**: [github.com/aequitas-coin/aequitas](https://github.com/aequitas-coin/aequitas)
- **Discord**: [Bientôt disponible]
- **Twitter**: [Bientôt disponible]

---

**Aequitas** - Une monnaie équitable pour tous. ⚖️🌍
