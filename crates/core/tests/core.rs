use commesse_core::clienti::{self, RichiestaCliente};
use commesse_core::config::Config;
use commesse_core::nonconf::{self, AggiornaNc, NuovaNc};
use commesse_core::operazioni::{self, ModificaMappa, NuovaCommessa};
use commesse_core::registro::Aggiornamento;
use commesse_core::util::{e_dentro, percorso_relativo, pulisci_nome};
use commesse_core::{eventi, giornale, nomi, procedura, profili, pulizia, registro, Contesto};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Amb {
    _t: TempDir,
    ctx: Contesto,
    radice: PathBuf,
    origine: PathBuf,
}

fn amb_con(mut cfg: Config) -> Amb {
    let t = tempfile::tempdir().unwrap();
    let radice = t.path().join("radice");
    let origine = t.path().join("origine");
    fs::create_dir_all(&origine).unwrap();
    let ctx = Contesto::nuovo(Some(t.path().join("dati"))).unwrap();
    cfg.cartelle.radice = radice.to_string_lossy().into();
    cfg.cartelle.origine = origine.to_string_lossy().into();
    ctx.salva_config(cfg).unwrap();
    Amb { _t: t, ctx, radice, origine }
}
fn amb() -> Amb {
    amb_con(Config::default())
}
fn cliente(a: &Amb, nome: &str) -> String {
    clienti::salva(&a.ctx, RichiestaCliente { nome: nome.into(), ..Default::default() }).unwrap().breve
}
fn agg(cfg: &Config, codice: &str, f: impl FnOnce(&mut Aggiornamento)) -> Result<registro::Riga, commesse_core::Errore> {
    let mut a = Aggiornamento { codice: codice.into(), ..Default::default() };
    f(&mut a);
    registro::aggiorna(cfg, &a)
}
/// Approva e chiude, come richiede la configurazione predefinita.
fn chiudi(cfg: &Config, codice: &str) {
    agg(cfg, codice, |a| a.approva = Some("Resp".into())).unwrap();
    let chiuso = cfg.registro.stato_chiuso.clone();
    agg(cfg, codice, |a| a.stato = Some(chiuso)).unwrap();
}
fn nuova(tipo: &str, anno: &str, num: u64, cli: &str, ogg: &str, simula: bool) -> NuovaCommessa {
    NuovaCommessa { tipo: tipo.into(), anno: anno.into(), num: Some(num), cliente: Some(cli.into()), ogg: ogg.into(), simula, ..Default::default() }
}

#[test]
fn nomi_puliti_in_quattro_stili() {
    assert_eq!(pulisci_nome("Sincronizzazione GPKG àè", "pascal"), "SincronizzazioneGPKGAe");
    assert_eq!(pulisci_nome("Relazione idraulica", "snake"), "relazione_idraulica");
    assert_eq!(pulisci_nome("Relazione idraulica", "kebab"), "relazione-idraulica");
    assert_eq!(pulisci_nome("Rilievo à", "maiuscolo"), "RILIEVOA");
    assert_eq!(pulisci_nome("***", "pascal"), "");
}

#[test]
fn modelli_validazione_resa_e_regex() {
    assert!(nomi::valida("{anno}-{num}", &nomi::CAMPI_CODICE, &["num"]).is_ok());
    assert!(nomi::valida("{anno}", &nomi::CAMPI_CODICE, &["num"]).is_err());
    assert!(nomi::valida("{ente}-{num}", &nomi::CAMPI_CODICE, &["num"]).is_err());
    assert!(nomi::valida("{num", &nomi::CAMPI_CODICE, &["num"]).is_err());
    assert!(nomi::valida("{num}/{anno}", &nomi::CAMPI_CODICE, &["num"]).is_err());
    let c = nomi::Campi { anno: "2026".into(), num: 7, sigla: "RIC".into(), ..Default::default() };
    assert_eq!(nomi::rendi("{sigla}-{anno}-{num:4}", &c, 3).unwrap(), "RIC-2026-0007");
    assert_eq!(nomi::rendi("{anno}-{num}", &c, 3).unwrap(), "2026-007");
    let re = nomi::regex_da_modello("{sigla}-{anno}-{num:4}", None).unwrap();
    let m = re.captures("RIC-2026-0007").unwrap();
    assert_eq!(&m["num"], "0007");
}

#[test]
fn configurazione_predefinita_e_profili_sono_validi() {
    assert!(Config::default().valida().is_ok());
    let base = Config::default();
    for p in profili::elenco("it") {
        let c = profili::applica(&p.id, &base).unwrap();
        assert!(c.valida().is_ok(), "profilo {}", p.id);
        assert_eq!(c.cartelle.radice, base.cartelle.radice, "le cartelle del computer restano");
    }
}

#[test]
fn configurazione_rifiuta_scelte_sbagliate() {
    let mut c = Config::default();
    c.tema.accento = "verde".into();
    assert!(c.valida().unwrap_err().0.contains("#rrggbb"));
    let mut c = Config::default();
    c.nomenclatura.codice = "{anno}".into();
    assert!(c.valida().is_err());
    let mut c = Config::default();
    c.nomenclatura.cartella = "{ente}_{oggetto}".into();
    assert!(c.valida().is_err());
    let mut c = Config::default();
    c.tipi[1].dir = c.tipi[0].dir.clone();
    assert!(c.valida().unwrap_err().0.contains("stessa cartella"));
    let mut c = Config::default();
    c.tipi[0].sub.push("../fuori".into());
    assert!(c.valida().is_err());
    let mut c = Config::default();
    c.registro.stato_chiuso = "Finita".into();
    assert!(c.valida().is_err());
    let mut c = Config::default();
    c.registro.colonne_extra = vec!["Stato".into()];
    assert!(c.valida().is_err());
    let mut c = Config::default();
    c.nomenclatura.codice = "{sigla}-{num}".into();
    c.tipi[0].sigla = String::new();
    assert!(c.valida().unwrap_err().0.contains("sigla"));
}

#[test]
fn nuova_commessa_crea_cartelle_readme_e_registro() {
    let a = amb();
    let c = cliente(&a, "Acme Idrica S.p.A.");
    assert_eq!(c, "AcmeIdricaSPA");
    operazioni::crea_struttura(&a.ctx, false).unwrap();
    let sim = operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 1, &c, "Relazione idraulica", true)).unwrap();
    assert_eq!(sim.codice, "2026-001");
    assert!(!a.radice.join("10_CLIENTI").join(&sim.nome).exists(), "la simulazione non scrive");
    let r = operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 1, &c, "Relazione idraulica", false)).unwrap();
    assert_eq!(r.nome, "2026-001_AcmeIdricaSPA_RelazioneIdraulica");
    let cart = a.radice.join("10_CLIENTI").join(&r.nome);
    assert!(cart.join("03_Lavoro").is_dir() && cart.join("README.md").is_file());
    let readme = fs::read_to_string(cart.join("README.md")).unwrap();
    assert!(readme.contains("# 2026-001 - Acme Idrica S.p.A. - Relazione idraulica") && readme.contains("v01, v02"));
    let reg = registro::leggi(&a.ctx.cfg()).unwrap();
    assert_eq!(reg.len(), 1);
    assert_eq!(reg[0]["Breve"], c);
    assert_eq!(reg[0]["Percorso"], format!("10_CLIENTI/{}", r.nome));
    // il file si apre in Excel: BOM e separatore ';'
    let raw = fs::read(a.ctx.cfg().file_registro()).unwrap();
    assert_eq!(&raw[..3], &[0xEF, 0xBB, 0xBF]);
    assert!(String::from_utf8_lossy(&raw).contains("Codice;Cartella;Tipo"));
    // duplicati
    assert!(operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 1, &c, "Altro", false)).is_err());
    assert_eq!(operazioni::prossimo(&a.ctx, "cliente", "2026").unwrap(), 2);
    assert_eq!(operazioni::prossimo(&a.ctx, "cliente", "2027").unwrap(), 1);
}

#[test]
fn nomenclatura_personalizzata_con_sigla_e_numerazione_per_tipo() {
    let mut cfg = Config::default();
    cfg.nomenclatura.codice = "{sigla}-{anno}-{num:4}".into();
    cfg.nomenclatura.cartella = "{codice}__{ente}__{oggetto}".into();
    cfg.nomenclatura.numerazione_per_tipo = true;
    cfg.nomenclatura.stile_nomi = "kebab".into();
    cfg.registro.colonne_extra = vec!["CIG".into(), "Importo".into()];
    let a = amb_con(cfg);
    let c = cliente(&a, "Comune di Esempio");
    assert_eq!(c, "comune-di-esempio");
    let mut n = nuova("cliente", "2026", 1, &c, "Rilievo Drone", false);
    n.extra.insert("CIG".into(), "Z123".into());
    let r = operazioni::nuova_commessa(&a.ctx, &n).unwrap();
    assert_eq!(r.codice, "CLI-2026-0001");
    assert_eq!(r.nome, "CLI-2026-0001__comune-di-esempio__rilievo-drone");
    // un altro tipo riparte da 1, lo stesso tipo prosegue
    assert_eq!(operazioni::prossimo(&a.ctx, "ricerca", "2026").unwrap(), 1);
    assert_eq!(operazioni::prossimo(&a.ctx, "cliente", "2026").unwrap(), 2);
    let reg = registro::leggi(&a.ctx.cfg()).unwrap();
    assert_eq!(reg[0]["CIG"], "Z123");
    let ap = operazioni::anteprima(&a.ctx, &NuovaCommessa { tipo: "ricerca".into(), anno: "2026".into(), num: Some(1), ..Default::default() }).unwrap();
    assert_eq!(ap.codice, "RIC-2026-0001");
    // colonna extra sconosciuta rifiutata
    let mut n2 = nuova("cliente", "2026", 2, &c, "X", false);
    n2.extra.insert("Boh".into(), "1".into());
    assert!(operazioni::nuova_commessa(&a.ctx, &n2).is_err());
}

#[test]
fn numerazione_continua_senza_azzerare_ogni_anno() {
    let mut cfg = Config::default();
    cfg.nomenclatura.azzera_ogni_anno = false;
    let a = amb_con(cfg);
    let c = cliente(&a, "Beta");
    operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 5, &c, "Uno", false)).unwrap();
    assert_eq!(operazioni::prossimo(&a.ctx, "cliente", "2027").unwrap(), 6);
}

#[test]
fn registro_aggiorna_stati_e_colonne_extra() {
    let mut cfg = Config::default();
    cfg.registro.stati = vec!["Aperto".into(), "Chiuso".into()];
    cfg.registro.stato_chiuso = "Chiuso".into();
    cfg.registro.colonne_extra = vec!["Referente".into()];
    let a = amb_con(cfg);
    let c = cliente(&a, "Beta");
    let r = operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 1, &c, "Uno", false)).unwrap();
    let cfg = a.ctx.cfg();
    let riga = agg(&cfg, &r.codice, |x| { x.fattura = Some(" F-12 ".into()); x.extra.insert("Referente".into(), "Mario".into()); }).unwrap();
    assert_eq!((riga["Fattura"].as_str(), riga["Referente"].as_str()), ("F-12", "Mario"));
    chiudi(&cfg, &r.codice);
    let riga = &registro::leggi(&cfg).unwrap()[0];
    assert_eq!(riga["Stato"], "Chiuso");
    assert!(!riga["Chiusa il"].is_empty());
    assert!(agg(&cfg, &r.codice, |x| x.stato = Some("Sospesa".into())).is_err());
    let riga = agg(&cfg, &r.codice, |x| x.stato = Some("Aperto".into())).unwrap();
    assert_eq!(riga["Chiusa il"], "");
    assert!(agg(&cfg, &r.codice, |x| { x.extra.insert("Boh".into(), "1".into()); }).is_err());
}

#[test]
fn clienti_modifica_e_eliminazione() {
    let a = amb();
    let b = cliente(&a, "Acme");
    assert!(clienti::salva(&a.ctx, RichiestaCliente { nome: "acme".into(), breve: "ACME".into(), ..Default::default() }).is_err(), "nome breve duplicato");
    let v = clienti::salva(&a.ctx, RichiestaCliente { vecchio: Some(b.clone()), nome: "Acme Srl".into(), breve: "Acme".into(), referente: "Rossi".into(), ..Default::default() }).unwrap();
    assert_eq!(v.nome, "Acme Srl");
    assert!(clienti::salva(&a.ctx, RichiestaCliente { nome: "X".into(), tipo: "inesistente".into(), ..Default::default() }).is_err());
    operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 1, "Acme", "Uno", false)).unwrap();
    let e = clienti::elenco(&a.ctx).unwrap();
    assert_eq!((e[0].commesse, e[0].aperte), (1, 1));
    assert!(clienti::elimina(&a.ctx, "Acme").is_err());
    assert!(clienti::salva(&a.ctx, RichiestaCliente { vecchio: Some("Acme".into()), nome: "Acme".into(), breve: "Altro".into(), ..Default::default() }).is_err());
    let vuoto = cliente(&a, "Vuoto");
    clienti::elimina(&a.ctx, &vuoto).unwrap();
    assert_eq!(clienti::carica(&a.ctx).len(), 1);
}

#[test]
fn riordino_simulazione_esecuzione_e_annullamento() {
    let a = amb();
    operazioni::crea_struttura(&a.ctx, false).unwrap();
    fs::create_dir_all(a.origine.join("CLIENTI/2025/Acme")).unwrap();
    fs::write(a.origine.join("CLIENTI/2025/Acme/rilievo.txt"), "dati").unwrap();
    for (v, n) in [("CLIENTI/2025/Acme", "10_CLIENTI/2025-001_Acme_Rilievi"), ("../fuori", "10_CLIENTI/x"), ("CLIENTI/Manca", "10_CLIENTI/y")] {
        let r = operazioni::modifica_mappa(&a.ctx, &ModificaMappa { vecchio: v.into(), nuovo: n.into(), ..Default::default() });
        assert_eq!(r.is_ok(), !v.contains(".."), "{v}");
    }
    operazioni::modifica_mappa(&a.ctx, &ModificaMappa { vecchio: "CLIENTI/Manca".into(), nuovo: "10_CLIENTI/y".into(), ..Default::default() }).unwrap_err();
    let st = operazioni::stato_mappa(&a.ctx);
    assert_eq!((st[0].stato.as_str(), st[1].stato.as_str()), ("pronto", "manca"));
    let sim = operazioni::riordina(&a.ctx, &[0], true).unwrap();
    assert_eq!(sim.esiti[0].esito, "simulato");
    assert!(a.origine.join("CLIENTI/2025/Acme").exists());
    let r = operazioni::riordina(&a.ctx, &[0], false).unwrap();
    assert_eq!(r.esiti[0].esito, "ok");
    let nuovo = a.radice.join("10_CLIENTI/2025-001_Acme_Rilievi/rilievo.txt");
    assert!(nuovo.exists() && !a.origine.join("CLIENTI/2025/Acme").exists());
    assert_eq!(operazioni::stato_mappa(&a.ctx)[0].stato, "fatto");
    // l'annullamento rimette tutto com'era
    let cfg = a.ctx.cfg();
    let sim = giornale::annulla_ultimo(&a.ctx, &cfg, true).unwrap();
    assert_eq!(sim.esiti[0].esito, "simulato");
    let ann = giornale::annulla_ultimo(&a.ctx, &cfg, false).unwrap();
    assert!(ann.esiti.iter().all(|e| e.esito == "ok"));
    assert!(a.origine.join("CLIENTI/2025/Acme/rilievo.txt").exists() && !nuovo.exists());
    assert!(giornale::annulla_ultimo(&a.ctx, &cfg, false).is_err(), "nessun altro giornale");
}

#[test]
fn spostamento_non_sovrascrive_mai() {
    let a = amb();
    operazioni::crea_struttura(&a.ctx, false).unwrap();
    fs::create_dir_all(a.origine.join("A")).unwrap();
    fs::create_dir_all(a.radice.join("10_CLIENTI/Dest")).unwrap();
    fs::write(a.radice.join("10_CLIENTI/Dest/keep.txt"), "x").unwrap();
    operazioni::modifica_mappa(&a.ctx, &ModificaMappa { vecchio: "A".into(), nuovo: "10_CLIENTI/Dest".into(), ..Default::default() }).unwrap();
    assert_eq!(operazioni::stato_mappa(&a.ctx)[0].stato, "conflitto");
    let r = operazioni::riordina(&a.ctx, &[0], false).unwrap();
    assert_eq!(r.esiti[0].esito, "esiste");
    assert!(a.origine.join("A").exists() && r.giornale.is_none());
}

#[test]
fn archiviazione_aggiorna_registro_e_si_annulla() {
    let a = amb();
    let c = cliente(&a, "Acme");
    operazioni::crea_struttura(&a.ctx, false).unwrap();
    let r = operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 1, &c, "Uno", false)).unwrap();
    assert!(operazioni::archivia(&a.ctx, &r.codice, true).is_err(), "solo le commesse chiuse");
    let cfg = a.ctx.cfg();
    chiudi(&cfg, &r.codice);
    let sim = operazioni::archivia(&a.ctx, &r.codice, true).unwrap();
    assert_eq!(sim.esiti[0].esito, "simulato");
    let ok = operazioni::archivia(&a.ctx, &r.codice, false).unwrap();
    assert_eq!(ok.esiti[0].esito, "ok");
    assert!(a.radice.join("90_ARCHIVIO/2026").join(&r.nome).is_dir());
    assert!(!a.radice.join("10_CLIENTI").join(&r.nome).exists());
    let riga = &registro::leggi(&cfg).unwrap()[0];
    assert_eq!((riga["Stato"].as_str(), riga["Percorso"].as_str()), ("Archiviata", format!("90_ARCHIVIO/2026/{}", r.nome).as_str()));
    assert!(agg(&cfg, &r.codice, |x| x.stato = Some("Aperta".into())).is_err());
    assert!(!riga_conserva(&cfg).is_empty(), "data di conservazione impostata");
    giornale::annulla_ultimo(&a.ctx, &cfg, false).unwrap();
    let riga = &registro::leggi(&cfg).unwrap()[0];
    assert_eq!((riga["Stato"].as_str(), riga["Percorso"].as_str()), ("Chiusa", format!("10_CLIENTI/{}", r.nome).as_str()));
    assert!(a.radice.join("10_CLIENTI").join(&r.nome).is_dir());
    assert_eq!(riga_conserva(&cfg), "", "l'annullamento ripristina anche la conservazione");
}

fn riga_conserva(cfg: &Config) -> String {
    registro::leggi(cfg).unwrap()[0]["Conserva fino al"].clone()
}

#[test]
fn pulizia_scansione_e_quarantena_con_regole_personali() {
    let mut cfg = Config::default();
    cfg.cartelle.radici_scansione = vec!["CLIENTI".into()];
    cfg.pulizia.parole_da_verificare = vec!["bozza".into()];
    cfg.pulizia.disattivate = vec!["lock_office".into()];
    cfg.pulizia.soglia_grande_mb = 1;
    let a = amb_con(cfg);
    let base = a.origine.join("CLIENTI/Acme");
    fs::create_dir_all(base.join("__pycache__")).unwrap();
    fs::create_dir_all(base.join(".git")).unwrap();
    fs::create_dir_all(base.join("dati")).unwrap();
    fs::write(base.join("__pycache__/m.pyc"), "x").unwrap();
    fs::write(base.join(".git/Thumbs.db"), "x").unwrap();
    fs::write(base.join("Thumbs.db"), "x").unwrap();
    fs::write(base.join("~$doc.docx"), "x").unwrap();
    fs::write(base.join("rilievo.tif.aux.xml"), "x").unwrap();
    fs::write(base.join("relazione (1).docx"), "x").unwrap();
    fs::write(base.join("dati.zip"), "x").unwrap();
    fs::write(base.join("bozza_finale.docx"), "x").unwrap();
    fs::write(base.join("enorme.bin"), vec![0u8; 2 * 1024 * 1024]).unwrap();
    fs::write(base.join("buono.txt"), "ok").unwrap();
    let cfg = a.ctx.cfg();
    a.ctx.scansione.esegui(&cfg);
    let s = a.ctx.scansione.stato();
    assert!(s.fine && s.errore.is_none());
    let id: Vec<_> = s.categorie.iter().map(|c| c.id.as_str()).collect();
    assert!(id.contains(&"cache_python") && id.contains(&"sistema") && id.contains(&"aux_xml"));
    assert!(!id.contains(&"lock_office"), "categoria disattivata");
    assert!(id.contains(&"copia") && id.contains(&"zip_estratto") && id.contains(&"parole") && id.contains(&"grande"));
    assert_eq!(s.categorie.iter().find(|c| c.id == "sistema").unwrap().n, 1, ".git è escluso");
    assert!(Path::new(s.csv.as_ref().unwrap()).exists());
    let q = pulizia::quarantena(&a.ctx, true).unwrap();
    assert!(q.esiti.iter().all(|e| e.esito == "simulato") && base.join("Thumbs.db").exists());
    let q = pulizia::quarantena(&a.ctx, false).unwrap();
    assert!(q.esiti.iter().all(|e| e.esito == "ok"));
    assert!(!base.join("Thumbs.db").exists() && !base.join("__pycache__").exists());
    assert!(base.join("buono.txt").exists() && base.join("dati.zip").exists() && base.join("~$doc.docx").exists());
    assert!(Path::new(&q.quarantena).join("CLIENTI/Acme/Thumbs.db").exists());
    giornale::annulla_ultimo(&a.ctx, &cfg, false).unwrap();
    assert!(base.join("Thumbs.db").exists() && base.join("__pycache__/m.pyc").exists());
}

#[test]
fn senza_scansione_la_quarantena_non_parte() {
    let a = amb();
    assert!(pulizia::quarantena(&a.ctx, true).is_err());
}

#[test]
fn profilo_si_esporta_e_si_importa_senza_percorsi_locali() {
    let a = amb();
    let mut cfg = a.ctx.cfg();
    cfg.app.nome = "Il mio studio".into();
    cfg.tema.accento = "#aa3300".into();
    cfg.vocabolario.commessa = "pratica".into();
    cfg.vocabolario.commesse = "pratiche".into();
    cfg.tipi.truncate(2);
    a.ctx.salva_config(cfg.clone()).unwrap();
    let json = profili::esporta(&cfg).unwrap();
    assert!(!json.contains(&a.radice.to_string_lossy().to_string()));
    let altro = amb();
    let importato = profili::importa(&json, &altro.ctx.cfg()).unwrap();
    assert_eq!(importato.app.nome, "Il mio studio");
    assert_eq!(importato.tipi.len(), 2);
    assert_eq!(importato.cartelle.radice, altro.ctx.cfg().cartelle.radice);
    assert!(profili::importa("{non json", &altro.ctx.cfg()).is_err());
    assert!(profili::importa(r##"{"tema":{"accento":"rosso"}}"##, &altro.ctx.cfg()).is_err());
}

#[test]
fn la_configurazione_persiste_e_si_ricarica() {
    let t = tempfile::tempdir().unwrap();
    let ctx = Contesto::nuovo(Some(t.path().to_path_buf())).unwrap();
    assert!(ctx.prima_volta());
    let mut c = ctx.cfg();
    c.app.nome = "Studio Gianoli".into();
    c.tema.compatto = true;
    ctx.salva_config(c).unwrap();
    assert!(!ctx.prima_volta());
    let ctx2 = Contesto::nuovo(Some(t.path().to_path_buf())).unwrap();
    assert!(!ctx2.prima_volta());
    assert_eq!(ctx2.cfg().app.nome, "Studio Gianoli");
    assert!(ctx2.cfg().tema.compatto);
    // una configurazione non valida non si salva e non cambia lo stato
    let mut male = ctx2.cfg();
    male.nomenclatura.cifre_numero = 0;
    assert!(ctx2.salva_config(male).is_err());
    assert_eq!(ctx2.cfg().nomenclatura.cifre_numero, 3);
}

#[test]
fn migrazione_dalla_versione_python() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("config.json"), r#"{"radice":"C:/Lavoro","desktop":"C:/Vecchio","radici_scansione":["CLIENTI"],"porta":8765,
        "tipi":{"cliente":{"dir":"10_CLIENTI","nome":"Cliente","sub":["01_A","02_B"]}}}"#).unwrap();
    let ctx = Contesto::nuovo(Some(t.path().to_path_buf())).unwrap();
    let c = ctx.cfg();
    assert_eq!((c.cartelle.radice.as_str(), c.cartelle.origine.as_str()), ("C:/Lavoro", "C:/Vecchio"));
    assert_eq!(c.cartelle.radici_scansione, vec!["CLIENTI".to_string()]);
    assert_eq!(c.tipi.len(), 1);
    assert_eq!(c.tipi[0].sub, vec!["01_A", "02_B"]);
}

#[test]
fn registro_di_versioni_precedenti_senza_colonna_breve() {
    let a = amb();
    let cfg = a.ctx.cfg();
    fs::create_dir_all(cfg.cartella_gestione()).unwrap();
    let mut csv = vec![0xEF, 0xBB, 0xBF];
    csv.extend_from_slice("Codice;Cartella;Tipo;Ente;Oggetto;Stato;Aperta il;Chiusa il;Fattura;Percorso\n2025-014;2025-014_Italgas_Sync;Cliente;Italgas;Sync;Aperta;2025-03-01;;;10_CLIENTI/2025-014_Italgas_Sync\n".as_bytes());
    fs::write(cfg.file_registro(), csv).unwrap();
    let righe = registro::leggi(&cfg).unwrap();
    assert_eq!(righe[0]["Breve"], "Italgas");
    assert_eq!(registro::prossimo_numero(&cfg, "2025", "CLI").unwrap(), 15);
}

#[test]
fn percorsi_sicuri() {
    assert!(percorso_relativo("CLIENTI/2025/Acme").is_ok());
    assert!(percorso_relativo("CLIENTI\\2025").is_ok());
    for male in ["", "  ", "../x", "a/../../b", "/etc", "C:\\x", "d:/y"] {
        assert!(percorso_relativo(male).is_err(), "{male}");
    }
    assert!(e_dentro(Path::new("/a/b/c"), Path::new("/a/b")));
    assert!(e_dentro(Path::new("/a/b"), Path::new("/a/b")));
    assert!(!e_dentro(Path::new("/a/b/../x"), Path::new("/a/b")));
    assert!(!e_dentro(Path::new("/a/bc"), Path::new("/a/b")));
}

fn con_progetto() -> (Amb, String) {
    let a = amb();
    operazioni::crea_struttura(&a.ctx, false).unwrap();
    let c = cliente(&a, "Acme");
    let r = operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 1, &c, "Uno", false)).unwrap();
    (a, r.codice)
}

#[test]
fn iso_riesame_responsabile_e_revisione() {
    let a = amb();
    let c = cliente(&a, "Acme");
    let mut n = nuova("cliente", "2026", 1, &c, "Uno", false);
    n.responsabile = "Federico".into();
    n.riesame = true;
    let r = operazioni::nuova_commessa(&a.ctx, &n).unwrap();
    let cfg = a.ctx.cfg();
    let riga = &registro::leggi(&cfg).unwrap()[0];
    assert_eq!((riga["Responsabile"].as_str(), riga["Revisione"].as_str()), ("Federico", "00"));
    assert!(!riga["Riesame il"].is_empty());
    let readme = fs::read_to_string(a.radice.join("10_CLIENTI").join(&r.nome).join("README.md")).unwrap();
    assert!(readme.contains("Responsabile: Federico") && readme.contains("8.5.3"));
    assert!(a.radice.join("10_CLIENTI").join(&r.nome).join("01_Offerta_Riesame").is_dir());
    // revisione: serve il motivo, incrementa e annulla l'approvazione
    agg(&cfg, &r.codice, |x| x.approva = Some("Direttore".into())).unwrap();
    assert!(agg(&cfg, &r.codice, |x| x.nuova_revisione = Some("  ".into())).is_err());
    let riga = agg(&cfg, &r.codice, |x| x.nuova_revisione = Some("Cambia il perimetro".into())).unwrap();
    assert_eq!(riga["Revisione"], "01");
    assert_eq!((riga["Approvata da"].as_str(), riga["Approvata il"].as_str()), ("", ""));
    assert_eq!(agg(&cfg, &r.codice, |x| x.nuova_revisione = Some("Ancora".into())).unwrap()["Revisione"], "02");
}

#[test]
fn iso_la_chiusura_richiede_approvazione_e_nessuna_nc_aperta() {
    let (a, cod) = con_progetto();
    let cfg = a.ctx.cfg();
    let e = agg(&cfg, &cod, |x| x.stato = Some("Chiusa".into())).unwrap_err();
    assert!(e.0.contains("approvazione"), "{e}");
    agg(&cfg, &cod, |x| x.approva = Some("Resp".into())).unwrap();
    let nc = nonconf::nuova(&a.ctx, &NuovaNc { commessa: cod.clone(), descrizione: "Quota errata".into(), responsabile: "Resp".into() }).unwrap();
    assert!(nc["Codice NC"].starts_with("NC-") && nc["Codice NC"].ends_with("-001"));
    let e = agg(&cfg, &cod, |x| x.stato = Some("Chiusa".into())).unwrap_err();
    assert!(e.0.contains("non conformità aperte"), "{e}");
    // non si chiude la NC senza causa e azione
    assert!(nonconf::aggiorna(&a.ctx, &AggiornaNc { codice: nc["Codice NC"].clone(), stato: Some("Chiusa".into()), ..Default::default() }).is_err());
    nonconf::aggiorna(&a.ctx, &AggiornaNc { codice: nc["Codice NC"].clone(), causa: Some("Dato errato in input".into()), azione: Some("Ricontrollo e rettifica".into()), ..Default::default() }).unwrap();
    let chiusa = nonconf::aggiorna(&a.ctx, &AggiornaNc { codice: nc["Codice NC"].clone(), stato: Some("Chiusa".into()), efficacia: Some(true), ..Default::default() }).unwrap();
    assert!(!chiusa["Chiusa il"].is_empty() && !chiusa["Efficacia verificata il"].is_empty());
    agg(&cfg, &cod, |x| x.stato = Some("Chiusa".into())).unwrap();
    assert_eq!(nonconf::leggi(&cfg).unwrap().len(), 1);
    assert!(nonconf::nuova(&a.ctx, &NuovaNc { commessa: "9999-999".into(), descrizione: "x".into(), ..Default::default() }).is_err());
    assert!(nonconf::nuova(&a.ctx, &NuovaNc::default()).is_err());
}

#[test]
fn iso_approvazione_non_obbligatoria_se_disattivata() {
    let mut cfg = Config::default();
    cfg.qualita.richiedi_approvazione = false;
    let a = amb_con(cfg);
    let c = cliente(&a, "Acme");
    let r = operazioni::nuova_commessa(&a.ctx, &nuova("cliente", "2026", 1, &c, "Uno", false)).unwrap();
    agg(&a.ctx.cfg(), &r.codice, |x| x.stato = Some("Chiusa".into())).unwrap();
}

#[test]
fn iso_registro_eventi_solo_in_coda() {
    let (a, cod) = con_progetto();
    let cfg = a.ctx.cfg();
    agg(&cfg, &cod, |x| x.responsabile = Some("Federico".into())).unwrap();
    chiudi(&cfg, &cod);
    let prima = fs::read_to_string(eventi::file(&cfg)).unwrap();
    assert!(prima.starts_with('\u{feff}'));
    operazioni::archivia(&a.ctx, &cod, false).unwrap();
    giornale::annulla_ultimo(&a.ctx, &cfg, false).unwrap();
    let dopo = fs::read_to_string(eventi::file(&cfg)).unwrap();
    assert!(dopo.starts_with(&prima), "le voci già registrate non cambiano");
    for voce in ["Commessa creata", "Registro: Approvazione e rilascio", "Registro: Stato", "Archiviata", "Operazione annullata"] {
        assert!(dopo.contains(voce), "manca «{voce}»");
    }
    let ult = eventi::ultimi(&cfg, 2).unwrap();
    assert_eq!(ult[0]["Operazione"], "Operazione annullata");
    // disattivabile
    let mut c2 = cfg.clone();
    c2.qualita.registra_eventi = false;
    a.ctx.salva_config(c2.clone()).unwrap();
    let n = fs::read_to_string(eventi::file(&cfg)).unwrap().lines().count();
    agg(&c2, &cod, |x| x.responsabile = Some("Altro".into())).unwrap();
    assert_eq!(fs::read_to_string(eventi::file(&cfg)).unwrap().lines().count(), n);
}

#[test]
fn iso_procedura_documentata_senza_sovrascrivere() {
    let (a, _) = con_progetto();
    let t = procedura::testo(&a.ctx.cfg());
    for pezzo in ["8.5.2", "8.5.3", "8.5.6", "8.6", "8.7", "10.2", "7.5.3", "{anno}-{num}", "10_CLIENTI", "non sostituisce"] {
        assert!(t.to_lowercase().contains(&pezzo.to_lowercase()), "manca «{pezzo}»");
    }
    let sim = procedura::crea(&a.ctx, true).unwrap();
    assert!(!Path::new(&sim.percorso).exists());
    let p0 = procedura::crea(&a.ctx, false).unwrap();
    let p1 = procedura::crea(&a.ctx, false).unwrap();
    assert!(p0.percorso.ends_with("rev00.md") && p1.percorso.ends_with("rev01.md") && p1.esiste_gia);
    assert!(Path::new(&p0.percorso).exists() && fs::read_to_string(&p1.percorso).unwrap().contains("Revisione: 01"));
}

#[test]
fn iso_conservazione_calcola_la_data() {
    use commesse_core::util::aggiungi_anni;
    assert_eq!(aggiungi_anni("2026-03-10", 10), "2036-03-10");
    assert_eq!(aggiungi_anni("2024-02-29", 1), "2025-02-28");
    assert_eq!(aggiungi_anni("boh", 1), "");
}

#[test]
fn inglese_profili_procedura_readme_ed_eventi() {
    // profilo inglese: lingua, vocabolario e cartelle in inglese
    let mut base = Config::default();
    base.app.lingua = "en".into();
    for id in ["studio", "ricerca", "minimo"] {
        let c = profili::applica(id, &base).unwrap();
        assert_eq!(c.app.lingua, "en", "{id}");
        assert_eq!(c.cartelle.archivio, "90_ARCHIVE");
        c.valida().unwrap();
    }
    assert_eq!(profili::elenco("en")[0].nome, "Professional firm");
    assert_eq!(profili::elenco("it")[0].nome, "Studio professionale");
    // lingua non ammessa
    let mut x = Config::default();
    x.app.lingua = "xx".into();
    assert!(x.valida().is_err());

    let cfg = profili::applica("studio", &base).unwrap();
    let a = amb_con(cfg);
    a.ctx.salva_config(a.ctx.cfg()).unwrap();
    operazioni::crea_struttura(&a.ctx, false).unwrap();
    let cl = cliente(&a, "Example Council");
    let r = operazioni::nuova_commessa(&a.ctx, &NuovaCommessa { tipo: "cliente".into(), anno: "2026".into(), num: Some(1), cliente: Some(cl), ogg: "Flood survey".into(), simula: false, ..Default::default() }).unwrap();
    let readme = fs::read_to_string(a.radice.join("10_CLIENTS").join(&r.nome).join("README.md")).unwrap();
    assert!(readme.contains("Person in charge:") && readme.contains("Rules"), "{readme}");
    assert!(!readme.contains("Responsabile:"));
    // procedura in inglese, nome file inglese
    let p = procedura::crea(&a.ctx, false).unwrap();
    assert!(p.percorso.contains("PR-01_Job_management_rev00"), "{}", p.percorso);
    let t = fs::read_to_string(&p.percorso).unwrap();
    assert!(t.contains("## 5. Controls") && t.contains("Revision: 00"), "{t}");
    // registro eventi con etichette inglesi
    let ev = eventi::ultimi(&a.ctx.cfg(), 20).unwrap();
    let ops: Vec<String> = ev.iter().filter_map(|e| e.get("Operazione").cloned()).collect();
    assert!(ops.iter().any(|o| o == "Job created"), "{ops:?}");
    assert!(ops.iter().any(|o| o == "Procedure issued"), "{ops:?}");
}
