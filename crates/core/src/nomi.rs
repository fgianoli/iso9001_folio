//! Nomenclature a modello: `{anno}-{num}`, `{codice}_{ente}_{oggetto}`, ...
//!
//! Un campo si scrive `{nome}` oppure `{nome:N}` (per i numeri: N cifre con zeri a sinistra).
use crate::errore::{err, Res};
use regex::Regex;

pub const CAMPI_CODICE: [&str; 3] = ["anno", "num", "sigla"];
pub const CAMPI_CARTELLA: [&str; 8] = ["codice", "anno", "num", "sigla", "tipo", "ente", "oggetto", "data"];

#[derive(Debug, Clone, PartialEq)]
pub enum Pezzo {
    Lett(String),
    Campo(String, Option<usize>),
}

/// Valori da inserire nei modelli.
#[derive(Debug, Clone, Default)]
pub struct Campi {
    pub anno: String,
    pub num: u64,
    pub sigla: String,
    pub tipo: String,
    pub ente: String,
    pub oggetto: String,
    pub data: String,
    pub codice: String,
}

pub fn analizza(p: &str) -> Res<Vec<Pezzo>> {
    let mut out = Vec::new();
    let mut lett = String::new();
    let mut it = p.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '{' => {
                let mut dentro = String::new();
                let mut chiuso = false;
                for d in it.by_ref() {
                    if d == '}' {
                        chiuso = true;
                        break;
                    }
                    dentro.push(d);
                }
                if !chiuso {
                    return err(format!("Modello «{p}»: manca una }}"));
                }
                if !lett.is_empty() {
                    out.push(Pezzo::Lett(std::mem::take(&mut lett)));
                }
                let (nome, larg) = match dentro.split_once(':') {
                    Some((n, w)) => {
                        let w: usize = w.trim().parse().map_err(|_| crate::errore::Errore(format!("Modello «{p}»: «{dentro}» non è valido")))?;
                        if !(1..=9).contains(&w) {
                            return err(format!("Modello «{p}»: le cifre vanno da 1 a 9"));
                        }
                        (n.trim().to_string(), Some(w))
                    }
                    None => (dentro.trim().to_string(), None),
                };
                if nome.is_empty() {
                    return err(format!("Modello «{p}»: campo vuoto"));
                }
                out.push(Pezzo::Campo(nome, larg));
            }
            '}' => return err(format!("Modello «{p}»: }} senza {{")),
            _ => lett.push(c),
        }
    }
    if !lett.is_empty() {
        out.push(Pezzo::Lett(lett));
    }
    Ok(out)
}

/// Controlla campi ammessi, campi obbligatori e caratteri vietati nelle parti fisse.
pub fn valida(p: &str, ammessi: &[&str], obbligatori: &[&str]) -> Res<()> {
    let pezzi = analizza(p)?;
    for z in &pezzi {
        match z {
            Pezzo::Campo(n, _) if !ammessi.contains(&n.as_str()) => {
                return err(format!("Modello «{p}»: il campo {{{n}}} qui non è disponibile (usa: {})", ammessi.iter().map(|a| format!("{{{a}}}")).collect::<Vec<_>>().join(", ")));
            }
            Pezzo::Lett(l) if l.chars().any(|c| "<>:\"/\\|?*".contains(c) || c.is_control()) => {
                return err(format!("Modello «{p}»: i caratteri < > : \" / \\ | ? * non sono ammessi nei nomi di cartelle"));
            }
            _ => {}
        }
    }
    for o in obbligatori {
        if !pezzi.iter().any(|z| matches!(z, Pezzo::Campo(n, _) if n == o)) {
            return err(format!("Modello «{p}»: deve contenere {{{o}}}"));
        }
    }
    Ok(())
}

pub fn ha_campo(p: &str, nome: &str) -> bool {
    analizza(p).map(|v| v.iter().any(|z| matches!(z, Pezzo::Campo(n, _) if n == nome))).unwrap_or(false)
}

pub fn rendi(p: &str, c: &Campi, cifre: usize) -> Res<String> {
    let mut out = String::new();
    for z in analizza(p)? {
        match z {
            Pezzo::Lett(l) => out.push_str(&l),
            Pezzo::Campo(n, w) => match n.as_str() {
                "anno" => out.push_str(&c.anno),
                "num" => out.push_str(&format!("{:0>w$}", c.num, w = w.unwrap_or(cifre))),
                "sigla" => out.push_str(&c.sigla),
                "tipo" => out.push_str(&c.tipo),
                "ente" => out.push_str(&c.ente),
                "oggetto" => out.push_str(&c.oggetto),
                "data" => out.push_str(&c.data),
                "codice" => out.push_str(&c.codice),
                altro => return err(format!("Campo sconosciuto: {{{altro}}}")),
            },
        }
    }
    Ok(out)
}

/// Espressione regolare che riconosce un nome costruito con il modello.
/// `codice_pezzi`: se presente, `{codice}` viene espanso con quel modello.
pub fn regex_da_modello(p: &str, codice: Option<&str>) -> Res<Regex> {
    let mut pezzi = analizza(p)?;
    if let Some(cp) = codice {
        let cp = analizza(cp)?;
        let mut esp = Vec::new();
        for z in pezzi {
            match z {
                Pezzo::Campo(n, _) if n == "codice" => esp.extend(cp.iter().cloned()),
                altro => esp.push(altro),
            }
        }
        pezzi = esp;
    }
    let mut visti: Vec<String> = Vec::new();
    let mut re = String::from("^");
    for z in &pezzi {
        match z {
            Pezzo::Lett(l) => re.push_str(&regex::escape(l)),
            Pezzo::Campo(n, _) => {
                let nuovo = !visti.contains(n);
                let corpo = match n.as_str() {
                    "anno" => r"\d{4}",
                    "num" => r"\d+",
                    "sigla" => r"[A-Za-z0-9]+",
                    _ => r".+?",
                };
                if nuovo {
                    re.push_str(&format!("(?P<{n}>{corpo})"));
                    visti.push(n.clone());
                } else {
                    re.push_str(&format!("(?:{corpo})"));
                }
            }
        }
    }
    re.push('$');
    Ok(Regex::new(&re)?)
}
