//! Anagrafica clienti. Il «nome breve» compare nei nomi delle cartelle.
use crate::config::{Config, Contesto};
use crate::errore::{err, Errore, Res};
use crate::registro;
use crate::util::{pulisci_nome, scrivi_atomico};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Cliente {
    pub breve: String,
    pub nome: String,
    pub tipo: String,
    pub referente: String,
    pub note: String,
}

#[derive(Serialize, Debug)]
pub struct ClienteElenco {
    #[serde(flatten)]
    pub cliente: Cliente,
    pub commesse: usize,
    pub aperte: usize,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct RichiestaCliente {
    /// Nome breve precedente, quando si modifica una scheda esistente
    pub vecchio: Option<String>,
    pub nome: String,
    pub breve: String,
    pub tipo: String,
    pub referente: String,
    pub note: String,
}

pub fn carica(ctx: &Contesto) -> Vec<Cliente> {
    std::fs::read_to_string(ctx.file_clienti()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn salva_lista(ctx: &Contesto, lista: &[Cliente]) -> Res<()> {
    scrivi_atomico(&ctx.file_clienti(), serde_json::to_string_pretty(lista)?.as_bytes())
}

pub fn trova(ctx: &Contesto, breve: &str) -> Option<Cliente> {
    carica(ctx).into_iter().find(|c| c.breve == breve)
}

fn numero_commesse(cfg: &Config, breve: &str) -> Res<usize> {
    Ok(registro::leggi(cfg)?.iter().filter(|r| r.get("Breve").map(|b| b == breve).unwrap_or(false)).count())
}

pub fn elenco(ctx: &Contesto) -> Res<Vec<ClienteElenco>> {
    let cfg = ctx.cfg();
    let reg = registro::leggi(&cfg)?;
    Ok(carica(ctx)
        .into_iter()
        .map(|c| {
            let mie: Vec<_> = reg.iter().filter(|r| r.get("Breve").map(|b| *b == c.breve).unwrap_or(false)).collect();
            let aperte = mie
                .iter()
                .filter(|r| {
                    let s = r.get("Stato").map(|s| s.as_str()).unwrap_or("");
                    s != cfg.registro.stato_chiuso && s != cfg.registro.stato_archiviato
                })
                .count();
            ClienteElenco { commesse: mie.len(), aperte, cliente: c }
        })
        .collect())
}

/// Crea o modifica una scheda.
pub fn salva(ctx: &Contesto, d: RichiestaCliente) -> Res<Cliente> {
    let cfg = ctx.cfg();
    let _g = ctx.blocca();
    let nome = d.nome.trim().to_string();
    let scelto = if d.breve.trim().is_empty() { nome.clone() } else { d.breve.trim().to_string() };
    let breve = pulisci_nome(&scelto, &cfg.nomenclatura.stile_nomi);
    if nome.is_empty() || breve.is_empty() {
        return err("Servono il nome esteso e il nome breve");
    }
    let tipo = if d.tipo.is_empty() { cfg.tipi[0].id.clone() } else { d.tipo.clone() };
    cfg.tipo(&tipo)?;
    let mut lista = carica(ctx);
    let mut idx = None;
    if let Some(v) = d.vecchio.as_deref().filter(|v| !v.is_empty()) {
        idx = lista.iter().position(|c| c.breve == v);
        if idx.is_none() {
            return err("Scheda non trovata");
        }
        if breve != v {
            let n = numero_commesse(&cfg, v)?;
            if n > 0 {
                return err(format!("Il nome breve è già nelle cartelle di {n} {}: non si può cambiare", cfg.vocabolario.commesse));
            }
        }
    }
    let vecchio = d.vecchio.clone().unwrap_or_default();
    if lista.iter().any(|c| c.breve.eq_ignore_ascii_case(&breve) && c.breve != vecchio) {
        return err(format!("Nome breve già in anagrafica: {breve}"));
    }
    let voce = Cliente { breve, nome, tipo, referente: d.referente.trim().into(), note: d.note.trim().into() };
    match idx {
        Some(i) => lista[i] = voce.clone(),
        None => lista.push(voce.clone()),
    }
    lista.sort_by_key(|c| c.breve.to_lowercase());
    salva_lista(ctx, &lista)?;
    Ok(voce)
}

/// Toglie la scheda dall'anagrafica. Non tocca mai cartelle o file.
pub fn elimina(ctx: &Contesto, breve: &str) -> Res<()> {
    let cfg = ctx.cfg();
    let _g = ctx.blocca();
    let n = numero_commesse(&cfg, breve)?;
    if n > 0 {
        return Err(Errore(format!("Con {n} {} nel registro non si elimina", cfg.vocabolario.commesse)));
    }
    let lista: Vec<Cliente> = carica(ctx).into_iter().filter(|c| c.breve != breve).collect();
    salva_lista(ctx, &lista)
}
