//! Configurazione personalizzabile: marchio, tema, vocabolario, cartelle, nomenclature, tipi di progetto, registro, pulizia.
use crate::errore::{err, Errore, Res};
use crate::nomi::{self, CAMPI_CARTELLA, CAMPI_CODICE};
use crate::util::{colore_valido, nome_semplice_valido, scrivi_atomico, sottopercorso_valido, STILI_NOME};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

pub const SCHEMA: u32 = 1;
pub const README_PREDEFINITO: &str = "# {codice} - {ente} - {oggetto}\n\nTipo: {tipo}\nAperta il: {data}\nStato: {stato}\nResponsabile: {responsabile}\nRiesame dei requisiti: {riesame}\nRiferimento preventivo/fattura: \nReferente: \n\nRegole\n- Gli input del cliente (proprietà del cliente, ISO 9001 8.5.3) non si modificano.\n- Prima della consegna si verifica e si approva (8.6): l'approvazione si registra nel registro.\n- Le consegne si congelano; versioni {versioni}, senza _FINAL. Una modifica ai requisiti apre una nuova revisione (8.5.6).\n- Ogni difetto o reclamo si registra come non conformità (8.7, 10.2).\n";
pub const README_PREDEFINITO_EN: &str = "# {codice} - {ente} - {oggetto}\n\nType: {tipo}\nOpened on: {data}\nStatus: {stato}\nPerson in charge: {responsabile}\nRequirements review: {riesame}\nQuote/invoice reference: \nContact person: \n\nRules\n- Client inputs (customer property, ISO 9001 8.5.3) are never modified.\n- Before delivery the work is verified and approved (8.6): the approval is recorded in the register.\n- Deliveries are frozen; versions {versioni}, never _FINAL. A change to the requirements opens a new revision (8.5.6).\n- Every defect or complaint is recorded as a nonconformity (8.7, 10.2).\n";

pub fn readme_predefinito(lingua: &str) -> &'static str {
    if lingua == "en" { README_PREDEFINITO_EN } else { README_PREDEFINITO }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Marchio {
    pub nome: String,
    pub sottotitolo: String,
    /// Lingua dell'interfaccia e dei documenti generati: "it" o "en"
    pub lingua: String,
}
impl Default for Marchio {
    fn default() -> Self {
        Marchio {
            lingua: "it".into(),
            nome: "Folio".into(),
            sottotitolo: "Codici, cartelle, clienti e archivio. Il programma sposta file ma non ne elimina mai; ogni operazione parte in simulazione e si può annullare.".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Tema {
    /// "auto" (segue il sistema), "chiaro", "scuro"
    pub modo: String,
    /// Colore principale `#rrggbb`. La variante per il tema scuro e il colore del testo sui pulsanti si ricavano da questo.
    pub accento: String,
    /// "sistema", "serif", "mono"
    pub carattere: String,
    /// Titoli con il carattere graffato (serif)
    pub titoli_serif: bool,
    /// Raggio degli angoli in px (0-20)
    pub raggio: u8,
    pub compatto: bool,
}
impl Default for Tema {
    fn default() -> Self {
        Tema { modo: "auto".into(), accento: "#1f5f55".into(), carattere: "sistema".into(), titoli_serif: true, raggio: 6, compatto: false }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Vocabolario {
    pub commessa: String,
    pub commesse: String,
    pub cliente: String,
    pub clienti: String,
    /// "f" o "m": serve per concordare «Nuova commessa» / «Nuovo progetto»
    pub genere_commessa: String,
    pub genere_cliente: String,
}
impl Default for Vocabolario {
    fn default() -> Self {
        Vocabolario {
            commessa: "commessa".into(),
            commesse: "commesse".into(),
            cliente: "cliente".into(),
            clienti: "clienti".into(),
            genere_commessa: "f".into(),
            genere_cliente: "m".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Cartelle {
    pub radice: String,
    pub origine: String,
    pub radici_scansione: Vec<String>,
    pub gestione: String,
    pub archivio: String,
    pub prefisso_quarantena: String,
}
impl Default for Cartelle {
    fn default() -> Self {
        let h = home_utente();
        Cartelle {
            radice: h.join("Commesse").to_string_lossy().to_string(),
            origine: h.join("Desktop").to_string_lossy().to_string(),
            radici_scansione: vec![],
            gestione: "00_GESTIONE".into(),
            archivio: "90_ARCHIVIO".into(),
            prefisso_quarantena: "_QUARANTENA_".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Nomenclatura {
    /// Campi: {anno} {num} {sigla}. Deve contenere {num}.
    pub codice: String,
    /// Campi: {codice} {anno} {num} {sigla} {tipo} {ente} {oggetto} {data}. Deve contenere {codice}.
    pub cartella: String,
    pub cifre_numero: u8,
    /// La numerazione riparte da 1 a ogni anno (se il codice contiene {anno})
    pub azzera_ogni_anno: bool,
    /// Ogni tipo ha una numerazione propria (se il codice contiene {sigla})
    pub numerazione_per_tipo: bool,
    /// pascal, snake, kebab, maiuscolo
    pub stile_nomi: String,
    pub versione_prefisso: String,
    pub versione_cifre: u8,
    pub crea_readme: bool,
    pub nome_readme: String,
    pub archivio_per_anno: bool,
}
impl Default for Nomenclatura {
    fn default() -> Self {
        Nomenclatura {
            codice: "{anno}-{num}".into(),
            cartella: "{codice}_{ente}_{oggetto}".into(),
            cifre_numero: 3,
            azzera_ogni_anno: true,
            numerazione_per_tipo: false,
            stile_nomi: "pascal".into(),
            versione_prefisso: "v".into(),
            versione_cifre: 2,
            crea_readme: true,
            nome_readme: "README.md".into(),
            archivio_per_anno: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Tipo {
    /// Identificativo interno (minuscole, cifre, _): non cambiarlo dopo l'uso, i clienti lo richiamano
    pub id: String,
    pub nome: String,
    /// Cartella di primo livello dentro la radice
    pub dir: String,
    /// Sigla per il codice, se il modello la usa ({sigla})
    pub sigla: String,
    /// Sottocartelle create in ogni progetto di questo tipo
    pub sub: Vec<String>,
    /// Etichetta del campo "cliente" per questo tipo (es. Ente, Committente)
    pub etichetta_ente: String,
    /// Modello del file README; vuoto = modello generale
    pub readme: String,
    /// Colore del tipo (facoltativo) per le etichette
    pub colore: String,
}
impl Default for Tipo {
    fn default() -> Self {
        Tipo { id: String::new(), nome: String::new(), dir: String::new(), sigla: String::new(), sub: vec![], etichetta_ente: "Cliente".into(), readme: String::new(), colore: String::new() }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct ImpRegistro {
    pub stati: Vec<String>,
    pub stato_chiuso: String,
    pub stato_archiviato: String,
    /// Colonne aggiuntive del registro (es. Importo, CIG, Referente)
    pub colonne_extra: Vec<String>,
}
impl Default for ImpRegistro {
    fn default() -> Self {
        ImpRegistro { stati: vec!["Aperta".into(), "Sospesa".into(), "Chiusa".into()], stato_chiuso: "Chiusa".into(), stato_archiviato: "Archiviata".into(), colonne_extra: vec![] }
    }
}

/// Controlli di qualità ispirati alla ISO 9001:2015.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Qualita {
    /// Anni di conservazione dopo l'archiviazione (7.5.3). 0 = non indicato.
    pub conservazione_anni: u32,
    /// Una commessa si chiude solo dopo l'approvazione (8.6, rilascio degli output)
    pub richiedi_approvazione: bool,
    /// Registro cronologico delle operazioni (7.5.3)
    pub registra_eventi: bool,
}
impl Default for Qualita {
    fn default() -> Self {
        Qualita { conservazione_anni: 10, richiedi_approvazione: true, registra_eventi: true }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct ImpPulizia {
    pub soglia_grande_mb: u64,
    pub esclusi: Vec<String>,
    /// Categorie sicure da non considerare (id: cache_python, cache_ide, checkpoint_jupyter, sistema, lock_office, temporanei, aux_xml)
    pub disattivate: Vec<String>,
    /// Parti di nome file da segnalare (senza spostarli), es. «bozza», «old»
    pub parole_da_verificare: Vec<String>,
}
impl Default for ImpPulizia {
    fn default() -> Self {
        ImpPulizia { soglia_grande_mb: 500, esclusi: vec![".git".into(), "node_modules".into(), ".venv".into(), "venv".into()], disattivate: vec![], parole_da_verificare: vec![] }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    pub schema: u32,
    pub app: Marchio,
    pub tema: Tema,
    pub vocabolario: Vocabolario,
    pub cartelle: Cartelle,
    pub nomenclatura: Nomenclatura,
    pub tipi: Vec<Tipo>,
    pub registro: ImpRegistro,
    pub qualita: Qualita,
    pub pulizia: ImpPulizia,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            schema: SCHEMA,
            app: Marchio::default(),
            tema: Tema::default(),
            vocabolario: Vocabolario::default(),
            cartelle: Cartelle::default(),
            nomenclatura: Nomenclatura::default(),
            tipi: tipi_predefiniti(),
            registro: ImpRegistro::default(),
            qualita: Qualita::default(),
            pulizia: ImpPulizia::default(),
        }
    }
}

pub fn tipi_predefiniti() -> Vec<Tipo> {
    let t = |id: &str, nome: &str, dir: &str, sigla: &str, sub: &[&str], et: &str| Tipo {
        id: id.into(),
        nome: nome.into(),
        dir: dir.into(),
        sigla: sigla.into(),
        sub: sub.iter().map(|s| s.to_string()).collect(),
        etichetta_ente: et.into(),
        ..Tipo::default()
    };
    vec![
        t("cliente", "Cliente", "10_CLIENTI", "CLI", &["01_Offerta_Riesame", "02_Input_Cliente", "03_Lavoro", "04_Verifica_Validazione", "05_Output_Consegne", "06_Comunicazioni", "07_Amministrazione"], "Cliente"),
        t("ricerca", "Ricerca", "20_RICERCA", "RIC", &["01_Protocollo", "02_Dati_Input", "03_Analisi", "04_Verifica", "05_Output", "06_Manoscritti", "07_Riferimenti"], "Progetto"),
        t("sviluppo", "Sviluppo", "30_SVILUPPO", "SVI", &["01_Requisiti_Input", "02_Pianificazione", "03_Codice_Repo", "04_Verifiche_Test", "05_Documentazione", "06_Release_Consegne", "07_Modifiche"], "Committente"),
        t("istituzione", "Istituzione", "40_ISTITUZIONI", "IST", &["01_Mandato_Riesame", "02_Input", "03_Lavoro", "04_Verifica_Validazione", "05_Output_Consegne", "06_Comunicazioni", "07_Amministrazione"], "Ente"),
        t("associazione", "Associazione", "50_ASSOCIAZIONI", "ASS", &["01_Documenti", "02_Materiali", "03_Output", "04_Comunicazioni"], "Associazione"),
    ]
}

pub fn home_utente() -> PathBuf {
    let v = if cfg!(windows) { std::env::var_os("USERPROFILE") } else { std::env::var_os("HOME") };
    v.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

/// Dove l'app tiene configurazione, anagrafica clienti, mappa e giornale degli spostamenti.
pub fn cartella_dati() -> PathBuf {
    if let Some(e) = std::env::var_os("COMMESSE_HOME") {
        if !e.is_empty() {
            return PathBuf::from(e);
        }
    }
    let h = home_utente();
    if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| h.join("AppData").join("Roaming")).join("commesse")
    } else if cfg!(target_os = "macos") {
        h.join("Library").join("Application Support").join("commesse")
    } else {
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| h.join(".config")).join("commesse")
    }
}

impl Config {
    pub fn tipo(&self, id: &str) -> Res<&Tipo> {
        self.tipi.iter().find(|t| t.id == id).ok_or_else(|| Errore(format!("Tipo non valido: {id}")))
    }
    pub fn radice(&self) -> PathBuf {
        PathBuf::from(self.cartelle.radice.trim())
    }
    pub fn origine(&self) -> PathBuf {
        PathBuf::from(self.cartelle.origine.trim())
    }
    pub fn cartella_gestione(&self) -> PathBuf {
        self.radice().join(&self.cartelle.gestione)
    }
    pub fn cartella_archivio(&self) -> PathBuf {
        self.radice().join(&self.cartelle.archivio)
    }
    pub fn file_registro(&self) -> PathBuf {
        self.cartella_gestione().join("registro_commesse.csv")
    }
    /// Esempio di nomi di versione, per i README: «v01, v02»
    pub fn esempio_versioni(&self) -> String {
        let w = self.nomenclatura.versione_cifre as usize;
        let p = &self.nomenclatura.versione_prefisso;
        format!("{p}{:0>w$}, {p}{:0>w$}", 1, 2, w = w)
    }

    /// Controlla tutta la configurazione. Il messaggio di errore dice cosa correggere.
    pub fn valida(&self) -> Res<()> {
        if !["it", "en"].contains(&self.app.lingua.as_str()) {
            return err("Lingua non valida (it, en)");
        }
        if self.app.nome.trim().is_empty() || self.app.nome.chars().count() > 60 {
            return err("Il nome dell'applicazione deve avere da 1 a 60 caratteri");
        }
        if !["auto", "chiaro", "scuro"].contains(&self.tema.modo.as_str()) {
            return err("Tema: modo non valido (auto, chiaro, scuro)");
        }
        if !colore_valido(&self.tema.accento) {
            return err("Tema: il colore principale deve essere nel formato #rrggbb");
        }
        if !["sistema", "serif", "mono"].contains(&self.tema.carattere.as_str()) {
            return err("Tema: carattere non valido (sistema, serif, mono)");
        }
        if self.tema.raggio > 20 {
            return err("Tema: il raggio degli angoli va da 0 a 20");
        }
        for (n, v) in [("commessa", &self.vocabolario.commessa), ("commesse", &self.vocabolario.commesse), ("cliente", &self.vocabolario.cliente), ("clienti", &self.vocabolario.clienti)] {
            if v.trim().is_empty() || v.chars().count() > 30 {
                return err(format!("Vocabolario: «{n}» deve avere da 1 a 30 caratteri"));
            }
        }
        for (n, v) in [("commessa", &self.vocabolario.genere_commessa), ("cliente", &self.vocabolario.genere_cliente)] {
            if v != "f" && v != "m" {
                return err(format!("Vocabolario: il genere di «{n}» deve essere f o m"));
            }
        }
        for (n, v) in [("gestione", &self.cartelle.gestione), ("archivio", &self.cartelle.archivio)] {
            if !nome_semplice_valido(v) {
                return err(format!("Cartella «{n}»: nome non valido (niente / \\ : * ? \" < > |)"));
            }
        }
        if self.cartelle.gestione.eq_ignore_ascii_case(&self.cartelle.archivio) {
            return err("Le cartelle di gestione e di archivio devono avere nomi diversi");
        }
        if self.cartelle.prefisso_quarantena.is_empty() || self.cartelle.prefisso_quarantena.chars().any(|c| "<>:\"/\\|?*".contains(c)) {
            return err("Prefisso della quarantena non valido");
        }
        let n = &self.nomenclatura;
        nomi::valida(&n.codice, &CAMPI_CODICE, &["num"])?;
        nomi::valida(&n.cartella, &CAMPI_CARTELLA, &["codice"])?;
        if !(1..=6).contains(&n.cifre_numero) {
            return err("Le cifre del numero vanno da 1 a 6");
        }
        if !(1..=4).contains(&n.versione_cifre) {
            return err("Le cifre della versione vanno da 1 a 4");
        }
        if !STILI_NOME.contains(&n.stile_nomi.as_str()) {
            return err("Stile dei nomi non valido (pascal, snake, kebab, maiuscolo)");
        }
        if n.crea_readme && !nome_semplice_valido(&n.nome_readme) {
            return err("Nome del file README non valido");
        }
        if self.tipi.is_empty() {
            return err("Serve almeno un tipo di progetto");
        }
        let usa_sigla = nomi::ha_campo(&n.codice, "sigla") || nomi::ha_campo(&n.cartella, "sigla");
        let mut ids: Vec<&str> = vec![];
        let mut dirs: Vec<String> = vec![];
        let mut sigle: Vec<&str> = vec![];
        for t in &self.tipi {
            let ok_id = !t.id.is_empty() && t.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            if !ok_id {
                return err(format!("Tipo «{}»: l'identificativo deve usare solo minuscole, cifre e _", t.nome));
            }
            if ids.contains(&t.id.as_str()) {
                return err(format!("Identificativo di tipo ripetuto: {}", t.id));
            }
            ids.push(&t.id);
            if t.nome.trim().is_empty() {
                return err(format!("Il tipo «{}» non ha un nome", t.id));
            }
            if !nome_semplice_valido(&t.dir) {
                return err(format!("Tipo «{}»: nome cartella non valido", t.nome));
            }
            if t.dir.eq_ignore_ascii_case(&self.cartelle.gestione) || t.dir.eq_ignore_ascii_case(&self.cartelle.archivio) {
                return err(format!("Tipo «{}»: la cartella coincide con gestione o archivio", t.nome));
            }
            let d = t.dir.to_lowercase();
            if dirs.contains(&d) {
                return err(format!("Due tipi usano la stessa cartella: {}", t.dir));
            }
            dirs.push(d);
            if usa_sigla {
                if t.sigla.is_empty() || !t.sigla.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return err(format!("Tipo «{}»: serve una sigla di lettere e cifre, perché il modello usa {{sigla}}", t.nome));
                }
                if sigle.contains(&t.sigla.as_str()) {
                    return err(format!("Sigla ripetuta: {}", t.sigla));
                }
                sigle.push(&t.sigla);
            }
            for s in &t.sub {
                if !sottopercorso_valido(s) {
                    return err(format!("Tipo «{}»: sottocartella non valida «{s}»", t.nome));
                }
            }
            if !t.colore.is_empty() && !colore_valido(&t.colore) {
                return err(format!("Tipo «{}»: il colore deve essere #rrggbb", t.nome));
            }
        }
        let r = &self.registro;
        if r.stati.is_empty() || r.stati.iter().any(|s| s.trim().is_empty() || s.contains(';')) {
            return err("Stati: servono nomi non vuoti e senza «;»");
        }
        for (i, s) in r.stati.iter().enumerate() {
            if r.stati[..i].contains(s) {
                return err(format!("Stato ripetuto: {s}"));
            }
        }
        if !r.stati.contains(&r.stato_chiuso) {
            return err("Lo stato «chiuso» deve essere uno degli stati");
        }
        if r.stati.contains(&r.stato_archiviato) || r.stato_archiviato.trim().is_empty() {
            return err("Lo stato «archiviato» deve essere diverso dagli altri stati");
        }
        let riservate = crate::registro::BASE;
        for (i, c) in r.colonne_extra.iter().enumerate() {
            if c.trim().is_empty() || c.contains(';') || c.contains('\n') {
                return err("Colonne extra: nomi non vuoti e senza «;»");
            }
            if riservate.contains(&c.as_str()) {
                return err(format!("La colonna «{c}» esiste già nel registro"));
            }
            if r.colonne_extra[..i].contains(c) {
                return err(format!("Colonna extra ripetuta: {c}"));
            }
        }
        if self.qualita.conservazione_anni > 99 {
            return err("Conservazione: massimo 99 anni");
        }
        if self.pulizia.soglia_grande_mb == 0 {
            return err("La soglia dei file grandi deve essere almeno 1 MB");
        }
        Ok(())
    }
}

/// Stato condiviso: dove vivono i dati e la configurazione corrente.
pub struct Contesto {
    pub home: PathBuf,
    cfg: RwLock<Config>,
    op: Mutex<()>,
    prima_volta: RwLock<bool>,
    pub scansione: crate::pulizia::Scansione,
}

impl Contesto {
    pub fn nuovo(home: Option<PathBuf>) -> Res<Contesto> {
        let home = home.unwrap_or_else(cartella_dati);
        std::fs::create_dir_all(&home)?;
        let file = home.join("impostazioni.json");
        let prima = !file.exists();
        let cfg = if file.exists() {
            let t = std::fs::read_to_string(&file)?;
            let mut c: Config = serde_json::from_str(&t).map_err(|e| Errore(format!("{} non è leggibile: {e}", file.display())))?;
            c.schema = SCHEMA;
            c
        } else {
            migra_vecchia_configurazione(&home).unwrap_or_default()
        };
        Ok(Contesto { home, cfg: RwLock::new(cfg), op: Mutex::new(()), prima_volta: RwLock::new(prima), scansione: crate::pulizia::Scansione::nuova() })
    }

    pub fn cfg(&self) -> Config {
        self.cfg.read().unwrap().clone()
    }
    pub fn prima_volta(&self) -> bool {
        *self.prima_volta.read().unwrap()
    }
    /// Serializza le operazioni che modificano file o registro.
    pub fn blocca(&self) -> std::sync::MutexGuard<'_, ()> {
        self.op.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn file_impostazioni(&self) -> PathBuf {
        self.home.join("impostazioni.json")
    }
    pub fn file_clienti(&self) -> PathBuf {
        self.home.join("clienti.json")
    }
    pub fn file_mappa(&self) -> PathBuf {
        self.home.join("mappa.json")
    }
    pub fn cartella_giornale(&self) -> PathBuf {
        self.home.join("journal")
    }

    /// Valida e salva la configurazione completa.
    pub fn salva_config(&self, mut nuova: Config) -> Res<Config> {
        nuova.schema = SCHEMA;
        for s in nuova.cartelle.radici_scansione.iter_mut() {
            *s = s.trim().to_string();
        }
        nuova.cartelle.radici_scansione.retain(|s| !s.is_empty());
        nuova.cartelle.radice = nuova.cartelle.radice.trim().to_string();
        nuova.cartelle.origine = nuova.cartelle.origine.trim().to_string();
        if nuova.cartelle.radice.is_empty() {
            return err("La cartella delle commesse non può essere vuota");
        }
        if nuova.cartelle.origine.is_empty() {
            return err("La cartella da riordinare non può essere vuota");
        }
        nuova.valida()?;
        let _g = self.blocca();
        scrivi_atomico(&self.file_impostazioni(), serde_json::to_string_pretty(&nuova)?.as_bytes())?;
        let cambiata = *self.cfg.read().unwrap() != nuova;
        *self.cfg.write().unwrap() = nuova.clone();
        if cambiata {
            crate::eventi::registra(&nuova, "Configurazione modificata", "", "");
        }
        *self.prima_volta.write().unwrap() = false;
        Ok(nuova)
    }
}

/// Le versioni Python salvavano `config.json` con `tipi` come dizionario.
fn migra_vecchia_configurazione(home: &Path) -> Option<Config> {
    let t = std::fs::read_to_string(home.join("config.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&t).ok()?;
    let mut c = Config::default();
    if let Some(r) = v.get("radice").and_then(|x| x.as_str()) {
        c.cartelle.radice = r.into();
    }
    if let Some(o) = v.get("origine").or_else(|| v.get("desktop")).and_then(|x| x.as_str()) {
        c.cartelle.origine = o.into();
    }
    if let Some(a) = v.get("radici_scansione").and_then(|x| x.as_array()) {
        c.cartelle.radici_scansione = a.iter().filter_map(|x| x.as_str().map(String::from)).collect();
    }
    if let Some(m) = v.get("tipi").and_then(|x| x.as_object()) {
        let mut tipi = vec![];
        for (id, d) in m {
            let sub = d.get("sub").and_then(|s| s.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            let sigla: String = id.chars().take(3).collect::<String>().to_uppercase();
            tipi.push(Tipo {
                id: id.clone(),
                nome: d.get("nome").and_then(|x| x.as_str()).unwrap_or(id).into(),
                dir: d.get("dir").and_then(|x| x.as_str()).unwrap_or(id).into(),
                sigla,
                sub,
                ..Tipo::default()
            });
        }
        if !tipi.is_empty() {
            c.tipi = tipi;
        }
    }
    if c.valida().is_ok() {
        Some(c)
    } else {
        None
    }
}
