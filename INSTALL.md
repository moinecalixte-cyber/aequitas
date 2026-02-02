# 📦 Instructions d'installation

## Prérequis

### 1. Installer Rust

**Windows (PowerShell):**
```powershell
irm https://sh.rustup.rs | iex
```

**Ou téléchargez depuis:** https://rustup.rs

**Linux/Mac:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Après l'installation, redémarrez votre terminal et vérifiez:
```bash
rustc --version
cargo --version
```

---

## Compilation

### Windows
```batch
cd aequitas-coin
scripts\build.bat
```

### Linux/Mac
```bash
cd aequitas-coin
chmod +x scripts/build.sh
./scripts/build.sh
```

---

## Lancer le mineur

### 1. Configurer
```bash
# Copier la config exemple
copy miner.toml.example miner.toml

# Éditer et ajouter votre adresse wallet
notepad miner.toml
```

### 2. Démarrer
```bash
# Windows
start_mining.bat

# Ou manuellement
bin\aequitas-miner mine --address aeq1VotreAdresse
```

---

## Structure du projet

```
aequitas-coin/
├── src/
│   ├── core/           # Blockchain core
│   │   ├── block.rs    # Blocs et headers
│   │   ├── transaction.rs # Transactions
│   │   ├── blockchain.rs # État blockchain
│   │   ├── address.rs  # Gestion adresses
│   │   ├── merkle.rs   # Arbres merkle
│   │   └── difficulty.rs # Ajustement difficulté
│   │
│   ├── consensus/      # Algorithme AequiHash
│   │   ├── aequihash.rs # Algorithme principal
│   │   ├── dag.rs      # Génération DAG
│   │   └── pow.rs      # Proof of Work
│   │
│   ├── network/        # Réseau P2P
│   │   ├── node.rs     # Nœud réseau
│   │   ├── messages.rs # Messages P2P
│   │   └── peer.rs     # Gestion peers
│   │
│   ├── wallet/         # Portefeuille
│   │   ├── keystore.rs # Stockage clés
│   │   ├── wallet.rs   # Interface wallet
│   │   └── builder.rs  # Construction TX
│   │
│   └── miner/          # Mineur GPU/CPU
│       ├── main.rs     # Point d'entrée
│       ├── config.rs   # Configuration
│       ├── worker.rs   # Workers minage
│       ├── stats.rs    # Statistiques
│       └── stratum.rs  # Support pool
│
├── docs/               # Documentation
│   ├── SPECIFICATIONS.md
│   └── MINING_GUIDE.md
│
├── scripts/            # Scripts utilitaires
│   ├── build.bat
│   └── build.sh
│
├── Cargo.toml          # Config Rust
├── README.md           # Documentation principale
├── LICENSE             # Licence MIT
└── miner.toml.example  # Config exemple
```

---

## Dépannage

### "cargo not found"
→ Rust n'est pas installé. Exécutez `scripts/install_rust.bat`

### Erreurs de compilation
→ Mettez à jour Rust: `rustup update`

### GPU non détecté
→ Le support GPU CUDA/OpenCL sera ajouté dans une future version
→ Utilisez le minage CPU pour l'instant

---

## Prochaines étapes

1. ✅ Core blockchain implémenté
2. ✅ Algorithme AequiHash créé
3. ✅ Mineur CPU fonctionnel
4. ⏳ Mineur GPU (CUDA/OpenCL)
5. ⏳ Nœud complet P2P
6. ⏳ Wallet graphique
7. ⏳ Tests et audits

---

**Questions?** Ouvrez une issue sur GitHub!
