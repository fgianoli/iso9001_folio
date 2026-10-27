//! Punto d'ingresso unico per le interfacce: `chiama(ctx, "comando", argomenti)`.
//!
//! L'app Tauri e il server di prova usano questa stessa funzione, quindi ciò che si prova nei test è ciò che gira nell'app.
use crate::config::Config;
use crate::errore::{Errore, Res};
use crate::operazioni::{self, ModificaMappa, NuovaCommessa};
use crate::{clienti, eventi, giornale, nonconf, procedura, profili, registro, util, Contesto, VERSIONE};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn arg<T: DeserializeOwned>(a: &Value, k: &str) -> Res<T> {
    serde_json::from_value(a.get(k).cloned().unwrap_or(Value::Null)).map_err(|e| Errore(format!("Argomento «{k}» non valido: {e}")))
}

fn vv<T: serde::Serialize>(x: T) -> Res<Value> {
    Ok(serde_json::to_value(x)?)
}

pub fn chiama(ctx: &Contesto, cmd: &str, a: &Value) -> Res<Value> {
    let simula = || a.get("simula").and_then(|s| s.as_bool()).unwrap_or(true);
    match cmd {
        "stato" => {
            let cfg = ctx.cfg();
            Ok(json!({
                "versione": VERSIONE,
                "config": cfg,
                "prima_volta": ctx.prima_volta(),
                "anno": util::anno_corrente(),
                "radice_esiste": cfg.radice().exists(),
                "home": ctx.home,
                "profili": profili::elenco(&cfg.app.lingua),
                "profili_it": profili::elenco("it"),
                "profili_en": profili::elenco("en"),
            }))
        }
        "salva_config" => vv(ctx.salva_config(arg::<Config>(a, "config")?)?),
        "profilo_applica" => vv(profili::applica(&arg::<String>(a, "id")?, &arg::<Config>(a, "config")?)?),
        "profilo_esporta" => vv(profili::esporta(&arg::<Config>(a, "config")?)?),
        "profilo_importa" => vv(profili::importa(&arg::<String>(a, "testo")?, &arg::<Config>(a, "config")?)?),

        "anteprima" => vv(operazioni::anteprima_cfg(ctx, &arg::<Config>(a, "config")?, &arg::<NuovaCommessa>(a, "richiesta")?)?),
        "prossimo" => vv(operazioni::prossimo(ctx, &arg::<String>(a, "tipo")?, &arg::<String>(a, "anno")?)?),
        "nuova_commessa" => vv(operazioni::nuova_commessa(ctx, &arg::<NuovaCommessa>(a, "richiesta")?)?),
        "crea_struttura" => vv(operazioni::crea_struttura(ctx, simula())?),

        "clienti" => vv(clienti::elenco(ctx)?),
        "cliente_salva" => vv(clienti::salva(ctx, arg(a, "richiesta")?)?),
        "cliente_elimina" => {
            clienti::elimina(ctx, &arg::<String>(a, "breve")?)?;
            Ok(json!({"ok": true}))
        }

        "registro" => {
            let cfg = ctx.cfg();
            Ok(json!({"colonne": registro::colonne(&cfg), "righe": registro::leggi(&cfg)?}))
        }
        "registro_aggiorna" => {
            let _g = ctx.blocca();
            vv(registro::aggiorna(&ctx.cfg(), &arg::<registro::Aggiornamento>(a, "richiesta")?)?)
        }
        "eventi" => vv(eventi::ultimi(&ctx.cfg(), 300)?),
        "nc_elenco" => vv(nonconf::leggi(&ctx.cfg())?),
        "nc_nuova" => vv(nonconf::nuova(ctx, &arg::<nonconf::NuovaNc>(a, "richiesta")?)?),
        "nc_aggiorna" => vv(nonconf::aggiorna(ctx, &arg::<nonconf::AggiornaNc>(a, "richiesta")?)?),
        "procedura_testo" => vv(procedura::testo(&ctx.cfg())),
        "procedura_crea" => vv(procedura::crea(ctx, simula())?),
        "archivia" => vv(operazioni::archivia(ctx, &arg::<String>(a, "codice")?, simula())?),

        "mappa" => vv(operazioni::stato_mappa(ctx)),
        "mappa_modifica" => vv(json!({"righe": operazioni::modifica_mappa(ctx, &arg::<ModificaMappa>(a, "richiesta")?)?})),
        "riordina" => vv(operazioni::riordina(ctx, &arg::<Vec<usize>>(a, "indici")?, simula())?),

        "scansione_avvia" => {
            ctx.scansione.avvia(ctx.cfg());
            Ok(json!({"ok": true}))
        }
        "scansione_stato" => vv(ctx.scansione.stato()),
        "quarantena" => vv(crate::pulizia::quarantena(ctx, simula())?),

        "giornali" => vv(giornale::elenco(ctx)),
        "annulla" => vv(giornale::annulla_ultimo(ctx, &ctx.cfg(), simula())?),
        altro => Err(Errore(format!("Comando sconosciuto: {altro}"))),
    }
}
