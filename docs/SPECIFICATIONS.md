# 📖 Spécifications Techniques - Aequitas (AEQ)

## 1. Vue d'ensemble

**Aequitas** est une cryptomonnaie décentralisée conçue pour une distribution équitable de la création monétaire, avec un accent particulier sur l'accessibilité aux petits mineurs GPU.

### Objectifs principaux :
- ✅ Résistance aux ASIC
- ✅ Distribution équitable
- ✅ Décentralisation maximale
- ✅ Résilience (non-indexée sur fiat)

---

## 2. Paramètres de la blockchain

| Paramètre | Valeur | Justification |
|-----------|--------|---------------|
| **Nom** | Aequitas | Latin pour "équité" |
| **Symbole** | AEQ | Court et mémorable |
| **Algorithme** | AequiHash | Dérivé KawPoW optimisé |
| **Temps de bloc** | 30 secondes | Équilibre finalité/orphans |
| **Récompense initiale** | 50 AEQ | Comparable à Bitcoin |
| **Halving** | 2,100,000 blocs (~2 ans) | Prévisibilité |
| **Supply maximum** | 210,000,000 AEQ | Rareté programmée |
| **Trésorerie** | 2% par bloc | Développement communautaire |
| **Décimales** | 9 | Précision suffisante |

---

## 3. Algorithme AequiHash

### 3.1 Principes de conception

AequiHash est un algorithme de hachage Proof-of-Work conçu pour :
1. **Favoriser les GPU grand public** (RTX 3060, 6GB VRAM)
2. **Résister aux ASIC** via opérations mémoire-hard
3. **Varier par epoch** pour empêcher l'optimisation statique

### 3.2 Structure de l'algorithme

```
┌─────────────────────────────────────────────────────────┐
│                    AEQUIHASH                            │
├─────────────────────────────────────────────────────────┤
│  1. Seed Generation (Keccak256)                         │
│     └─ Génère seed depuis epoch number                  │
│                                                         │
│  2. Cache Generation (RandMemoHash)                     │
│     └─ 64 MB cache depuis seed                          │
│                                                         │
│  3. DAG Generation (pour GPU)                           │
│     └─ 4 GB DAG depuis cache                            │
│                                                         │
│  4. Mix Loop (64 rounds)                                │
│     └─ Opérations variables par epoch                   │
│     └─ 64 accès DAG pseudo-aléatoires                   │
│                                                         │
│  5. Final Hash (Blake3)                                 │
│     └─ Compression du mix + header + nonce              │
└─────────────────────────────────────────────────────────┘
```

### 3.3 Opérations du Mix Loop

Les opérations varient à chaque epoch :

| Op | Nom | Description |
|----|-----|-------------|
| 0 | ADD | Addition modulaire |
| 1 | MUL | Multiplication modulaire |
| 2 | SUB | Soustraction modulaire |
| 3 | XOR | OU exclusif |
| 4 | ROTL | Rotation gauche |
| 5 | ROTR | Rotation droite |
| 6 | AND | ET logique |
| 7 | OR | OU logique |

### 3.4 Paramètres mémoire

| Paramètre | Valeur | Cible |
|-----------|--------|-------|
| DAG Size | 4 GB | RTX 3060 (6GB) |
| Cache Size | 64 MB | Vérification light |
| Epoch Length | 240 blocs (~2h) | Changement DAG |
| Mix Rounds | 64 | Sécurité |
| Dataset Accesses | 64 | Mémoire-hard |

---

## 4. Économie monétaire

### 4.1 Courbe d'émission

```
Année  | Blocs/an     | Récompense | Émission annuelle | Total cumulé
-------|--------------|------------|-------------------|-------------
1      | 1,051,200    | 50 AEQ     | 52,560,000 AEQ    | 52,560,000
2      | 1,051,200    | 50 AEQ     | 52,560,000 AEQ    | 105,120,000
3      | 1,051,200    | 25 AEQ     | 26,280,000 AEQ    | 131,400,000
4      | 1,051,200    | 25 AEQ     | 26,280,000 AEQ    | 157,680,000
5      | 1,051,200    | 12.5 AEQ   | 13,140,000 AEQ    | 170,820,000
...
```

### 4.2 Distribution

- **98%** → Mineurs (récompense de bloc)
- **2%** → Trésorerie communautaire

### 4.3 Trésorerie

La trésorerie est contrôlée par gouvernance on-chain :
- Vote sur les propositions de dépense
- Financement du développement
- Audits de sécurité
- Infrastructure communautaire

---

## 5. Transactions

### 5.1 Format

```rust
struct Transaction {
    version: u32,           // Version du format (1)
    tx_type: TxType,        // Transfer | Coinbase | Vote | Proposal
    inputs: Vec<TxInput>,   // UTXOs consommés
    outputs: Vec<TxOutput>, // Nouvelles sorties
    timestamp: i64,         // Horodatage
    memo: Vec<u8>,          // Données (max 256 bytes)
}
```

### 5.2 Signatures

- **Algorithme** : Ed25519
- **Clé publique** : 32 bytes
- **Signature** : 64 bytes

### 5.3 Frais

- **Minimum** : 0.000001 AEQ (1000 unités)
- **Par byte** : ~10 unités
- **Politique** : First-price, ordre par frais/byte

---

## 6. Adresses

### 6.1 Format

```
aeq1<base58(bytes[20] + checksum[4])>
```

Exemple : `aeq1Qm3nVzKL7xPdF9jR2sY4wT6hA8bC`

### 6.2 Dérivation

```
PublicKey (32 bytes)
    ↓ Keccak256
Hash (32 bytes)
    ↓ Prendre bytes [12:32]
Address bytes (20 bytes)
    ↓ Keccak256
Checksum (4 bytes)
    ↓ Base58 + préfixe
Address string
```

---

## 7. Réseau P2P

### 7.1 Protocole

- **Transport** : TCP/IP sur libp2p
- **Chiffrement** : Noise Protocol
- **Multiplexage** : Yamux
- **Découverte** : mDNS + Gossipsub

### 7.2 Ports

| Port | Usage |
|------|-------|
| 23420 | P2P mainnet |
| 23421 | RPC API |
| 33420 | P2P testnet |
| 33421 | RPC testnet |

### 7.3 Messages

| Message | Description |
|---------|-------------|
| Handshake | Établissement connexion |
| GetHeaders | Demande headers |
| Headers | Réponse headers |
| GetBlocks | Demande blocs |
| Blocks | Réponse blocs |
| NewBlock | Annonce nouveau bloc |
| NewTransactions | Annonce transactions |
| Ping/Pong | Keep-alive |

---

## 8. Sécurité

### 8.1 Attaque 51%

Le coût d'une attaque 51% est élevé grâce à :
- **Mémoire-hard** : Nécessite 4GB VRAM par GPU
- **Distribution** : Pas de concentration ASIC
- **Coût électrique** : Pas d'avantage d'efficacité

### 8.2 Résistance ASIC

- **Variation d'epoch** : Algorithme change toutes les 2h
- **Opérations mixtes** : Pas d'optimisation unique
- **Accès mémoire** : Limité par bande passante

### 8.3 Bonnes pratiques

- Attendre 20+ confirmations pour transactions importantes
- Vérifier les mises à jour de sécurité
- Utiliser des mots de passe forts pour les wallets

---

## 9. Gouvernance

### 9.1 Propositions

```
Proposal {
    id: u64,
    title: String,
    description: String,
    amount: Option<u64>,     // Pour dépenses trésorerie
    voting_start: u64,       // Hauteur de bloc
    voting_end: u64,
    proposer: Address,
}
```

### 9.2 Votes

- **Pondération** : 1 AEQ = 1 vote (avec caps anti-whale)
- **Quorum** : 10% de la supply votante
- **Seuil** : 66% pour les propositions de dépense
- **Durée** : 7 jours (20,160 blocs)

### 9.3 Vote quadratique (futur)

Pour limiter l'influence des gros détenteurs :
```
Pouvoir de vote = √(AEQ détenus)
```

---

## 10. Feuille de route

### Phase 1 : Testnet (T1 2026)
- [x] Core blockchain
- [x] Algorithme AequiHash
- [x] Mineur CPU
- [ ] Nœud complet
- [ ] Wallet CLI

### Phase 2 : MainNet (T2 2026)
- [ ] Audit de sécurité
- [ ] Mineur GPU (CUDA/OpenCL)
- [ ] Wallet desktop
- [ ] Explorateur de blocs

### Phase 3 : Écosystème (T3-T4 2026)
- [ ] Pools de minage
- [ ] Intégrations exchanges
- [ ] Gouvernance on-chain
- [ ] Mobile wallet

---

## Annexes

### A. Compilation

```bash
# Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Cloner et compiler
git clone https://github.com/aequitas-coin/aequitas.git
cd aequitas
cargo build --release
```

### B. Configuration minimale

| Composant | Minimum | Recommandé |
|-----------|---------|------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 16 GB |
| GPU VRAM | 6 GB | 8+ GB |
| Stockage | 50 GB SSD | 100+ GB NVMe |
| Réseau | 10 Mbps | 100+ Mbps |

### C. GPU supportés

| Marque | Modèles |
|--------|---------|
| NVIDIA | RTX 3060+, RTX 20xx+, GTX 16xx |
| AMD | RX 6600+, RX 5000+ |

---

*Document version 1.0 - Février 2026*
