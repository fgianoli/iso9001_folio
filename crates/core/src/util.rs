//! Funzioni di utilità senza stato.
use crate::errore::{err, Res};
use std::path::{Component, Path, PathBuf};
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

pub const STILI_NOME: [&str; 4] = ["pascal", "snake", "kebab", "maiuscolo"];

/// Trasforma un testo libero in un nome adatto alle cartelle: niente accenti, spazi o simboli.
/// Stili: `pascal` (SincronizzazioneGpkg), `snake` (sincronizzazione_gpkg), `kebab`, `maiuscolo` (SINCRONIZZAZIONEGPKG).
pub fn pulisci_nome(s: &str, stile: &str) -> String {
    let senza: String = s.nfd().filter(|c| !is_combining_mark(*c)).collect();
    let parole: Vec<&str> = senza.split(|c: char| !c.is_ascii_alphanumeric()).filter(|p| !p.is_empty()).collect();
    match stile {
        "snake" => parole.iter().map(|p| p.to_lowercase()).collect::<Vec<_>>().join("_"),
        "kebab" => parole.iter().map(|p| p.to_lowercase()).collect::<Vec<_>>().join("-"),
        "maiuscolo" => parole.iter().map(|p| p.to_uppercase()).collect::<String>(),
        _ => parole
            .iter()
            .map(|p| {
                let mut c = p.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect(),
    }
}

/// Percorso assoluto e normalizzato lessicalmente (senza `.` e `..`), senza toccare il disco.
pub fn normalizza(p: &Path) -> PathBuf {
    let ass = std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
    let mut out = PathBuf::new();
    for c in ass.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            altro => out.push(altro.as_os_str()),
        }
    }
    out
}

fn chiave(p: &Path) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(p.to_string_lossy().to_lowercase())
    } else {
        p.to_path_buf()
    }
}

/// Vero se `figlio` sta dentro `padre` (o coincide).
pub fn e_dentro(figlio: &Path, padre: &Path) -> bool {
    chiave(&normalizza(figlio)).starts_with(chiave(&normalizza(padre)))
}

pub fn oggi() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

pub fn anno_corrente() -> i32 {
    use chrono::Datelike;
    chrono::Local::now().year()
}

/// Accetta solo percorsi relativi e senza `..`.
pub fn percorso_relativo(rel: &str) -> Res<PathBuf> {
    let t = rel.trim();
    if t.is_empty() {
        return err("Percorso vuoto");
    }
    let uniforme = t.replace('\\', "/");
    let b = uniforme.as_bytes();
    let con_unita = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
    if uniforme.starts_with('/') || con_unita {
        return err(format!("Usa un percorso relativo, senza '..': {t}"));
    }
    let mut out = PathBuf::new();
    for parte in uniforme.split('/') {
        match parte {
            "" | "." => {}
            ".." => return err(format!("Usa un percorso relativo, senza '..': {t}")),
            p => out.push(p),
        }
    }
    if out.as_os_str().is_empty() {
        return err("Percorso vuoto");
    }
    Ok(out)
}

/// Nome di un singolo elemento (cartella o file): niente separatori, niente caratteri vietati su Windows.
pub fn nome_semplice_valido(n: &str) -> bool {
    let t = n.trim();
    !t.is_empty()
        && t == n
        && !t.ends_with('.')
        && t != "."
        && t != ".."
        && !t.chars().any(|c| "<>:\"/\\|?*".contains(c) || c.is_control())
}

/// Percorso relativo interno (sottocartelle): ogni parte deve essere un nome semplice.
pub fn sottopercorso_valido(p: &str) -> bool {
    let u = p.trim().replace('\\', "/");
    !u.is_empty() && !u.starts_with('/') && u.split('/').all(nome_semplice_valido)
}

/// Scrittura atomica: file temporaneo + rinomina. Errore leggibile se il file è aperto altrove.
pub fn scrivi_atomico(dest: &Path, contenuto: &[u8]) -> Res<()> {
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut tmp = dest.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, contenuto)?;
    if let Err(e) = std::fs::rename(&tmp, dest) {
        let _ = std::fs::remove_file(&tmp);
        if e.kind() == std::io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(32) {
            return err(format!(
                "Il file {} è aperto in un altro programma (Excel?): chiudilo e riprova",
                dest.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
            ));
        }
        return Err(e.into());
    }
    Ok(())
}

/// Colore esadecimale `#rrggbb`.
pub fn colore_valido(c: &str) -> bool {
    c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|x| x.is_ascii_hexdigit())
}

/// Aggiunge anni a una data `AAAA-MM-GG` (il 29 febbraio diventa 28 negli anni non bisestili).
pub fn aggiungi_anni(data: &str, anni: u32) -> String {
    use chrono::{Datelike, NaiveDate};
    let Ok(d) = NaiveDate::parse_from_str(data, "%Y-%m-%d") else { return String::new() };
    let anno = d.year() + anni as i32;
    d.with_year(anno).or_else(|| NaiveDate::from_ymd_opt(anno, d.month(), d.day().saturating_sub(1))).map(|x| x.format("%Y-%m-%d").to_string()).unwrap_or_default()
}
