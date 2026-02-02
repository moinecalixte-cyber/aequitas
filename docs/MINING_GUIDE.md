# 🚀 Guide de démarrage rapide - Minage Aequitas

## 📋 Prérequis

- **GPU** : NVIDIA RTX 3060 ou supérieur (6+ GB VRAM)
- **RAM** : 16 GB minimum
- **OS** : Windows 10/11 ou Linux
- **Rust** : 1.75+ (pour compilation)

---

## 🔧 Installation

### Option 1 : Télécharger le binaire (recommandé)

```bash
# Windows
powershell -Command "Invoke-WebRequest -Uri 'https://github.com/aequitas-coin/releases/latest/aequitas-miner-win64.zip' -OutFile 'aequitas-miner.zip'"
Expand-Archive aequitas-miner.zip -DestinationPath .

# Linux
wget https://github.com/aequitas-coin/releases/latest/aequitas-miner-linux64.tar.gz
tar xzf aequitas-miner-linux64.tar.gz
```

### Option 2 : Compiler depuis les sources

```bash
# Cloner le projet
git clone https://github.com/aequitas-coin/aequitas.git
cd aequitas-coin

# Compiler (release optimisé)
cargo build --release

# Les binaires sont dans target/release/
```

---

## 💳 Créer un wallet

### 1. Générer une nouvelle adresse

```bash
# Créer un nouveau wallet avec mot de passe
aequitas-wallet new --password "VotreMotDePasseFort123!"

# Sortie :
# ✓ Nouvelle adresse créée : aeq1Qm3nVzKL7xPdF9jR2sY4wT6hA8bCdEfG
# ✓ Wallet sauvegardé : wallet.json
# 
# ⚠️ IMPORTANT : Notez votre adresse et gardez votre mot de passe en sécurité !
```

### 2. Sauvegarder votre wallet

```bash
# Exporter la clé privée (GARDER SECRET !)
aequitas-wallet export --password "VotreMotDePasse"

# Backup le fichier wallet.json dans un endroit sûr
```

---

## ⛏️ Configuration du mineur

### 1. Initialiser la configuration

```bash
aequitas-miner init
```

Ceci crée un fichier `miner.toml` :

```toml
# Aequitas Miner Configuration
# ================================

# Votre adresse Aequitas (OBLIGATOIRE)
address = "aeq1VotreAdresseIci"

# URL du nœud (solo mining)
node_url = "http://127.0.0.1:23421"

# Threads CPU (0 = désactiver CPU)
cpu_threads = 4

# Activer le minage GPU
gpu_enabled = true

# GPUs à utiliser (vide = tous)
gpu_devices = []

# Intensité GPU (1-100)
# RTX 3060 recommandé : 70-80
gpu_intensity = 75

# Nom du worker
worker_name = "mon-pc"

# Pool mining (optionnel)
stratum_enabled = false
# stratum_url = "stratum+tcp://pool.aequitas.network:3333"

# Niveau de log : trace, debug, info, warn, error
log_level = "info"
```

### 2. Modifier la configuration

Éditez `miner.toml` et ajoutez votre adresse wallet.

---

## 🏃 Lancer le minage

### Solo Mining (avec votre propre nœud)

```bash
# Démarrer le nœud en premier
aequitas-node &

# Lancer le mineur
aequitas-miner mine
```

### Pool Mining (recommandé pour commencer)

```bash
# Éditer miner.toml
stratum_enabled = true
stratum_url = "stratum+tcp://pool.aequitas.network:3333"

# Lancer
aequitas-miner mine
```

### Options en ligne de commande

```bash
# Spécifier l'adresse directement
aequitas-miner mine --address aeq1VotreAdresse

# Limiter les threads CPU
aequitas-miner mine --threads 2

# Désactiver GPU
aequitas-miner mine --no-gpu

# Utiliser une config différente
aequitas-miner mine --config /chemin/vers/config.toml
```

---

## 📊 Comprendre les statistiques

```
⛏️  Mining started! Press Ctrl+C to stop.

[INFO] New block template at height 12345
[INFO] Hashrate: 45.23 KH/s | Total: 1,234,567 hashes
[INFO] Hashrate: 46.01 KH/s | Total: 1,694,567 hashes
[INFO] 🎉 Solution found! Nonce: 8472936
[INFO] ✓ Solution accepted!
```

| Métrique | Description |
|----------|-------------|
| **Hashrate** | Vitesse de calcul (H/s, KH/s, MH/s) |
| **Total hashes** | Nombre total de hashes calculés |
| **Height** | Hauteur du bloc en cours de minage |
| **Solution found** | Vous avez trouvé un bloc ! |

---

## 🔥 Optimisation pour RTX 3060

### Paramètres recommandés

```toml
# miner.toml optimisé pour RTX 3060
gpu_enabled = true
gpu_intensity = 75           # Ne pas dépasser 80
cpu_threads = 2              # Laisser des cores pour le GPU
```

### Contrôle de la température

- **Cible** : 70°C max
- **Si trop chaud** : Réduire `gpu_intensity` à 60-70
- **Ventilation** : Assurez une bonne circulation d'air

### Overclocking (optionnel)

Avec MSI Afterburner :
- **Memory Clock** : +500 à +1000 MHz
- **Power Limit** : 70-80%
- **Core Clock** : +0 à +100 MHz

---

## 🔧 Dépannage

### "GPU not detected"

```bash
# Vérifier les drivers NVIDIA
nvidia-smi

# Si manquant, installer les derniers drivers NVIDIA
```

### "Failed to connect to node"

```bash
# Vérifier que le nœud tourne
aequitas-node status

# Ou utiliser un pool
stratum_enabled = true
```

### "Out of memory"

```bash
# Réduire l'intensité GPU
gpu_intensity = 50

# Ou désactiver le GPU et utiliser CPU
gpu_enabled = false
cpu_threads = 8
```

### Hashrate trop faible

1. Vérifier que le GPU est bien utilisé
2. Augmenter `gpu_intensity`
3. Fermer les autres applications
4. Vérifier la température (throttling?)

---

## 📈 Estimation des gains

Avec une RTX 3060 typique (~45 KH/s) :

| Hashrate réseau | Blocs/jour (estimé) | AEQ/jour |
|-----------------|---------------------|----------|
| 1 MH/s | ~1.3 | ~65 AEQ |
| 10 MH/s | ~0.13 | ~6.5 AEQ |
| 100 MH/s | ~0.013 | ~0.65 AEQ |

*Note : Les gains réels dépendent de la difficulté du réseau*

---

## 🤝 Rejoindre la communauté

- **Discord** : [discord.gg/aequitas](#)
- **Telegram** : [@AequitasCoin](#)
- **GitHub** : [github.com/aequitas-coin](#)
- **Forums** : [forum.aequitas.network](#)

---

## ⚠️ Sécurité

1. **Ne partagez JAMAIS** votre clé privée
2. **Sauvegardez** votre wallet.json
3. **Vérifiez** les URLs des pools
4. **Téléchargez** uniquement depuis les sources officielles

---

**Bon minage ! ⛏️🌟**
