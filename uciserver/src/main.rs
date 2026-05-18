// uciserver — écoute un port TCP, lance le moteur UCI pour chaque connexion
// et fait le pont bidirectionnel socket ↔ stdin/stdout du moteur.
//
// Usage : uciserver --port 7900 --engine stockfish
//         uciserver -p 7900 -e /usr/bin/stockfish
//         uciserver -p 7900 -e stockfish --host 192.168.1.10

use std::process::Stdio;
use clap::Parser;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::Command;

const AUTHOR: &str = "
Author : Fabrice Corraire <antidote1911@gmail.com>
Github : https://github.com/Antidote1911/
";

#[derive(Parser)]
#[command(about = "UCI chess engine TCP server", author = AUTHOR, version)]
struct Cli {
    /// Port TCP à écouter
    #[arg(short, long)]
    port: u16,

    /// Chemin vers le moteur UCI (ex: stockfish, /usr/bin/lc0)
    #[arg(short, long)]
    engine: String,

    /// Interface réseau à écouter (défaut: toutes les interfaces)
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let addr = format!("{}:{}", cli.host, cli.port);
    let engine_path = cli.engine;

    // --- écoute TCP ----------------------------------------------------------
    let listener = TcpListener::bind(&addr).await?;
    eprintln!("[uciserver] Listening on {addr}, engine: {engine_path}");

    loop {
        let (socket, peer) = listener.accept().await?;
        let engine_path = engine_path.clone();

        tokio::spawn(async move {
            eprintln!("[uciserver] New connection from {peer}");
            if let Err(e) = handle_connection(socket, &engine_path).await {
                eprintln!("[uciserver] Connection {peer} ended: {e}");
            }
            eprintln!("[uciserver] Disconnected {peer}");
        });
    }
}

/// Gère une connexion cliente : lance le moteur et fait le pont bidirectionnel.
async fn handle_connection(
    socket: tokio::net::TcpStream,
    engine_path: &str,
) -> anyhow::Result<()> {
    let mut child = Command::new(engine_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start engine '{}': {}", engine_path, e))?;

    let engine_stdin = child.stdin.take().expect("engine stdin");
    let engine_stdout = child.stdout.take().expect("engine stdout");

    let (socket_reader, mut socket_writer) = socket.into_split();
    let mut socket_lines = BufReader::new(socket_reader).lines();
    let mut engine_lines = BufReader::new(engine_stdout).lines();
    let mut engine_stdin = engine_stdin;

    loop {
        tokio::select! {
            // Socket → moteur (commandes UCI du client)
            line = socket_lines.next_line() => {
                match line? {
                    Some(l) => {
                        engine_stdin.write_all(l.as_bytes()).await?;
                        engine_stdin.write_all(b"\n").await?;
                        engine_stdin.flush().await?;
                    }
                    None => break, // client déconnecté
                }
            }
            // Moteur → socket (réponses UCI)
            line = engine_lines.next_line() => {
                match line? {
                    Some(l) => {
                        socket_writer.write_all(l.as_bytes()).await?;
                        socket_writer.write_all(b"\n").await?;
                        socket_writer.flush().await?;
                    }
                    None => break, // moteur terminé
                }
            }
        }
    }

    // Nettoyage : tue le moteur si encore vivant
    let _ = child.kill().await;
    Ok(())
}
