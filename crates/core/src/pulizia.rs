//! Ricerca dei file inutili e quarantena. Niente viene eliminato: i file sicuri si spostano in una cartella datata.
use crate::config::{Config, Contesto};
use crate::errore::{err, Res};
use crate::giornale::{self, Esito, Voce};
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub const CAT_SICURE: [(&str, &str); 7] = [
    ("cache_python", "Cache Python"),
    ("cache_ide", "Cache IDE (.idea)"),
    ("checkpoint_jupyter", "Checkpoint Jupyter"),
    ("sistema", "File di sistema"),
    ("lock_office", "Lock di Office (~$)"),
    ("temporanei", "Temporanei (.tmp)"),
    ("aux_xml", "Sidecar GDAL (.aux.xml)"),
];
pub const CAT_VERIFICA: [(&str, &str); 6] = [
    ("download", "Download di Drive/OneDrive"),
    ("copia", "Copie e backup"),
    ("zip_estratto", "Zip già estratto accanto"),
    ("zip", "Zip"),
    ("grande", "File oltre la soglia"),
    ("parole", "Nomi da verificare (regole personali)"),
];

fn nome_cat(id: &str) -> &'static str {
    CAT_SICURE.iter().chain(CAT_VERIFICA.iter()).find(|(i, _)| *i == id).map(|(_, n)| *n).unwrap_or("")
}
fn e_sicura(id: &str) -> bool {
    CAT_SICURE.iter().any(|(i, _)| *i == id)
}

pub struct Regole {
    down: Regex,
    copia: Regex,
    soglia: u64,
    parole: Vec<String>,
    disattivate: Vec<String>,
}

impl Regole {
    pub fn da_config(cfg: &Config) -> Regole {
        Regole {
            down: Regex::new(r"(?i)^(drive-download|OneDrive_)|-00\d(\.|$)").unwrap(),
            copia: Regex::new(r"(?i)\(\d\)|_BACKUP|\.bak$|copy").unwrap(),
            soglia: cfg.pulizia.soglia_grande_mb * 1024 * 1024,
            parole: cfg.pulizia.parole_da_verificare.iter().map(|p| p.to_lowercase()).filter(|p| !p.is_empty()).collect(),
            disattivate: cfg.pulizia.disattivate.clone(),
        }
    }
    fn attiva(&self, id: &'static str) -> Option<&'static str> {
        if self.disattivate.iter().any(|d| d == id) { None } else { Some(id) }
    }

    pub fn classifica_file(&self, nome: &str, cartella: &Path, size: u64) -> Option<&'static str> {
        let low = nome.to_lowercase();
        let id = if ["desktop.ini", "thumbs.db", ".ds_store"].contains(&low.as_str()) {
            "sistema"
        } else if nome.starts_with("~$") {
            "lock_office"
        } else if low.ends_with(".pyc") {
            "cache_python"
        } else if low.ends_with(".tmp") {
            "temporanei"
        } else if low.ends_with(".aux.xml") {
            "aux_xml"
        } else if self.down.is_match(nome) {
            "download"
        } else if self.copia.is_match(nome) {
            "copia"
        } else if low.ends_with(".zip") {
            if cartella.join(&nome[..nome.len() - 4]).is_dir() { "zip_estratto" } else { "zip" }
        } else if size > self.soglia {
            "grande"
        } else if self.parole.iter().any(|p| low.contains(p.as_str())) {
            "parole"
        } else {
            return None;
        };
        self.attiva(id)
    }

    pub fn classifica_cartella(&self, nome: &str) -> Option<&'static str> {
        let id = match nome {
            "__pycache__" => "cache_python",
            ".idea" => "cache_ide",
            ".ipynb_checkpoints" => "checkpoint_jupyter",
            _ if self.down.is_match(nome) => "download",
            _ => return None,
        };
        self.attiva(id)
    }
}

pub fn dimensione_cartella(p: &Path) -> u64 {
    WalkDir::new(p).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum()
}

#[derive(Clone, Debug)]
struct Voce2 {
    cat: &'static str,
    path: PathBuf,
    size: u64,
    dir: bool,
}

#[derive(Default)]
struct Stato {
    in_corso: bool,
    fine: bool,
    visti: u64,
    cat: BTreeMap<&'static str, (u64, u64)>,
    items: Vec<Voce2>,
    csv: Option<String>,
    errore: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Categoria {
    pub id: String,
    pub nome: String,
    pub sicura: bool,
    pub n: u64,
    pub bytes: u64,
}

#[derive(Serialize, Debug)]
pub struct StatoScansione {
    pub in_corso: bool,
    pub fine: bool,
    pub visti: u64,
    pub csv: Option<String>,
    pub errore: Option<String>,
    pub categorie: Vec<Categoria>,
}

/// Scansione di sola lettura, eseguita in un thread. Lo stato si interroga con `stato()`.
#[derive(Clone)]
pub struct Scansione {
    inner: Arc<Mutex<Stato>>,
}

impl Scansione {
    pub fn nuova() -> Scansione {
        Scansione { inner: Arc::new(Mutex::new(Stato::default())) }
    }

    /// Avvia la scansione in un thread. Non fa nulla se ce n'è già una in corso.
    pub fn avvia(&self, cfg: Config) {
        {
            let mut s = self.inner.lock().unwrap();
            if s.in_corso {
                return;
            }
            *s = Stato { in_corso: true, ..Stato::default() };
        }
        let me = self.clone();
        std::thread::spawn(move || me.esegui(&cfg));
    }

    /// Corpo della scansione (sincrono: usato dal thread e dai test).
    pub fn esegui(&self, cfg: &Config) {
        {
            let mut s = self.inner.lock().unwrap();
            *s = Stato { in_corso: true, ..Stato::default() };
        }
        let r = self.visita(cfg);
        let mut s = self.inner.lock().unwrap();
        if let Err(e) = r {
            s.errore = Some(e.0);
        }
        s.in_corso = false;
        s.fine = true;
    }

    fn registra(&self, cat: &'static str, path: PathBuf, size: u64, dir: bool) {
        let mut s = self.inner.lock().unwrap();
        let c = s.cat.entry(cat).or_insert((0, 0));
        c.0 += 1;
        c.1 += size;
        s.items.push(Voce2 { cat, path, size, dir });
    }

    fn visita(&self, cfg: &Config) -> Res<()> {
        let regole = Regole::da_config(cfg);
        for radice in &cfg.cartelle.radici_scansione {
            let base = cfg.origine().join(radice);
            if !base.exists() {
                continue;
            }
            let mut it = WalkDir::new(&base).into_iter();
            while let Some(e) = it.next() {
                let Ok(e) = e else { continue };
                if e.depth() == 0 {
                    continue;
                }
                let nome = e.file_name().to_string_lossy().to_string();
                if e.file_type().is_dir() {
                    if cfg.pulizia.esclusi.contains(&nome) || nome.starts_with(".tmp.drive") {
                        it.skip_current_dir();
                    } else if let Some(cat) = regole.classifica_cartella(&nome) {
                        self.registra(cat, e.path().to_path_buf(), dimensione_cartella(e.path()), true);
                        it.skip_current_dir();
                    }
                } else if e.file_type().is_file() {
                    let Ok(m) = e.metadata() else { continue };
                    self.inner.lock().unwrap().visti += 1;
                    let cartella = e.path().parent().unwrap_or(&base);
                    if let Some(cat) = regole.classifica_file(&nome, cartella, m.len()) {
                        self.registra(cat, e.path().to_path_buf(), m.len(), false);
                    }
                }
            }
        }
        self.scrivi_csv(cfg)
    }

    fn scrivi_csv(&self, cfg: &Config) -> Res<()> {
        let mut voci: Vec<Voce2> = self.inner.lock().unwrap().items.iter().filter(|i| !e_sicura(i.cat)).cloned().collect();
        voci.sort_by_key(|v| std::cmp::Reverse(v.size));
        let mut buf: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
        {
            let mut w = csv::WriterBuilder::new().delimiter(b';').from_writer(&mut buf);
            w.write_record(["Categoria", "GB", "Cartella", "Percorso"])?;
            for v in &voci {
                let gb = format!("{:.3}", v.size as f64 / 1024f64.powi(3));
                w.write_record([nome_cat(v.cat), &gb, if v.dir { "sì" } else { "no" }, &v.path.to_string_lossy()])?;
            }
            w.flush()?;
        }
        let out = cfg.cartella_gestione().join("verifica_pulizia.csv");
        crate::util::scrivi_atomico(&out, &buf)?;
        self.inner.lock().unwrap().csv = Some(out.to_string_lossy().into());
        Ok(())
    }

    pub fn stato(&self) -> StatoScansione {
        let s = self.inner.lock().unwrap();
        let mut categorie: Vec<Categoria> = s
            .cat
            .iter()
            .map(|(k, (n, b))| Categoria { id: k.to_string(), nome: nome_cat(k).into(), sicura: e_sicura(k), n: *n, bytes: *b })
            .collect();
        categorie.sort_by_key(|c| (!c.sicura, std::cmp::Reverse(c.bytes)));
        StatoScansione { in_corso: s.in_corso, fine: s.fine, visti: s.visti, csv: s.csv.clone(), errore: s.errore.clone(), categorie }
    }

    fn voci_sicure(&self) -> Res<Vec<PathBuf>> {
        let s = self.inner.lock().unwrap();
        if !s.fine {
            return err("Esegui prima la scansione");
        }
        Ok(s.items.iter().filter(|i| e_sicura(i.cat)).map(|i| i.path.clone()).collect())
    }
}

#[derive(Serialize, Debug)]
pub struct RisultatoQuarantena {
    pub esiti: Vec<Esito>,
    pub quarantena: String,
    pub giornale: Option<String>,
}

/// Sposta i file sicuri trovati dalla scansione in `<archivio>/_QUARANTENA_<data>`, mantenendo i percorsi.
pub fn quarantena(ctx: &Contesto, simula: bool) -> Res<RisultatoQuarantena> {
    let cfg = ctx.cfg();
    let voci_scan = ctx.scansione.voci_sicure()?;
    let q = cfg.cartella_archivio().join(format!("{}{}", cfg.cartelle.prefisso_quarantena, chrono::Local::now().format("%Y%m%d")));
    let _g = ctx.blocca();
    let (mut voci, mut esiti): (Vec<Voce>, Vec<Esito>) = (vec![], vec![]);
    for p in voci_scan {
        if !p.exists() {
            continue;
        }
        let rel = p.strip_prefix(cfg.origine()).map(|r| r.to_path_buf()).unwrap_or_else(|_| PathBuf::from(p.file_name().unwrap_or_default()));
        let dst = q.join(rel);
        let (es, msg) = giornale::sposta(&p, &dst, simula, &mut voci, &cfg.origine(), &q);
        esiti.push(Esito::nuovo(&es, p.display(), dst.display(), msg));
    }
    let fatti = voci.len();
    let nome = if simula { None } else { giornale::salva(ctx, "pulisci", voci, None)? };
    if nome.is_some() {
        crate::eventi::registra(&cfg, "Quarantena", "", &format!("{fatti} elementi spostati in {}", q.display()));
    }
    Ok(RisultatoQuarantena { esiti, quarantena: q.to_string_lossy().into(), giornale: nome })
}
