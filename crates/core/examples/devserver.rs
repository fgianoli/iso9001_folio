//! Server di prova: serve la cartella `ui/` e inoltra `POST /rpc` a `api::chiama`.
//! Serve solo a provare l'interfaccia in un browser (e nei test end-to-end); l'app vera usa Tauri.
//!
//!   cargo run -p commesse-core --example devserver -- <cartella-ui> <cartella-dati> [porta]
use commesse_core::{api, Contesto};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

fn tipo(p: &str) -> &'static str {
    match p.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => "application/octet-stream",
    }
}

fn risposta(s: &mut TcpStream, codice: &str, ct: &str, corpo: &[u8]) {
    let _ = write!(s, "HTTP/1.1 {codice}\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", corpo.len());
    let _ = s.write_all(corpo);
}

fn gestisci(mut s: TcpStream, ui: &std::path::Path, ctx: &Contesto) {
    let mut r = BufReader::new(s.try_clone().unwrap());
    let mut prima = String::new();
    if r.read_line(&mut prima).is_err() {
        return;
    }
    let mut lung = 0usize;
    loop {
        let mut h = String::new();
        if r.read_line(&mut h).unwrap_or(0) == 0 || h == "\r\n" {
            break;
        }
        if let Some(v) = h.to_lowercase().strip_prefix("content-length:") {
            lung = v.trim().parse().unwrap_or(0);
        }
    }
    let mut it = prima.split_whitespace();
    let (metodo, percorso) = (it.next().unwrap_or(""), it.next().unwrap_or("/"));
    if metodo == "POST" && percorso == "/rpc" {
        let mut corpo = vec![0u8; lung];
        let _ = r.read_exact(&mut corpo);
        let v: serde_json::Value = serde_json::from_slice(&corpo).unwrap_or_default();
        let cmd = v["cmd"].as_str().unwrap_or("");
        match api::chiama(ctx, cmd, &v["args"]) {
            Ok(x) => risposta(&mut s, "200 OK", "application/json", x.to_string().as_bytes()),
            Err(e) => risposta(&mut s, "400 Bad Request", "application/json", serde_json::json!({"errore": e.0}).to_string().as_bytes()),
        }
        return;
    }
    let p = percorso.split('?').next().unwrap_or("/");
    let p = if p == "/" { "/index.html" } else { p };
    if p.contains("..") {
        return risposta(&mut s, "403 Forbidden", "text/plain", b"no");
    }
    match std::fs::read(ui.join(p.trim_start_matches('/'))) {
        Ok(b) => risposta(&mut s, "200 OK", tipo(p), &b),
        Err(_) => risposta(&mut s, "404 Not Found", "text/plain", b"non trovato"),
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let ui = PathBuf::from(a.get(1).expect("cartella ui"));
    let dati = PathBuf::from(a.get(2).expect("cartella dati"));
    let porta = a.get(3).and_then(|p| p.parse().ok()).unwrap_or(8780u16);
    let ctx = Arc::new(Contesto::nuovo(Some(dati)).expect("contesto"));
    let l = TcpListener::bind(("127.0.0.1", porta)).expect("porta");
    println!("pronto http://127.0.0.1:{porta}/");
    for s in l.incoming().flatten() {
        let (ctx, ui) = (ctx.clone(), ui.clone());
        std::thread::spawn(move || gestisci(s, &ui, &ctx));
    }
}
