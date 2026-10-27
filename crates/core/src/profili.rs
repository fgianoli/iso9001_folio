//! Profili: configurazioni pronte (tipi, nomenclature, vocabolario) e importazione/esportazione per condividerle.
use crate::config::Config;
use crate::errore::{Errore, Res};
use serde::Serialize;

struct Incorporato {
    id: &'static str,
    nome: &'static str,
    descrizione: &'static str,
    json: &'static str,
    nome_en: &'static str,
    descrizione_en: &'static str,
    json_en: &'static str,
}

const PROFILI: [Incorporato; 3] = [
    Incorporato { id: "studio", nome: "Studio professionale", descrizione: "Clienti, ricerca, sviluppo, istituzioni e associazioni. Codici 2026-014.", json: include_str!("../profili/studio.json"), nome_en: "Professional firm", descrizione_en: "Clients, research, development, institutions and associations. Codes 2026-014.", json_en: include_str!("../profili/studio_en.json") },
    Incorporato { id: "ricerca", nome: "Ricerca e consulenza", descrizione: "Progetti con sigla (RIC-2026-003), numerazione per tipo e colonne per finanziatore e CUP.", json: include_str!("../profili/ricerca.json"), nome_en: "Research and consulting", descrizione_en: "Projects with code letters (RES-2026-003), numbering per type and columns for funder and grant.", json_en: include_str!("../profili/ricerca_en.json") },
    Incorporato { id: "minimo", nome: "Minimo", descrizione: "Un solo tipo, poche sottocartelle: da ampliare a piacere.", json: include_str!("../profili/minimo.json"), nome_en: "Minimal", descrizione_en: "One type, a few subfolders: expand as you like.", json_en: include_str!("../profili/minimo_en.json") },
];

#[derive(Serialize, Debug)]
pub struct InfoProfilo {
    pub id: String,
    pub nome: String,
    pub descrizione: String,
}

pub fn elenco(lingua: &str) -> Vec<InfoProfilo> {
    let en = lingua == "en";
    PROFILI.iter().map(|p| InfoProfilo { id: p.id.into(), nome: (if en { p.nome_en } else { p.nome }).into(), descrizione: (if en { p.descrizione_en } else { p.descrizione }).into() }).collect()
}

/// Applica un profilo mantenendo le cartelle di questo computer.
fn con_cartelle_di(mut nuovo: Config, attuale: &Config) -> Res<Config> {
    nuovo.cartelle.radice = attuale.cartelle.radice.clone();
    nuovo.cartelle.origine = attuale.cartelle.origine.clone();
    nuovo.cartelle.radici_scansione = attuale.cartelle.radici_scansione.clone();
    nuovo.valida()?;
    Ok(nuovo)
}

pub fn applica(id: &str, attuale: &Config) -> Res<Config> {
    let p = PROFILI.iter().find(|p| p.id == id).ok_or_else(|| Errore(format!("Profilo sconosciuto: {id}")))?;
    let nuovo: Config = serde_json::from_str(if attuale.app.lingua == "en" { p.json_en } else { p.json })?;
    con_cartelle_di(nuovo, attuale)
}

/// Il profilo da condividere: tutto tranne i percorsi di questo computer.
pub fn esporta(cfg: &Config) -> Res<String> {
    let mut c = cfg.clone();
    c.cartelle.radice = String::new();
    c.cartelle.origine = String::new();
    c.cartelle.radici_scansione = vec![];
    Ok(serde_json::to_string_pretty(&c)?)
}

pub fn importa(testo: &str, attuale: &Config) -> Res<Config> {
    let nuovo: Config = serde_json::from_str(testo)?;
    con_cartelle_di(nuovo, attuale)
}
