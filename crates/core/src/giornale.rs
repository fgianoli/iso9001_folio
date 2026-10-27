//! Spostamenti sicuri e giornale per l'annullamento. Il programma non elimina mai nulla.
use crate::config::{Config, Contesto};
use crate::errore::{err, Res};
use crate::registro;
use crate::util::{e_dentro, scrivi_atomico};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Voce {
    pub src: String,
    pub dst: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PrimaRegistro {
    pub codice: String,
    #[serde(rename = "Percorso")]
    pub percorso: String,
    #[serde(rename = "Stato")]
    pub stato: String,
    #[serde(default, rename = "Conserva fino al")]
    pub conserva: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File {
    pub op: String,
    pub voci: Vec<Voce>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registro: Option<Vec<PrimaRegistro>>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Esito {
    pub esito: String,
    pub src: String,
    pub dst: String,
    pub msg: String,
}

impl Esito {
    pub fn nuovo(esito: &str, src: impl ToString, dst: impl ToString, msg: impl ToString) -> Esito {
        Esito { esito: esito.into(), src: src.to_string(), dst: dst.to_string(), msg: msg.to_string() }
    }
}

fn copia_ricorsiva(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        for e in std::fs::read_dir(src)? {
            let e = e?;
            copia_ricorsiva(&e.path(), &dst.join(e.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(src, dst).map(|_| ())
    }
}

/// Sposta `src` in `dst` senza mai sovrascrivere. Restituisce (esito, messaggio):
/// `simulato`, `ok`, `manca`, `esiste`, `errore`. Gli spostamenti riusciti si aggiungono a `voci`.
pub fn sposta(src: &Path, dst: &Path, simula: bool, voci: &mut Vec<Voce>, base_src: &Path, base_dst: &Path) -> (String, String) {
    let r = |a: &str, b: &str| (a.to_string(), b.to_string());
    if !e_dentro(src, base_src) {
        return r("errore", "origine fuori dalle cartelle consentite");
    }
    if !e_dentro(dst, base_dst) {
        return r("errore", "destinazione fuori dalle cartelle consentite");
    }
    if !src.exists() {
        return r("manca", "l'origine non esiste");
    }
    if dst.exists() {
        return r("esiste", "la destinazione esiste già");
    }
    if e_dentro(dst, src) {
        return r("errore", "la destinazione è dentro l'origine");
    }
    if simula {
        return r("simulato", "");
    }
    if let Some(p) = dst.parent() {
        if let Err(e) = std::fs::create_dir_all(p) {
            return ("errore".into(), e.to_string());
        }
    }
    if let Err(e) = std::fs::rename(src, dst) {
        if e.kind() != std::io::ErrorKind::CrossesDevices {
            return ("errore".into(), e.to_string());
        }
        // Altro disco o altro volume: si copia, e si toglie l'originale solo a copia riuscita.
        match copia_ricorsiva(src, dst) {
            Ok(()) => {
                let tolto = if src.is_dir() { std::fs::remove_dir_all(src) } else { std::fs::remove_file(src) };
                if let Err(e) = tolto {
                    return ("errore".into(), format!("copiato, ma l'originale non si è potuto togliere: {e}"));
                }
            }
            Err(e) => {
                let _ = if dst.is_dir() { std::fs::remove_dir_all(dst) } else { std::fs::remove_file(dst) };
                return ("errore".into(), e.to_string());
            }
        }
    }
    voci.push(Voce { src: src.to_string_lossy().into(), dst: dst.to_string_lossy().into() });
    r("ok", "")
}

pub fn salva(ctx: &Contesto, op: &str, voci: Vec<Voce>, prima: Option<Vec<PrimaRegistro>>) -> Res<Option<String>> {
    if voci.is_empty() {
        return Ok(None);
    }
    let nome = format!("{}_{}.json", chrono::Local::now().format("%Y%m%d_%H%M%S_%6f"), op);
    let f = File { op: op.into(), voci, registro: prima };
    scrivi_atomico(&ctx.cartella_giornale().join(&nome), serde_json::to_string_pretty(&f)?.as_bytes())?;
    Ok(Some(nome))
}

pub fn elenco(ctx: &Contesto) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(ctx.cartella_giornale())
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.ends_with(".json")).collect())
        .unwrap_or_default();
    v.sort();
    v.reverse();
    v
}

#[derive(Serialize, Debug)]
pub struct RisultatoAnnulla {
    pub giornale: String,
    pub esiti: Vec<Esito>,
}

/// Riporta ogni elemento dove si trovava prima dell'ultima operazione di spostamento.
pub fn annulla_ultimo(ctx: &Contesto, cfg: &Config, simula: bool) -> Res<RisultatoAnnulla> {
    let _g = ctx.blocca();
    let gs = elenco(ctx);
    let Some(nome) = gs.first().cloned() else { return err("Nessuna operazione da annullare") };
    let percorso = ctx.cartella_giornale().join(&nome);
    let dati: File = serde_json::from_str(&std::fs::read_to_string(&percorso)?)?;
    let mut esiti = vec![];
    for v in dati.voci.iter().rev() {
        let (s, d) = (Path::new(&v.dst), Path::new(&v.src));
        if !s.exists() {
            esiti.push(Esito::nuovo("manca", s.display(), d.display(), "non c'è più dove era stato messo"));
        } else if d.exists() {
            esiti.push(Esito::nuovo("esiste", s.display(), d.display(), "il posto originale è occupato"));
        } else if simula {
            esiti.push(Esito::nuovo("simulato", s.display(), d.display(), ""));
        } else {
            let mut tmp = vec![];
            let base_s = s.parent().unwrap_or(s);
            let base_d = d.parent().unwrap_or(d);
            let (es, msg) = sposta(s, d, false, &mut tmp, base_s, base_d);
            esiti.push(Esito::nuovo(&es, s.display(), d.display(), msg));
        }
    }
    if !simula {
        if let Some(prima) = &dati.registro {
            if esiti.iter().all(|e| e.esito == "ok") {
                let r = (|| -> Res<()> {
                    let mut righe = registro::leggi(cfg)?;
                    for p in prima {
                        let r = registro::trova(&mut righe, &p.codice)?;
                        r.insert("Percorso".into(), p.percorso.clone());
                        r.insert("Stato".into(), p.stato.clone());
                        r.insert("Conserva fino al".into(), p.conserva.clone());
                    }
                    registro::riscrivi(cfg, &righe)
                })();
                if let Err(e) = r {
                    esiti.push(Esito::nuovo("errore", "registro", "", e.0));
                }
            }
        }
        crate::eventi::registra(cfg, "Operazione annullata", "", &format!("{} ({} elementi riportati)", nome, esiti.iter().filter(|e| e.esito == "ok").count()));
        let mut nuovo = percorso.as_os_str().to_owned();
        nuovo.push(".annullato");
        std::fs::rename(&percorso, nuovo)?;
    }
    Ok(RisultatoAnnulla { giornale: nome, esiti })
}
