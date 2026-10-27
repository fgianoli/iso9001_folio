//! Registro delle commesse: un file CSV (separatore «;», UTF-8 con BOM, si apre in Excel).
//!
//! Colonne di controllo ispirate alla ISO 9001:2015: identificazione e rintracciabilità (8.5.2, Codice),
//! responsabile, riesame dei requisiti (8.2.3), revisione (8.5.6), approvazione e rilascio (8.6),
//! conservazione (7.5.3).
use crate::config::Config;
use crate::csvio;
pub use crate::csvio::Riga;
use crate::errore::{err, Errore, Res};
use crate::{eventi, nomi, nonconf};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const BASE: [&str; 17] = [
    "Codice", "Cartella", "Tipo", "Ente", "Oggetto", "Stato", "Aperta il", "Chiusa il", "Fattura", "Percorso", "Breve",
    "Responsabile", "Revisione", "Riesame il", "Approvata da", "Approvata il", "Conserva fino al",
];

/// Colonne attese: quelle fisse più le colonne extra scelte dall'utente.
pub fn colonne(cfg: &Config) -> Vec<String> {
    let mut c: Vec<String> = BASE.iter().map(|s| s.to_string()).collect();
    for e in &cfg.registro.colonne_extra {
        if !c.contains(e) {
            c.push(e.clone());
        }
    }
    c
}

pub fn leggi(cfg: &Config) -> Res<Vec<Riga>> {
    let mut righe = csvio::leggi(&cfg.file_registro())?;
    let re_cartella = nomi::regex_da_modello(&cfg.nomenclatura.cartella, Some(&cfg.nomenclatura.codice)).ok();
    for riga in righe.iter_mut() {
        // Registri creati da versioni precedenti non hanno la colonna «Breve»: la ricaviamo dal nome della cartella.
        if riga.get("Breve").map(|s| s.is_empty()).unwrap_or(true) {
            if let (Some(re), Some(c)) = (&re_cartella, riga.get("Cartella")) {
                if let Some(m) = re.captures(c).and_then(|cap| cap.name("ente")) {
                    riga.insert("Breve".into(), m.as_str().to_string());
                }
            }
        }
    }
    Ok(righe)
}

pub fn riscrivi(cfg: &Config, righe: &[Riga]) -> Res<()> {
    csvio::scrivi(&cfg.file_registro(), &colonne(cfg), righe)
}

pub fn aggiungi(cfg: &Config, riga: Riga) -> Res<()> {
    let mut righe = leggi(cfg)?;
    righe.push(riga);
    riscrivi(cfg, &righe)
}

/// Prossimo numero libero per l'anno (e il tipo) indicati, secondo le regole di numerazione.
pub fn prossimo_numero(cfg: &Config, anno: &str, sigla: &str) -> Res<u64> {
    let n = &cfg.nomenclatura;
    let re = nomi::regex_da_modello(&n.codice, None)?;
    let per_anno = nomi::ha_campo(&n.codice, "anno") && n.azzera_ogni_anno;
    let per_tipo = nomi::ha_campo(&n.codice, "sigla") && n.numerazione_per_tipo;
    let mut max = 0u64;
    for r in leggi(cfg)? {
        let Some(c) = r.get("Codice").and_then(|c| re.captures(c)) else { continue };
        if per_anno && c.name("anno").map(|m| m.as_str()) != Some(anno) {
            continue;
        }
        if per_tipo && c.name("sigla").map(|m| m.as_str()) != Some(sigla) {
            continue;
        }
        if let Some(v) = c.name("num").and_then(|m| m.as_str().parse::<u64>().ok()) {
            max = max.max(v);
        }
    }
    Ok(max + 1)
}

pub fn trova<'a>(righe: &'a mut [Riga], codice: &str) -> Res<&'a mut Riga> {
    righe.iter_mut().find(|r| r.get("Codice").map(|c| c == codice).unwrap_or(false)).ok_or_else(|| Errore(format!("{codice} non trovata nel registro")))
}

/// Modifiche a una riga del registro. Ogni campo è facoltativo.
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Aggiornamento {
    pub codice: String,
    pub stato: Option<String>,
    pub fattura: Option<String>,
    pub extra: BTreeMap<String, String>,
    pub responsabile: Option<String>,
    /// Riesame dei requisiti eseguito (8.2.3): registra la data di oggi; `false` la toglie
    pub riesame: Option<bool>,
    /// Nome di chi approva e rilascia (8.6)
    pub approva: Option<String>,
    pub revoca_approvazione: bool,
    /// Motivo di una modifica ai requisiti: crea la revisione successiva e annulla l'approvazione (8.5.6)
    pub nuova_revisione: Option<String>,
}

fn val(r: &Riga, k: &str) -> String {
    r.get(k).cloned().unwrap_or_default()
}

/// Applica le modifiche, controlla le regole di qualità e riscrive il registro.
pub fn aggiorna(cfg: &Config, d: &Aggiornamento) -> Res<Riga> {
    let mut righe = leggi(cfg)?;
    let r = trova(&mut righe, &d.codice)?;
    if val(r, "Stato") == cfg.registro.stato_archiviato {
        return err("Archiviata: annulla l'archiviazione per modificarla");
    }
    let mut eventi_da_scrivere: Vec<(String, String)> = vec![];
    let stato_prima = val(r, "Stato");

    if let Some(v) = &d.responsabile {
        r.insert("Responsabile".into(), v.trim().into());
        eventi_da_scrivere.push(("Responsabile".into(), v.trim().into()));
    }
    match d.riesame {
        Some(true) if val(r, "Riesame il").is_empty() => {
            r.insert("Riesame il".into(), crate::util::oggi());
            eventi_da_scrivere.push(("Riesame dei requisiti eseguito".into(), String::new()));
        }
        Some(false) if !val(r, "Riesame il").is_empty() => {
            r.insert("Riesame il".into(), String::new());
            eventi_da_scrivere.push(("Riesame dei requisiti tolto".into(), String::new()));
        }
        _ => {}
    }
    if let Some(motivo) = &d.nuova_revisione {
        let motivo = motivo.trim();
        if motivo.is_empty() {
            return err("Indica il motivo della nuova revisione");
        }
        if stato_prima == cfg.registro.stato_chiuso {
            return err("Riapri la commessa prima di revisionarla");
        }
        let n: u32 = val(r, "Revisione").trim().parse().unwrap_or(0) + 1;
        r.insert("Revisione".into(), format!("{n:02}"));
        r.insert("Approvata da".into(), String::new());
        r.insert("Approvata il".into(), String::new());
        eventi_da_scrivere.push((format!("Nuova revisione {n:02}"), motivo.into()));
    }
    if let Some(chi) = &d.approva {
        let chi = chi.trim();
        if chi.is_empty() {
            return err("Indica chi approva");
        }
        r.insert("Approvata da".into(), chi.into());
        r.insert("Approvata il".into(), crate::util::oggi());
        eventi_da_scrivere.push(("Approvazione e rilascio".into(), chi.into()));
    }
    if d.revoca_approvazione {
        r.insert("Approvata da".into(), String::new());
        r.insert("Approvata il".into(), String::new());
        eventi_da_scrivere.push(("Approvazione revocata".into(), String::new()));
    }
    if let Some(f) = &d.fattura {
        r.insert("Fattura".into(), f.trim().into());
        eventi_da_scrivere.push(("Fattura".into(), f.trim().into()));
    }
    for (k, v) in &d.extra {
        if !cfg.registro.colonne_extra.contains(k) {
            return err(format!("Colonna sconosciuta: {k}"));
        }
        r.insert(k.clone(), v.trim().into());
        eventi_da_scrivere.push((k.clone(), v.trim().into()));
    }
    if let Some(s) = &d.stato {
        if !cfg.registro.stati.iter().any(|x| x == s) {
            return err("Stato non valido");
        }
        if *s == cfg.registro.stato_chiuso && stato_prima != *s {
            if cfg.qualita.richiedi_approvazione && val(r, "Approvata il").is_empty() {
                return err("Prima di chiudere serve l'approvazione (verifica e rilascio, ISO 9001 8.6)");
            }
            let nc = nonconf::aperte_per(cfg, &d.codice)?;
            if nc > 0 {
                return err(format!("Ci sono {nc} non conformità aperte: chiudile prima (ISO 9001 8.7)"));
            }
        }
        r.insert("Stato".into(), s.clone());
        r.insert("Chiusa il".into(), if *s == cfg.registro.stato_chiuso { crate::util::oggi() } else { String::new() });
        if stato_prima != *s {
            eventi_da_scrivere.push(("Stato".into(), format!("{stato_prima} → {s}")));
        }
    }
    let out = r.clone();
    riscrivi(cfg, &righe)?;
    for (op, det) in eventi_da_scrivere {
        eventi::registra(cfg, &format!("Registro: {op}"), &d.codice, &det);
    }
    Ok(out)
}
