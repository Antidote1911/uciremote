# uci-remote (Rust)

Deux outils pour faire tourner un moteur d'échecs UCI sur une machine distante.

- **`uciserver`** — tourne sur la machine Linux qui possède le moteur
- **`uciproxy`** — tourne sur le poste client (Windows ou Linux), se comporte comme un moteur local

---

## Build

```bash
cargo build --release
```

Binaires produits :
- `target/release/uciserver`
- `target/release/uciproxy` (ou `.exe` sur Windows)

Cross-compilation pour Windows depuis Linux :
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

---

## uciserver

Lance le moteur UCI et expose son interface sur un port TCP.
Chaque connexion cliente démarre une instance du moteur.

```
uciserver [ip]:port /chemin/vers/moteur
```

**Exemples :**

```bash
# Écoute sur toutes les interfaces, port 7900
uciserver :7900 stockfish

# Interface spécifique
uciserver 192.168.1.10:7900 /usr/games/stockfish

# Plusieurs moteurs sur des ports différents (script)
uciserver :7900 /usr/bin/stockfish &
uciserver :7901 /usr/bin/lc0 &
```

---

## uciproxy

Se connecte à un `uciserver` et simule un moteur UCI local.
Le logiciel d'échecs (Arena, CuteChess, etc.) n'y voit que du feu.

```
uciproxy ip:port
```

**Exemples :**

```bash
# Connexion directe
uciproxy 192.168.2.30:7900
```

**Mode proxy nommé** (sans argument) — pour les logiciels qui ne passent pas d'argument au moteur :

```bash
# Linux
cp uciproxy remote_stockfish
echo "192.168.2.30:7900" > remote_stockfish.txt
./remote_stockfish

# Windows
copy uciproxy.exe remote_stockfish.exe
echo 192.168.2.30:7900 > remote_stockfish.txt
remote_stockfish.exe
```

---

## Cas d'usage typique

**Serveur Linux 192.168.2.30 :**
```bash
uciserver :7900 /usr/bin/stockfish &
uciserver :7901 /usr/bin/komodo &
```

**PC client Windows :**
```cmd
copy uciproxy.exe remote_stockfish.exe
echo 192.168.2.30:7900 > remote_stockfish.txt

copy uciproxy.exe remote_komodo.exe
echo 192.168.2.30:7901 > remote_komodo.txt
```

Ajouter `remote_stockfish.exe` et `remote_komodo.exe` dans Arena/CuteChess comme n'importe quel moteur local.

---

## Architecture

```
[Logiciel d'échecs]
       │  stdin/stdout
  [uciproxy]
       │  TCP socket (lignes texte UCI)
  [uciserver]
       │  stdin/stdout
  [Moteur UCI (stockfish, lc0…)]
```

Le protocole de transport est trivial : des lignes de texte UCI brut, sans encapsulation.
