//! Non conformità e azioni correttive (ISO 9001 cl. 8.7 «Controllo degli output non conformi» e 10.2).
use crate::config::{Config, Contesto};
use crate::csvio::{self, Riga};
use crate::errore::{err, Errore, Res};
use crate::{eventi, registro, util};
use serde::Deserialize;
use std::path::PathBuf;

pub const COLONNE: [&str; 10] = ["Codice NC", "Aperta il", "Commessa", "Descrizione", "Causa", "Azione correttiva", "Responsabile", "Stato", "Chiusa il", "Efficacia verificata il"];
pub const STATI: [&str; 3] = ["Aperta", "In corso", "Chiusa"];

pub fn file(cfg: &Config) -> PathBuf {
    cfg.cartella_gestione().join("non_conformita.csv")
}

pub fn leggi(cfg: &Config) -> Res<Vec<Riga>> {
    csvio::leggi(&file(cfg))
}

fn scrivi(cfg: &Config, righe: &[Riga]) -> Res<()> {
    let cols: Vec<String> = COLONNE.iter().map(|s| s.to_string()).collect();
    csvio::scrivi(&file(cfg), &cols, righe)
}

/// Quante non conformità non ancora chiuse riguardano la commessa.
pub fn aperte_per(cfg: &Config, commessa: &str) -> Res<usize> {
    Ok(leggi(cfg)?.iter().filter(|r| r.get("Commessa").map(|c| c == commessa).unwrap_or(false) && r.get("Stato").map(|s| s != "Chiusa").unwrap_or(true)).count())
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct NuovaNc {
    pub commessa: String,
    pub descrizione: String,
    pub responsabile: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct AggiornaNc {
    pub codice: String,
    pub causa: Option<String>,
    pub azione: Option<String>,
    pub responsabile: Option<String>,
    pub stato: Option<String>,
    /// Segna la verifica dell'efficacia dell'azione (10.2.1 e)
    pub efficacia: Option<bool>,
}

pub fn nuova(ctx: &Contesto, d: &NuovaNc) -> Res<Riga> {
    let cfg = ctx.cfg();
    let _g = ctx.blocca();
    let desc = d.descrizione.trim();
    if desc.is_empty() {
        return err("Descrivi la non conformità");
    }
    let commessa = d.commessa.trim();
    if !commessa.is_empty() && !registro::leggi(&cfg)?.iter().any(|r| r.get("Codice").map(|c| c == commessa).unwrap_or(false)) {
        return err(format!("{commessa} non è nel registro"));
    }
    let mut righe = leggi(&cfg)?;
    let anno = util::anno_corrente().to_string();
    let n = righe.iter().filter_map(|r| r.get("Codice NC")).filter_map(|c| c.strip_prefix(&format!("NC-{anno}-"))).filter_map(|x| x.parse::<u32>().ok()).max().unwrap_or(0) + 1;
    let codice = format!("NC-{anno}-{n:03}");
    let mut r = Riga::new();
    for (k, v) in [("Codice NC", codice.as_str()), ("Aperta il", &util::oggi()), ("Commessa", commessa), ("Descrizione", desc), ("Responsabile", d.responsabile.trim()), ("Stato", "Aperta")] {
        r.insert(k.into(), v.into());
    }
    righe.push(r.clone());
    scrivi(&cfg, &righe)?;
    eventi::registra(&cfg, "Non conformità aperta", commessa, &format!("{codice}: {desc}"));
    Ok(r)
}

pub fn aggiorna(ctx: &Contesto, d: &AggiornaNc) -> Res<Riga> {
    let cfg = ctx.cfg();
    let _g = ctx.blocca();
    let mut righe = leggi(&cfg)?;
    let r = righe.iter_mut().find(|r| r.get("Codice NC") == Some(&d.codice)).ok_or_else(|| Errore(format!("{} non trovata", d.codice)))?;
    if let Some(v) = &d.causa {
        r.insert("Causa".into(), v.trim().into());
    }
    if let Some(v) = &d.azione {
        r.insert("Azione correttiva".into(), v.trim().into());
    }
    if let Some(v) = &d.responsabile {
        r.insert("Responsabile".into(), v.trim().into());
    }
    if let Some(e) = d.efficacia {
        r.insert("Efficacia verificata il".into(), if e { util::oggi() } else { String::new() });
    }
    if let Some(s) = &d.stato {
        if !STATI.contains(&s.as_str()) {
            return err("Stato non valido");
        }
        if s == "Chiusa" {
            let vuoto = |k: &str| r.get(k).map(|v| v.trim().is_empty()).unwrap_or(true);
            if vuoto("Causa") || vuoto("Azione correttiva") {
                return err("Per chiudere servono la causa e l'azione correttiva (ISO 9001 10.2)");
            }
            r.insert("Chiusa il".into(), util::oggi());
        } else {
            r.insert("Chiusa il".into(), String::new());
        }
        r.insert("Stato".into(), s.clone());
    }
    if r.get("Stato").map(|s| s == "Chiusa").unwrap_or(false) && r.get("Efficacia verificata il").map(|v| v.is_empty()).unwrap_or(true) {
        // la chiusura è ammessa, ma l'efficacia resta da verificare: la colonna vuota lo rende visibile nell'elenco
    }
    let out = r.clone();
    let commessa = out.get("Commessa").cloned().unwrap_or_default();
    scrivi(&cfg, &righe)?;
    eventi::registra(&cfg, "Non conformità aggiornata", &commessa, &format!("{} → {}", d.codice, out.get("Stato").cloned().unwrap_or_default()));
    Ok(out)
}
