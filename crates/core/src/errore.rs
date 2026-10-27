//! Errore unico dell'applicazione: un messaggio già pronto da mostrare all'utente.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Errore(pub String);

pub type Res<T> = Result<T, Errore>;

impl fmt::Display for Errore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Errore {}

impl serde::Serialize for Errore {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl From<std::io::Error> for Errore {
    fn from(e: std::io::Error) -> Self {
        Errore(e.to_string())
    }
}
impl From<serde_json::Error> for Errore {
    fn from(e: serde_json::Error) -> Self {
        Errore(format!("JSON non valido: {e}"))
    }
}
impl From<csv::Error> for Errore {
    fn from(e: csv::Error) -> Self {
        Errore(format!("CSV non valido: {e}"))
    }
}
impl From<regex::Error> for Errore {
    fn from(e: regex::Error) -> Self {
        Errore(format!("Modello non valido: {e}"))
    }
}

/// Scorciatoia: `err("messaggio")` o `err(format!(...))`.
pub fn err<T>(m: impl Into<String>) -> Res<T> {
    Err(Errore(m.into()))
}
