//! Registro cronologico delle operazioni (ISO 9001 cl. 7.5.3: conservazione e controllo delle informazioni documentate).
//! Si scrive solo in coda: nessuna voce già registrata viene modificata o tolta.
use crate::config::Config;
use crate::csvio::{self, Riga};
use crate::errore::Res;
use std::path::PathBuf;

pub const COLONNE: [&str; 5] = ["Data e ora", "Utente", "Operazione", "Codice", "Dettaglio"];

pub fn file(cfg: &Config) -> PathBuf {
    cfg.cartella_gestione().join("registro_eventi.csv")
}

pub fn utente() -> String {
    ["USERNAME", "USER", "LOGNAME"].iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty())).unwrap_or_default()
}

/// Registra un evento. Se la cartella di gestione non esiste ancora (prima configurazione) non fa nulla.
/// Un errore di scrittura non blocca l'operazione principale.
fn etichetta_en(op: &str) -> String {
    let t = match op {
        "Configurazione modificata" => "Settings changed",
        "Operazione annullata" => "Operation undone",
        "Non conformità aperta" => "Nonconformity opened",
        "Non conformità aggiornata" => "Nonconformity updated",
        "Commessa creata" => "Job created",
        "Riordino" => "Reorganization",
        "Archiviata" => "Archived",
        "Procedura emessa" => "Procedure issued",
        "Quarantena" => "Quarantine",
        _ => return op.strip_prefix("Registro: ").map(|r| format!("Register: {r}")).unwrap_or_else(|| op.to_string()),
    };
    t.into()
}

pub fn registra(cfg: &Config, operazione: &str, codice: &str, dettaglio: &str) {
    let operazione = &if cfg.app.lingua == "en" { etichetta_en(operazione) } else { operazione.to_string() };
    if !cfg.qualita.registra_eventi || !cfg.cartella_gestione().is_dir() {
        return;
    }
    let ora = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = csvio::aggiungi_in_coda(&file(cfg), &COLONNE, &[&ora, &utente(), operazione, codice, dettaglio]);
}

/// Le ultime `n` voci, dalla più recente.
pub fn ultimi(cfg: &Config, n: usize) -> Res<Vec<Riga>> {
    let mut v = csvio::leggi(&file(cfg))?;
    v.reverse();
    v.truncate(n);
    Ok(v)
}
