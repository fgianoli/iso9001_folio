//! Operazioni sulle cartelle: struttura base, nuova commessa, riordino con mappa, archiviazione.
use crate::clienti;
use crate::config::{readme_predefinito, Config, Contesto};
use crate::errore::{err, Errore, Res};
use crate::eventi;
use crate::giornale::{self, Esito, PrimaRegistro, Voce};
use crate::nomi::{self, Campi};
use crate::registro::{self, Riga};
use crate::util::{oggi, percorso_relativo, pulisci_nome, scrivi_atomico};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Serialize, Debug)]
pub struct Azione {
    pub azione: String,
    pub percorso: String,
}
fn az(a: &str, p: impl ToString) -> Azione {
    Azione { azione: a.into(), percorso: p.to_string() }
}

// ------------------------------------------------------------------ struttura base

pub fn crea_struttura(ctx: &Contesto, simula: bool) -> Res<Vec<Azione>> {
    let cfg = ctx.cfg();
    let _g = ctx.blocca();
    let mut nomi_dir: Vec<String> = vec![cfg.cartelle.gestione.clone(), cfg.cartelle.archivio.clone()];
    nomi_dir.extend(cfg.tipi.iter().map(|t| t.dir.clone()));
    nomi_dir.sort();
    nomi_dir.dedup();
    let mut azioni = vec![az("crea cartella", cfg.radice().display())];
    for n in &nomi_dir {
        azioni.push(az("crea cartella", cfg.radice().join(n).display()));
    }
    if !simula {
        for n in &nomi_dir {
            std::fs::create_dir_all(cfg.radice().join(n))?;
        }
    }
    Ok(azioni)
}

// ------------------------------------------------------------------ nuova commessa

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct NuovaCommessa {
    pub tipo: String,
    pub anno: String,
    pub num: Option<u64>,
    /// Nome breve del cliente in anagrafica
    pub cliente: Option<String>,
    /// Ente scritto a mano, se non si usa l'anagrafica
    pub ente: Option<String>,
    pub ogg: String,
    pub responsabile: String,
    /// Riesame dei requisiti già eseguito (8.2.3)
    pub riesame: bool,
    pub extra: BTreeMap<String, String>,
    pub simula: bool,
}

#[derive(Serialize, Debug)]
pub struct Anteprima {
    pub codice: String,
    pub nome: String,
}

#[derive(Serialize, Debug)]
pub struct RisultatoNuova {
    pub codice: String,
    pub nome: String,
    pub azioni: Vec<Azione>,
}

struct Preparata {
    codice: String,
    nome: String,
    ente_esteso: String,
    breve: String,
    ogg_esteso: String,
    tipo_id: String,
}

fn prepara(ctx: &Contesto, cfg: &Config, d: &NuovaCommessa) -> Res<Preparata> {
    let t = cfg.tipo(&d.tipo)?;
    let anno = d.anno.trim();
    let usa_anno = nomi::ha_campo(&cfg.nomenclatura.codice, "anno") || nomi::ha_campo(&cfg.nomenclatura.cartella, "anno");
    if usa_anno && !(anno.len() == 4 && anno.chars().all(|c| c.is_ascii_digit())) {
        return err("Anno non valido");
    }
    let num = d.num.ok_or_else(|| Errore("Numero non valido".into()))?;
    if num == 0 {
        return err("Numero non valido");
    }
    let stile = &cfg.nomenclatura.stile_nomi;
    let ogg_esteso = d.ogg.trim().to_string();
    let ogg = pulisci_nome(&ogg_esteso, stile);
    let (ente_esteso, breve) = match d.cliente.as_deref() {
        Some("") | None if d.ente.is_none() => return err(format!("Manca: {}", cfg.vocabolario.cliente)),
        Some(c) if !c.is_empty() => {
            let c = clienti::trova(ctx, c).ok_or_else(|| Errore(format!("{} non presente in anagrafica", cfg.vocabolario.cliente)))?;
            (c.nome, c.breve)
        }
        _ => {
            let e = d.ente.clone().unwrap_or_default().trim().to_string();
            let b = pulisci_nome(&e, stile);
            (e, b)
        }
    };
    if breve.is_empty() || ogg.is_empty() {
        return err(format!("Compila {} e oggetto", cfg.vocabolario.cliente));
    }
    let mut c = Campi { anno: anno.into(), num, sigla: t.sigla.clone(), tipo: t.nome.clone(), ente: breve.clone(), oggetto: ogg, data: oggi(), codice: String::new() };
    c.codice = nomi::rendi(&cfg.nomenclatura.codice, &c, cfg.nomenclatura.cifre_numero as usize)?;
    let nome = nomi::rendi(&cfg.nomenclatura.cartella, &c, cfg.nomenclatura.cifre_numero as usize)?;
    if nome.len() > 150 || nome.ends_with(' ') || nome.ends_with('.') {
        return err("Il nome della cartella risulta troppo lungo o termina con un punto: accorcia l'oggetto o cambia la nomenclatura");
    }
    Ok(Preparata { codice: c.codice, nome, ente_esteso, breve, ogg_esteso, tipo_id: t.id.clone() })
}

/// Mostra come verrebbero il codice e il nome della cartella, senza scrivere nulla.
pub fn anteprima(ctx: &Contesto, d: &NuovaCommessa) -> Res<Anteprima> {
    anteprima_cfg(ctx, &ctx.cfg(), d)
}

/// Come `anteprima`, ma con una configurazione ancora da salvare (la scheda Personalizza la usa per mostrare l'effetto in diretta).
pub fn anteprima_cfg(ctx: &Contesto, cfg: &Config, d: &NuovaCommessa) -> Res<Anteprima> {
    cfg.valida()?;
    let mut d2 = NuovaCommessa { tipo: d.tipo.clone(), anno: d.anno.clone(), num: d.num, cliente: d.cliente.clone(), ente: d.ente.clone(), ogg: d.ogg.clone(), ..Default::default() };
    if d2.cliente.as_deref().map(|c| c.is_empty()).unwrap_or(true) && d2.ente.is_none() {
        d2.ente = Some("Cliente".into());
    }
    if d2.ogg.trim().is_empty() {
        d2.ogg = "Oggetto".into();
    }
    let usa_stile_cliente = d2.cliente.as_deref().map(|c| !c.is_empty()).unwrap_or(false);
    if usa_stile_cliente && clienti::trova(ctx, d2.cliente.as_deref().unwrap()).is_none() {
        d2.cliente = None;
        d2.ente = Some("Cliente".into());
    }
    let p = prepara(ctx, cfg, &d2)?;
    Ok(Anteprima { codice: p.codice, nome: p.nome })
}

pub fn prossimo(ctx: &Contesto, tipo: &str, anno: &str) -> Res<u64> {
    let cfg = ctx.cfg();
    let sigla = cfg.tipo(tipo)?.sigla.clone();
    registro::prossimo_numero(&cfg, anno, &sigla)
}

pub fn nuova_commessa(ctx: &Contesto, d: &NuovaCommessa) -> Res<RisultatoNuova> {
    let cfg = ctx.cfg();
    let p = prepara(ctx, &cfg, d)?;
    let t = cfg.tipo(&p.tipo_id)?.clone();
    let cartella = cfg.radice().join(&t.dir).join(&p.nome);
    let _g = ctx.blocca();
    if registro::leggi(&cfg)?.iter().any(|r| r.get("Codice") == Some(&p.codice)) {
        return err(format!("Il codice {} è già nel registro", p.codice));
    }
    if cartella.exists() {
        return err("La cartella esiste già");
    }
    for k in d.extra.keys() {
        if !cfg.registro.colonne_extra.contains(k) {
            return err(format!("Campo sconosciuto: {k}"));
        }
    }
    let mut azioni = vec![az("crea cartella", cartella.display())];
    for s in &t.sub {
        azioni.push(az("crea cartella", cartella.join(s).display()));
    }
    if cfg.nomenclatura.crea_readme {
        azioni.push(az("crea file", cartella.join(&cfg.nomenclatura.nome_readme).display()));
    }
    azioni.push(az("aggiungi al registro", cfg.file_registro().display()));
    if !d.simula {
        std::fs::create_dir_all(&cartella)?;
        for s in &t.sub {
            std::fs::create_dir_all(cartella.join(s.replace('\\', "/")))?;
        }
        let data = oggi();
        if cfg.nomenclatura.crea_readme {
            let modello = if t.readme.trim().is_empty() { readme_predefinito(&cfg.app.lingua) } else { t.readme.as_str() };
            let testo = modello
                .replace("{codice}", &p.codice)
                .replace("{cartella}", &p.nome)
                .replace("{ente}", &p.ente_esteso)
                .replace("{oggetto}", &p.ogg_esteso)
                .replace("{tipo}", &t.nome)
                .replace("{data}", &data)
                .replace("{stato}", cfg.registro.stati.first().map(|s| s.as_str()).unwrap_or(""))
                .replace("{responsabile}", d.responsabile.trim())
                .replace("{riesame}", if d.riesame { data.as_str() } else { "da eseguire" })
                .replace("{versioni}", &cfg.esempio_versioni());
            std::fs::write(cartella.join(&cfg.nomenclatura.nome_readme), testo)?;
        }
        let mut riga = Riga::new();
        for (k, v) in [
            ("Codice", p.codice.as_str()),
            ("Cartella", p.nome.as_str()),
            ("Tipo", t.nome.as_str()),
            ("Ente", p.ente_esteso.as_str()),
            ("Oggetto", p.ogg_esteso.as_str()),
            ("Stato", cfg.registro.stati.first().map(|s| s.as_str()).unwrap_or("")),
            ("Aperta il", data.as_str()),
            ("Breve", p.breve.as_str()),
            ("Responsabile", d.responsabile.trim()),
            ("Revisione", "00"),
            ("Riesame il", if d.riesame { data.as_str() } else { "" }),
        ] {
            riga.insert(k.into(), v.into());
        }
        riga.insert("Percorso".into(), PathBuf::from(&t.dir).join(&p.nome).to_string_lossy().replace('\\', "/"));
        for (k, v) in &d.extra {
            riga.insert(k.clone(), v.trim().into());
        }
        registro::aggiungi(&cfg, riga)?;
        eventi::registra(&cfg, "Commessa creata", &p.codice, &p.nome);
    }
    Ok(RisultatoNuova { codice: p.codice, nome: p.nome, azioni })
}

// ------------------------------------------------------------------ mappa di riordino

pub type RigaMappa = [String; 3];

pub fn carica_mappa(ctx: &Contesto) -> Vec<RigaMappa> {
    let Ok(t) = std::fs::read_to_string(ctx.file_mappa()) else { return vec![] };
    let Ok(v) = serde_json::from_str::<Vec<Vec<String>>>(&t) else { return vec![] };
    v.into_iter().filter(|r| r.len() >= 2).map(|r| [r[0].clone(), r[1].clone(), r.get(2).cloned().unwrap_or_default()]).collect()
}

fn salva_mappa(ctx: &Contesto, righe: &[RigaMappa]) -> Res<()> {
    scrivi_atomico(&ctx.file_mappa(), serde_json::to_string_pretty(righe)?.as_bytes())
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct ModificaMappa {
    pub azione: String,
    pub indice: Option<usize>,
    pub vecchio: String,
    pub nuovo: String,
    pub nota: String,
}

/// Aggiunge o toglie una riga della mappa. Le righe sono percorsi relativi.
pub fn modifica_mappa(ctx: &Contesto, d: &ModificaMappa) -> Res<usize> {
    let _g = ctx.blocca();
    let mut righe = carica_mappa(ctx);
    if d.azione == "elimina" {
        let i = d.indice.ok_or_else(|| Errore("Riga non trovata".into()))?;
        if i >= righe.len() {
            return err("Riga non trovata");
        }
        righe.remove(i);
    } else {
        let (v, n) = (d.vecchio.trim(), d.nuovo.trim());
        percorso_relativo(v)?;
        percorso_relativo(n)?;
        if righe.iter().any(|r| r[0] == v) {
            return err("Questa origine è già in mappa");
        }
        righe.push([v.into(), n.into(), d.nota.trim().into()]);
    }
    salva_mappa(ctx, &righe)?;
    Ok(righe.len())
}

#[derive(Serialize, Debug)]
pub struct StatoMappa {
    pub i: usize,
    pub vecchio: String,
    pub nuovo: String,
    pub nota: String,
    pub stato: String,
}

pub fn stato_mappa(ctx: &Contesto) -> Vec<StatoMappa> {
    let cfg = ctx.cfg();
    carica_mappa(ctx)
        .into_iter()
        .enumerate()
        .map(|(i, [vecchio, nuovo, nota])| {
            let stato = match (percorso_relativo(&vecchio), percorso_relativo(&nuovo)) {
                (Ok(v), Ok(n)) => {
                    let (s, d) = (cfg.origine().join(v), cfg.radice().join(n));
                    if d.exists() {
                        if s.exists() { "conflitto" } else { "fatto" }
                    } else if s.exists() {
                        "pronto"
                    } else {
                        "manca"
                    }
                }
                _ => "errore",
            };
            StatoMappa { i, vecchio, nuovo, nota, stato: stato.into() }
        })
        .collect()
}

#[derive(Serialize, Debug)]
pub struct RisultatoSposta {
    pub esiti: Vec<Esito>,
    pub giornale: Option<String>,
}

pub fn riordina(ctx: &Contesto, indici: &[usize], simula: bool) -> Res<RisultatoSposta> {
    let cfg = ctx.cfg();
    let mappa = carica_mappa(ctx);
    let _g = ctx.blocca();
    let (mut voci, mut esiti): (Vec<Voce>, Vec<Esito>) = (vec![], vec![]);
    for &i in indici {
        let Some([vecchio, nuovo, _]) = mappa.get(i) else { continue };
        match (percorso_relativo(vecchio), percorso_relativo(nuovo)) {
            (Ok(v), Ok(n)) => {
                let (s, d) = (cfg.origine().join(v), cfg.radice().join(n));
                let (es, msg) = giornale::sposta(&s, &d, simula, &mut voci, &cfg.origine(), &cfg.radice());
                esiti.push(Esito::nuovo(&es, s.display(), d.display(), msg));
            }
            (a, b) => {
                let m = a.err().or(b.err()).map(|e| e.0).unwrap_or_default();
                esiti.push(Esito::nuovo("errore", vecchio, nuovo, m));
            }
        }
    }
    let fatti = voci.len();
    let nome = if simula { None } else { giornale::salva(ctx, "riordina", voci, None)? };
    if nome.is_some() {
        eventi::registra(&cfg, "Riordino", "", &format!("{fatti} elementi spostati"));
    }
    Ok(RisultatoSposta { esiti, giornale: nome })
}

// ------------------------------------------------------------------ archiviazione

fn anno_della_riga(cfg: &Config, r: &Riga) -> String {
    if let (Ok(re), Some(c)) = (nomi::regex_da_modello(&cfg.nomenclatura.codice, None), r.get("Codice")) {
        if let Some(a) = re.captures(c).and_then(|c| c.name("anno").map(|m| m.as_str().to_string())) {
            return a;
        }
    }
    r.get("Aperta il").map(|d| d.chars().take(4).collect()).filter(|a: &String| a.len() == 4).unwrap_or_else(|| crate::util::anno_corrente().to_string())
}

/// Sposta una commessa chiusa nell'archivio (per anno, se scelto) e aggiorna il registro.
pub fn archivia(ctx: &Contesto, codice: &str, simula: bool) -> Res<RisultatoSposta> {
    let cfg = ctx.cfg();
    let _g = ctx.blocca();
    let mut righe = registro::leggi(&cfg)?;
    let r = registro::trova(&mut righe, codice)?;
    if r.get("Stato") != Some(&cfg.registro.stato_chiuso) {
        return err(format!("Si archivia solo da stato «{}» ({})", cfg.registro.stato_chiuso, cfg.vocabolario.commesse));
    }
    let percorso = r.get("Percorso").cloned().unwrap_or_default();
    let cartella = r.get("Cartella").cloned().unwrap_or_default();
    let src = cfg.radice().join(percorso_relativo(&percorso)?);
    let mut rel = PathBuf::from(&cfg.cartelle.archivio);
    if cfg.nomenclatura.archivio_per_anno {
        rel.push(anno_della_riga(&cfg, r));
    }
    rel.push(&cartella);
    let dst = cfg.radice().join(&rel);
    let mut voci = vec![];
    if simula {
        let (es, msg) = giornale::sposta(&src, &dst, true, &mut voci, &cfg.radice(), &cfg.radice());
        return Ok(RisultatoSposta { esiti: vec![Esito::nuovo(&es, src.display(), dst.display(), msg)], giornale: None });
    }
    let prima = PrimaRegistro { codice: codice.into(), percorso: percorso.clone(), stato: cfg.registro.stato_chiuso.clone(), conserva: r.get("Conserva fino al").cloned().unwrap_or_default() };
    if cfg.qualita.conservazione_anni > 0 {
        r.insert("Conserva fino al".into(), crate::util::aggiungi_anni(&crate::util::oggi(), cfg.qualita.conservazione_anni));
    }
    r.insert("Percorso".into(), rel.to_string_lossy().replace('\\', "/"));
    r.insert("Stato".into(), cfg.registro.stato_archiviato.clone());
    registro::riscrivi(&cfg, &righe)?; // se il registro è bloccato ci si ferma prima di spostare
    let (es, msg) = giornale::sposta(&src, &dst, false, &mut voci, &cfg.radice(), &cfg.radice());
    if es != "ok" {
        let r = registro::trova(&mut righe, codice)?;
        r.insert("Percorso".into(), prima.percorso.clone());
        r.insert("Stato".into(), prima.stato.clone());
        r.insert("Conserva fino al".into(), prima.conserva.clone());
        registro::riscrivi(&cfg, &righe)?;
        return Ok(RisultatoSposta { esiti: vec![Esito::nuovo(&es, src.display(), dst.display(), msg)], giornale: None });
    }
    let nome = giornale::salva(ctx, "archivia", voci, Some(vec![prima]))?;
    eventi::registra(&cfg, "Archiviata", codice, &rel.to_string_lossy().replace('\\', "/"));
    Ok(RisultatoSposta { esiti: vec![Esito::nuovo("ok", src.display(), dst.display(), "")], giornale: nome })
}
