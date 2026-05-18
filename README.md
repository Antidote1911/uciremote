# uci-remote

> 🇫🇷 [Français](#français) | 🇬🇧 [English](#english)

---

# Français

Deux outils pour faire tourner un moteur d'échecs UCI sur une machine distante.

- **`uciserver`** — tourne sur la machine serveur (Linux) qui possède le moteur UCI
- **`uciproxy`** — tourne sur le poste client (Windows ou Linux), se comporte comme un moteur local aux yeux du logiciel d'échecs

## Architecture

```
[Logiciel d'échecs (Arena, CuteChess, Droidfish...)]
              │  stdin / stdout
          [uciproxy]
              │  TCP socket — lignes de texte UCI brut
          [uciserver]
              │  stdin / stdout
    [Moteur UCI (Stockfish, Dragon, lc0...)]
```

---

## Build

```bash
cargo build --release
```

Binaires produits :
- `target/release/uciserver`
- `target/release/uciproxy` (ou `.exe` sur Windows)

**Cross-compilation uciproxy pour Windows depuis Linux :**
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release -p uciproxy --target x86_64-pc-windows-gnu
```

---

## uciserver

Lance le moteur UCI et expose son interface sur un port TCP.
Chaque connexion cliente démarre une instance indépendante du moteur.

### Syntaxe

```
uciserver --port <PORT> --engine <CHEMIN_MOTEUR> [--host <IP>]
uciserver -p <PORT> -e <CHEMIN_MOTEUR>
```

### Options

| Option | Court | Défaut | Description |
|--------|-------|--------|-------------|
| `--port` | `-p` | requis | Port TCP à écouter |
| `--engine` | `-e` | requis | Chemin vers le moteur UCI |
| `--host` | | `0.0.0.0` | Interface réseau (toutes par défaut) |

### Exemples

```bash
# Écoute sur toutes les interfaces, port 8100, moteur stockfish dans le PATH
uciserver --port 8100 --engine stockfish

# Forme courte
uciserver -p 8100 -e stockfish

# Interface réseau spécifique
uciserver -p 8100 -e /usr/bin/stockfish --host 192.168.1.10

# Chemin avec espaces (guillemets nécessaires)
uciserver -p 8100 -e "/usr/games/stockfish 17"

# Plusieurs moteurs en parallèle (script bash)
uciserver -p 8100 -e /usr/bin/stockfish &
uciserver -p 8200 -e /usr/bin/dragon &

# Aide et version
uciserver --help
uciserver --version
```

---

## uciproxy

Se connecte à un `uciserver` distant et simule un moteur UCI local.
Le logiciel d'échecs ne fait pas la différence avec un moteur local.

### Syntaxe

```
uciproxy --server <IP:PORT> [--retry <SECONDES>]
uciproxy -s <IP:PORT>
```

### Options

| Option | Court | Défaut | Description |
|--------|-------|--------|-------------|
| `--server` | `-s` | optionnel* | Adresse du serveur `ip:port` |
| `--retry` | `-r` | `5` | Délai de reconnexion en secondes (0 = désactivé) |

*Si `--server` est absent, uciproxy cherche un fichier `<nom_du_binaire>.txt` (voir mode proxy nommé ci-dessous).

### Exemples

```bash
# Connexion directe à un uciserver
uciproxy --server 192.168.1.65:8100
uciproxy -s 192.168.1.65:8100

# Reconnexion toutes les 10 secondes si la connexion tombe
uciproxy -s 192.168.1.65:8100 --retry 10

# Pas de reconnexion automatique
uciproxy -s 192.168.1.65:8100 --retry 0

# Aide et version
uciproxy --help
uciproxy --version
```

### Mode proxy nommé (sans argument)

De nombreux logiciels d'échecs ne permettent pas de passer des arguments au moteur.
Dans ce cas, copier l'exécutable sous le nom souhaité et créer un fichier `.txt` associé.

**Linux :**
```bash
cp uciproxy remote_stockfish
echo "192.168.1.65:8100" > remote_stockfish.txt
./remote_stockfish
```

**Windows :**
```cmd
copy uciproxy.exe remote_stockfish.exe
echo 192.168.1.65:8100 > remote_stockfish.txt
remote_stockfish.exe
```

uciproxy cherche automatiquement `remote_stockfish.txt` (même dossier, même nom, extension `.txt`) et se connecte à l'adresse qu'il contient.

### Reconnexion automatique

Par défaut (`--retry 5`), si le serveur se déconnecte (redémarrage d'Unraid, coupure réseau...), uciproxy attend 5 secondes et retente la connexion automatiquement, sans que le logiciel d'échecs le remarque.

---

## Cas d'usage complet

### Serveur Unraid / Linux (192.168.1.65)

Via Docker Compose (voir [server-docker-uciengines](https://github.com/Antidote1911/server-docker-uciengines)) :
```bash
docker compose up -d
# Stockfish sur le port 8100
# Dragon sur le port 8200
```

Ou directement :
```bash
uciserver -p 8100 -e /usr/bin/stockfish &
uciserver -p 8200 -e /usr/bin/dragon &
```

### Client Windows (CuteChess, Arena, Chessbase)

```cmd
copy uciproxy.exe remote_stockfish.exe
echo 192.168.1.65:8100 > remote_stockfish.txt

copy uciproxy.exe remote_dragon.exe
echo 192.168.1.65:8200 > remote_dragon.txt
```

Ajouter `remote_stockfish.exe` et `remote_dragon.exe` dans le logiciel d'échecs comme des moteurs locaux normaux.

### Client Android (Droidfish)

Droidfish supporte nativement les moteurs TCP :
`Paramètres → Moteur réseau → 192.168.1.65:8100`

Pour accéder depuis l'extérieur du réseau local, utiliser **Tailscale** :
`Paramètres → Moteur réseau → 100.70.128.103:8100`

---

---

# English

Two tools to run a UCI chess engine on a remote machine.

- **`uciserver`** — runs on the server machine (Linux) that hosts the UCI engine
- **`uciproxy`** — runs on the client machine (Windows or Linux), appears as a local engine to the chess software

## Architecture

```
[Chess software (Arena, CuteChess, Droidfish...)]
              │  stdin / stdout
          [uciproxy]
              │  TCP socket — raw UCI text lines
          [uciserver]
              │  stdin / stdout
    [UCI engine (Stockfish, Dragon, lc0...)]
```

---

## Build

```bash
cargo build --release
```

Output binaries:
- `target/release/uciserver`
- `target/release/uciproxy` (or `.exe` on Windows)

**Cross-compile uciproxy for Windows from Linux:**
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release -p uciproxy --target x86_64-pc-windows-gnu
```

---

## uciserver

Starts the UCI engine and exposes its interface over a TCP port.
Each client connection spawns an independent engine instance.

### Syntax

```
uciserver --port <PORT> --engine <ENGINE_PATH> [--host <IP>]
uciserver -p <PORT> -e <ENGINE_PATH>
```

### Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--port` | `-p` | required | TCP port to listen on |
| `--engine` | `-e` | required | Path to the UCI engine |
| `--host` | | `0.0.0.0` | Network interface (all interfaces by default) |

### Examples

```bash
# Listen on all interfaces, port 8100, engine in PATH
uciserver --port 8100 --engine stockfish

# Short form
uciserver -p 8100 -e stockfish

# Specific network interface
uciserver -p 8100 -e /usr/bin/stockfish --host 192.168.1.10

# Path with spaces (quotes required)
uciserver -p 8100 -e "/usr/games/stockfish 17"

# Multiple engines in parallel (bash script)
uciserver -p 8100 -e /usr/bin/stockfish &
uciserver -p 8200 -e /usr/bin/dragon &

# Help and version
uciserver --help
uciserver --version
```

---

## uciproxy

Connects to a remote `uciserver` and emulates a local UCI engine.
Chess software cannot tell the difference from a local engine.

### Syntax

```
uciproxy --server <IP:PORT> [--retry <SECONDS>]
uciproxy -s <IP:PORT>
```

### Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--server` | `-s` | optional* | Server address `ip:port` |
| `--retry` | `-r` | `5` | Reconnection delay in seconds (0 = disabled) |

*If `--server` is omitted, uciproxy looks for a `<binary_name>.txt` file (see named proxy mode below).

### Examples

```bash
# Direct connection to a uciserver
uciproxy --server 192.168.1.65:8100
uciproxy -s 192.168.1.65:8100

# Reconnect every 10 seconds if connection drops
uciproxy -s 192.168.1.65:8100 --retry 10

# No automatic reconnection
uciproxy -s 192.168.1.65:8100 --retry 0

# Help and version
uciproxy --help
uciproxy --version
```

### Named proxy mode (no argument)

Many chess applications do not allow passing arguments to the engine binary.
In this case, copy the executable with the desired name and create an associated `.txt` file.

**Linux:**
```bash
cp uciproxy remote_stockfish
echo "192.168.1.65:8100" > remote_stockfish.txt
./remote_stockfish
```

**Windows:**
```cmd
copy uciproxy.exe remote_stockfish.exe
echo 192.168.1.65:8100 > remote_stockfish.txt
remote_stockfish.exe
```

uciproxy automatically looks for `remote_stockfish.txt` (same folder, same name, `.txt` extension) and connects to the address it contains.

### Automatic reconnection

By default (`--retry 5`), if the server disconnects (Unraid restart, network drop...), uciproxy waits 5 seconds and retries automatically, without the chess software noticing.

---

## Full use case

### Unraid / Linux server (192.168.1.65)

Via Docker Compose (see [server-docker-uciengines](https://github.com/Antidote1911/server-docker-uciengines)):
```bash
docker compose up -d
# Stockfish on port 8100
# Dragon on port 8200
```

Or directly:
```bash
uciserver -p 8100 -e /usr/bin/stockfish &
uciserver -p 8200 -e /usr/bin/dragon &
```

### Windows client (CuteChess, Arena, Chessbase)

```cmd
copy uciproxy.exe remote_stockfish.exe
echo 192.168.1.65:8100 > remote_stockfish.txt

copy uciproxy.exe remote_dragon.exe
echo 192.168.1.65:8200 > remote_dragon.txt
```

Add `remote_stockfish.exe` and `remote_dragon.exe` to your chess software as regular local engines.

### Android client (Droidfish)

Droidfish natively supports TCP engines:
`Settings → Network engine → 192.168.1.65:8100`

For access from outside the local network, use **Tailscale**:
`Settings → Network engine → 100.70.128.103:8100`
