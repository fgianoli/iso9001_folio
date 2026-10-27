//! Procedura documentata generata dalla configurazione (ISO 9001 cl. 7.5.1: informazioni documentate richieste dall'organizzazione).
use crate::config::{Config, Contesto};
use crate::errore::Res;
use crate::nomi::{self, Campi};
use crate::{eventi, util};
use serde::Serialize;
use std::path::PathBuf;

/// Testo Markdown della procedura, costruito con i nomi, i modelli e le regole in uso.
pub fn testo(cfg: &Config) -> String {
    if cfg.app.lingua == "en" {
        return testo_en(cfg);
    }
    let n = &cfg.nomenclatura;
    let c = Campi { anno: util::anno_corrente().to_string(), num: 14, sigla: cfg.tipi.first().map(|t| t.sigla.clone()).unwrap_or_default(), tipo: String::new(), ente: "AcmeIdrica".into(), oggetto: "RelazioneIdraulica".into(), data: util::oggi(), codice: String::new() };
    let mut c2 = c.clone();
    c2.codice = nomi::rendi(&n.codice, &c, n.cifre_numero as usize).unwrap_or_default();
    let esempio_cartella = nomi::rendi(&n.cartella, &c2, n.cifre_numero as usize).unwrap_or_default();
    let v = &cfg.vocabolario;
    let mut s = String::new();
    s += &format!("# PR-01 Gestione di {} e registrazioni\n\n", v.commesse);
    s += &format!("Data di emissione: {}  |  Revisione: 00\n\n", util::oggi());
    s += "## 1. Scopo e campo di applicazione\n\n";
    s += &format!("Questa procedura descrive come si identificano, si organizzano, si approvano, si conservano e si archiviano: {} e relative registrazioni. È ispirata ai requisiti della ISO 9001:2015 sulle informazioni documentate (7.5), sul riesame dei requisiti (8.2.3), sull'identificazione e rintracciabilità (8.5.2), sulla proprietà del cliente (8.5.3), sul controllo delle modifiche (8.5.6), sul rilascio (8.6) e sugli output non conformi (8.7, 10.2).\n\n", v.commesse);
    s += "Non sostituisce gli altri elementi di un sistema di gestione per la qualità (politica, obiettivi, analisi dei rischi, audit interni, riesame di direzione).\n\n";
    s += "## 2. Identificazione (8.5.2)\n\n";
    s += &format!("- Codice univoco: `{}`. Esempio: **{}**.\n- Nome della cartella: `{}`. Esempio: **{}**.\n", n.codice, c2.codice, n.cartella, esempio_cartella);
    s += &format!("- La numerazione {}{}.\n- Il codice non si riusa né si modifica; la cartella e il registro lo riportano.\n\n", if n.azzera_ogni_anno && nomi::ha_campo(&n.codice, "anno") { "riparte a ogni anno" } else { "è continua" }, if n.numerazione_per_tipo && nomi::ha_campo(&n.codice, "sigla") { ", separatamente per ogni tipo" } else { "" });
    s += "## 3. Struttura delle cartelle (7.5.2, 8.5.3)\n\n";
    for t in &cfg.tipi {
        s += &format!("**{}** — `{}/`", t.nome, t.dir);
        if !t.sub.is_empty() {
            s += "\n";
            for x in &t.sub {
                s += &format!("  - `{x}/`\n");
            }
        }
        s += "\n";
    }
    s += &format!("Cartelle di servizio: `{}/` (registri) e `{}/` (archivio).\n\n", cfg.cartelle.gestione, cfg.cartelle.archivio);
    s += "Regole:\n- I documenti ricevuti dal cliente (proprietà del cliente, 8.5.3) si conservano come ricevuti e non si modificano; si lavora su copie.\n";
    s += &format!("- Le versioni dei documenti si numerano {} (es. {}); non si usano nomi come «FINALE».\n- Le consegne si congelano: dopo l'invio non si modificano; una modifica genera una nuova versione.\n\n", n.versione_prefisso.clone() + &"0".repeat(n.versione_cifre.saturating_sub(1) as usize) + "N", cfg.esempio_versioni());
    s += &format!("## 4. Ciclo di vita e stati\n\nStati: {}. L'archiviazione è possibile dallo stato «{}»; dopo l'archiviazione lo stato è «{}».\n\n", cfg.registro.stati.iter().map(|x| format!("«{x}»")).collect::<Vec<_>>().join(", "), cfg.registro.stato_chiuso, cfg.registro.stato_archiviato);
    s += "## 5. Controlli\n\n";
    s += "| Requisito | Come si soddisfa | Registrazione |\n|---|---|---|\n";
    s += "| Riesame dei requisiti (8.2.3) | Prima di iniziare, il responsabile verifica requisiti, tempi e risorse e lo registra | colonna «Riesame il» |\n";
    s += "| Responsabilità (5.3) | Ogni commessa ha un responsabile | colonna «Responsabile» |\n";
    s += "| Controllo delle modifiche (8.5.6) | Una modifica ai requisiti apre una nuova revisione con il motivo; l'approvazione precedente decade | colonna «Revisione», registro cronologico |\n";
    s += &format!("| Verifica e rilascio (8.6) | {} | colonne «Approvata da», «Approvata il» |\n", if cfg.qualita.richiedi_approvazione { "La commessa si chiude solo dopo l'approvazione di chi ne ha l'autorità" } else { "L'approvazione si registra prima della consegna" });
    s += "| Output non conformi (8.7) e azioni correttive (10.2) | Ogni difetto o reclamo si registra con causa, azione correttiva e verifica dell'efficacia; con non conformità aperte la commessa non si chiude | file `non_conformita.csv` |\n";
    s += &format!("| Conservazione e distruzione (7.5.3) | Dopo l'archiviazione i documenti si conservano {}; nulla viene eliminato dal programma | colonna «Conserva fino al» |\n", if cfg.qualita.conservazione_anni > 0 { format!("per {} anni", cfg.qualita.conservazione_anni) } else { "per il periodo stabilito dall'organizzazione".into() });
    s += &format!("| Rintracciabilità delle operazioni (7.5.3) | {} | file `registro_eventi.csv` |\n\n", if cfg.qualita.registra_eventi { "Ogni creazione, modifica, approvazione, archiviazione e annullamento viene registrata con data, ora e utente; il file si scrive solo in coda" } else { "Registro cronologico disattivato" });
    s += "## 6. Protezione e backup\n\nLe cartelle e i registri devono stare in un'area con copia di sicurezza (disco di rete o servizio cloud). Se sono sincronizzati con un servizio cloud, gli spostamenti e le eliminazioni si propagano: il programma sposta e non elimina, ma il servizio può avere un proprio cestino.\n\n";
    s += "## 7. Registrazioni\n\n";
    s += &format!("- `{}/registro_commesse.csv` — registro delle commesse\n- `{}/non_conformita.csv` — non conformità e azioni correttive\n- `{}/registro_eventi.csv` — registro cronologico delle operazioni\n- giornale degli spostamenti (cartella dei dati del programma) — consente l'annullamento\n\n", cfg.cartelle.gestione, cfg.cartelle.gestione, cfg.cartelle.gestione);
    s += "## 8. Modifiche alla procedura\n\nOgni modifica a questa procedura (nomenclature, tipi, stati) si registra con una nuova revisione del documento. Il programma, alla creazione del documento, non sovrascrive mai le revisioni precedenti.\n";
    s
}

fn testo_en(cfg: &Config) -> String {
    let n = &cfg.nomenclatura;
    let c = Campi { anno: util::anno_corrente().to_string(), num: 14, sigla: cfg.tipi.first().map(|t| t.sigla.clone()).unwrap_or_default(), tipo: String::new(), ente: "AcmeWater".into(), oggetto: "HydraulicReport".into(), data: util::oggi(), codice: String::new() };
    let mut c2 = c.clone();
    c2.codice = nomi::rendi(&n.codice, &c, n.cifre_numero as usize).unwrap_or_default();
    let esempio_cartella = nomi::rendi(&n.cartella, &c2, n.cifre_numero as usize).unwrap_or_default();
    let v = &cfg.vocabolario;
    let mut s = String::new();
    s += &format!("# PR-01 Management of {} and records\n\n", v.commesse);
    s += &format!("Issue date: {}  |  Revision: 00\n\n", util::oggi());
    s += "## 1. Purpose and scope\n\n";
    s += &format!("This procedure describes how the following are identified, organized, approved, kept and archived: {} and the related records. It is inspired by the ISO 9001:2015 requirements on documented information (7.5), review of requirements (8.2.3), identification and traceability (8.5.2), customer property (8.5.3), control of changes (8.5.6), release (8.6) and nonconforming outputs (8.7, 10.2).\n\n", v.commesse);
    s += "It does not replace the other elements of a quality management system (policy, objectives, risk analysis, internal audits, management review).\n\n";
    s += "## 2. Identification (8.5.2)\n\n";
    s += &format!("- Unique code: `{}`. Example: **{}**.\n- Folder name: `{}`. Example: **{}**.\n", n.codice, c2.codice, n.cartella, esempio_cartella);
    s += &format!("- Numbering {}{}.\n- The code is never reused or changed; the folder and the register carry it.\n\n", if n.azzera_ogni_anno && nomi::ha_campo(&n.codice, "anno") { "restarts every year" } else { "is continuous" }, if n.numerazione_per_tipo && nomi::ha_campo(&n.codice, "sigla") { ", separately for each type" } else { "" });
    s += "## 3. Folder structure (7.5.2, 8.5.3)\n\n";
    for t in &cfg.tipi {
        s += &format!("**{}** — `{}/`", t.nome, t.dir);
        if !t.sub.is_empty() {
            s += "\n";
            for x in &t.sub {
                s += &format!("  - `{x}/`\n");
            }
        }
        s += "\n";
    }
    s += &format!("Service folders: `{}/` (records) and `{}/` (archive).\n\n", cfg.cartelle.gestione, cfg.cartelle.archivio);
    s += "Rules:\n- Documents received from the client (customer property, 8.5.3) are kept as received and never modified; work is done on copies.\n";
    s += &format!("- Document versions are numbered {} (e.g. {}); names such as \"FINAL\" are not used.\n- Deliveries are frozen: after sending they are not modified; a change creates a new version.\n\n", n.versione_prefisso.clone() + &"0".repeat(n.versione_cifre.saturating_sub(1) as usize) + "N", cfg.esempio_versioni());
    s += &format!("## 4. Life cycle and statuses\n\nStatuses: {}. Archiving is possible from the status \"{}\"; after archiving the status is \"{}\".\n\n", cfg.registro.stati.iter().map(|x| format!("\"{x}\"")).collect::<Vec<_>>().join(", "), cfg.registro.stato_chiuso, cfg.registro.stato_archiviato);
    s += "## 5. Controls\n\n";
    s += "| Requirement | How it is met | Record |\n|---|---|---|\n";
    s += "| Review of requirements (8.2.3) | Before starting, the person in charge checks requirements, time and resources and records it | column \"Riesame il\" |\n";
    s += "| Responsibility (5.3) | Every job has a person in charge | column \"Responsabile\" |\n";
    s += "| Control of changes (8.5.6) | A change to the requirements opens a new revision with the reason; the previous approval lapses | column \"Revisione\", chronological log |\n";
    s += &format!("| Verification and release (8.6) | {} | columns \"Approvata da\", \"Approvata il\" |\n", if cfg.qualita.richiedi_approvazione { "The job is closed only after approval by the person with authority" } else { "The approval is recorded before delivery" });
    s += "| Nonconforming outputs (8.7) and corrective action (10.2) | Every defect or complaint is recorded with cause, corrective action and effectiveness check; with open nonconformities the job cannot be closed | file `non_conformita.csv` |\n";
    s += &format!("| Retention and disposal (7.5.3) | After archiving, documents are kept {}; the program never deletes anything | column \"Conserva fino al\" |\n", if cfg.qualita.conservazione_anni > 0 { format!("for {} years", cfg.qualita.conservazione_anni) } else { "for the period set by the organization".into() });
    s += &format!("| Traceability of operations (7.5.3) | {} | file `registro_eventi.csv` |\n\n", if cfg.qualita.registra_eventi { "Every creation, change, approval, archiving and undo is recorded with date, time and user; the file is append-only" } else { "Chronological log disabled" });
    s += "## 6. Protection and backup\n\nFolders and records must be kept in an area with a backup (network drive or cloud service). If they are synchronized with a cloud service, moves and deletions propagate: the program moves and never deletes, but the service may have its own recycle bin.\n\n";
    s += "## 7. Records\n\n";
    s += &format!("- `{}/registro_commesse.csv` — job register\n- `{}/non_conformita.csv` — nonconformities and corrective actions\n- `{}/registro_eventi.csv` — chronological log of operations\n- move journal (program data folder) — allows undo\n\n", cfg.cartelle.gestione, cfg.cartelle.gestione, cfg.cartelle.gestione);
    s += "## 8. Changes to the procedure\n\nEvery change to this procedure (naming, types, statuses) is recorded with a new revision of the document. When creating the document, the program never overwrites earlier revisions.\n";
    s
}

#[derive(Serialize, Debug)]
pub struct EsitoProcedura {
    pub percorso: String,
    pub esiste_gia: bool,
}

/// Scrive la procedura in `<gestione>/PR-01_Gestione_commesse_revNN.md`. Non sovrascrive: se c'è già una revisione, crea la successiva.
pub fn crea(ctx: &Contesto, simula: bool) -> Res<EsitoProcedura> {
    let cfg = ctx.cfg();
    let _g = ctx.blocca();
    let dir = cfg.cartella_gestione();
    let mut n = 0u32;
    let mut p: PathBuf;
    loop {
        p = dir.join(format!("PR-01_{}_rev{n:02}.md", if cfg.app.lingua == "en" { "Job_management" } else { "Gestione_commesse" }));
        if !p.exists() {
            break;
        }
        n += 1;
    }
    if !simula {
        let mut t = testo(&cfg);
        t = t.replace("Revisione: 00", &format!("Revisione: {n:02}")).replace("Revision: 00", &format!("Revision: {n:02}"));
        std::fs::create_dir_all(&dir)?;
        std::fs::write(&p, t)?;
        eventi::registra(&cfg, "Procedura emessa", "", &format!("revisione {n:02}"));
    }
    Ok(EsitoProcedura { percorso: p.to_string_lossy().into(), esiste_gia: n > 0 })
}
