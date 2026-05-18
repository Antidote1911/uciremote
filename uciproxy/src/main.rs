// uciproxy — se connecte à un uciserver et se comporte comme un moteur UCI local.
// Compatible Windows et Linux.
//
// Usage :
//   uciproxy --server 192.168.2.30:8100
//   uciproxy -s 192.168.2.30:8100 --retry 5
//
// Ou sans argument (mode proxy nommé) :
//   Copier l'exécutable sous le nom voulu, ex: remote_stockfish[.exe]
//   Créer remote_stockfish.txt contenant "192.168.2.30:8100"
//   Lancer remote_stockfish[.exe]

use std::env;
use std::path::PathBuf;
use std::time::Duration;
use clap::Parser;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;

const AUTHOR: &str = "
Author : Fabrice Corraire <antidote1911@gmail.com>
Github : https://github.com/Antidote1911/
";

#[derive(Parser)]
#[command(about = "UCI chess engine proxy — emulates a local engine via a remote uciserver",
          author = AUTHOR, version)]
struct Cli {
    /// Adresse du serveur uciserver (ex: 192.168.1.65:8100)
    /// Si absent, cherche un fichier <nom_du_binaire>.txt
    #[arg(short, long)]
    server: Option<String>,

    /// Délai en secondes entre deux tentatives de reconnexion (0 = pas de reconnexion)
    #[arg(short, long, default_value = "5")]
    retry: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let server_addr = match cli.server {
        Some(addr) => addr.trim().to_string(),
        None => resolve_addr_from_file()?,
    };

    let retry_delay = cli.retry;

    loop {
        match run_session(&server_addr).await {
            Ok(()) => {
                // Déconnexion propre (EOF stdin ou serveur fermé)
                eprintln!("[uciproxy] Session ended.");
                if retry_delay == 0 {
                    break;
                }
            }
            Err(e) => {
                eprintln!("[uciproxy] Error: {e}");
                if retry_delay == 0 {
                    return Err(e);
                }
            }
        }

        eprintln!("[uciproxy] Reconnecting to {server_addr} in {retry_delay}s...");
        sleep(Duration::from_secs(retry_delay)).await;
    }

    Ok(())
}

/// Une session complète : connexion → bridge → déconnexion
async fn run_session(server_addr: &str) -> anyhow::Result<()> {
    eprintln!("[uciproxy] Connecting to {server_addr}...");

    let stream = TcpStream::connect(server_addr).await.map_err(|e| {
        anyhow::anyhow!("Cannot connect to '{}': {}", server_addr, e)
    })?;

    eprintln!("[uciproxy] Connected to {server_addr}");

    let (tcp_reader, mut tcp_writer) = stream.into_split();
    let mut tcp_lines = BufReader::new(tcp_reader).lines();

    let stdin = io::stdin();
    let mut stdin_lines = BufReader::new(stdin).lines();
    let mut stdout = io::stdout();

    loop {
        tokio::select! {
            // stdin (logiciel d'échecs) → serveur
            line = stdin_lines.next_line() => {
                match line? {
                    Some(l) => {
                        tcp_writer.write_all(l.as_bytes()).await?;
                        tcp_writer.write_all(b"\n").await?;
                        tcp_writer.flush().await?;
                    }
                    None => {
                        eprintln!("[uciproxy] stdin closed.");
                        return Ok(());
                    }
                }
            }
            // serveur → stdout (vers logiciel d'échecs)
            line = tcp_lines.next_line() => {
                match line? {
                    Some(l) => {
                        stdout.write_all(l.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                    None => {
                        eprintln!("[uciproxy] Server disconnected.");
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// Trouve l'adresse du serveur depuis le fichier <nom_du_binaire>.txt
/// Ex: remote_stockfish.exe → remote_stockfish.txt
fn resolve_addr_from_file() -> anyhow::Result<String> {
    let exe_path = env::current_exe()?;
    let config_path = config_file_path(&exe_path);

    if !config_path.exists() {
        anyhow::bail!(
            "No server address provided.\n\
            Usage:\n  uciproxy --server <ip>:<port>\n\
            Or create '{}' with '<ip>:<port>' inside.",
            config_path.display()
        );
    }

    let content = std::fs::read_to_string(&config_path).map_err(|e| {
        anyhow::anyhow!("Cannot read '{}': {}", config_path.display(), e)
    })?;

    let addr = content.lines().next().unwrap_or("").trim().to_string();

    if addr.is_empty() {
        anyhow::bail!("Config file '{}' is empty", config_path.display());
    }

    Ok(addr)
}

/// <exe>.txt (sans extension éventuelle .exe)
fn config_file_path(exe: &PathBuf) -> PathBuf {
    let stem = exe.file_stem().unwrap_or_default();
    let mut config = exe.with_file_name(stem);
    config.set_extension("txt");
    config
}
