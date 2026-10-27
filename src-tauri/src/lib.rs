//! Shell Tauri: finestra, comandi nativi (cartelle, file, apertura) e un unico comando `chiama` verso il nucleo.
use commesse_core::{api, Contesto, Errore};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tauri::{Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

struct Stato(Contesto);

fn imposta_titolo(w: &WebviewWindow, nome: &str) {
    let _ = w.set_title(nome);
}

/// Unico punto di ingresso per la logica: vedi `commesse_core::api::chiama`.
#[tauri::command]
fn chiama(window: WebviewWindow, stato: State<'_, Stato>, cmd: String, args: Value) -> Result<Value, Errore> {
    let r = api::chiama(&stato.0, &cmd, &args)?;
    if cmd == "salva_config" {
        if let Some(n) = r.pointer("/app/nome").and_then(|n| n.as_str()) {
            imposta_titolo(&window, n);
        }
    }
    Ok(r)
}

/// Finestra di scelta cartella. Restituisce `None` se l'utente annulla.
#[tauri::command]
async fn scegli_cartella(app: tauri::AppHandle, titolo: String, iniziale: Option<String>) -> Result<Option<String>, Errore> {
    let mut d = app.dialog().file().set_title(titolo);
    if let Some(i) = iniziale.filter(|i| Path::new(i).is_dir()) {
        d = d.set_directory(i);
    }
    Ok(d.blocking_pick_folder().and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().to_string()))
}

/// Salva un testo (il profilo esportato) dove sceglie l'utente.
#[tauri::command]
async fn salva_file_testo(app: tauri::AppHandle, nome: String, testo: String) -> Result<Option<String>, Errore> {
    let Some(p) = app.dialog().file().set_file_name(nome).add_filter("Profilo", &["json"]).blocking_save_file().and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    std::fs::write(&p, testo)?;
    Ok(Some(p.to_string_lossy().to_string()))
}

/// Legge un file di testo scelto dall'utente (profilo da importare). Massimo 1 MB.
#[tauri::command]
async fn apri_file_testo(app: tauri::AppHandle) -> Result<Option<String>, Errore> {
    let Some(p) = app.dialog().file().add_filter("Profilo", &["json"]).blocking_pick_file().and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    if std::fs::metadata(&p)?.len() > 1_000_000 {
        return Err(Errore("Il file è troppo grande per essere un profilo".into()));
    }
    Ok(Some(std::fs::read_to_string(&p)?))
}

/// Apre una cartella nel gestore file, solo se sta dentro le cartelle dell'app.
#[tauri::command]
fn apri_percorso(app: tauri::AppHandle, stato: State<'_, Stato>, percorso: String) -> Result<(), Errore> {
    let cfg = stato.0.cfg();
    let permessi: [PathBuf; 3] = [stato.0.home.clone(), cfg.radice(), cfg.origine()];
    let p = PathBuf::from(&percorso);
    if !permessi.iter().any(|b| commesse_core::util::e_dentro(&p, b)) {
        return Err(Errore("Percorso non consentito".into()));
    }
    std::fs::create_dir_all(&p).ok();
    app.opener().open_path(p.to_string_lossy(), None::<&str>).map_err(|e| Errore(e.to_string()))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let home = std::env::var_os("COMMESSE_HOME").map(PathBuf::from);
            let ctx = Contesto::nuovo(home).map_err(|e| e.0)?;
            if let Some(w) = app.get_webview_window("main") {
                imposta_titolo(&w, &ctx.cfg().app.nome);
            }
            app.manage(Stato(ctx));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![chiama, scegli_cartella, salva_file_testo, apri_file_testo, apri_percorso])
        .run(tauri::generate_context!())
        .expect("errore nell'avvio di Folio");
}
