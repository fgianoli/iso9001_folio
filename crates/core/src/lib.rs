//! Folio: nucleo senza interfaccia. Lo usano l'app Tauri e i test.
pub mod api;
pub mod clienti;
pub mod config;
pub mod csvio;
pub mod errore;
pub mod eventi;
pub mod giornale;
pub mod nomi;
pub mod nonconf;
pub mod operazioni;
pub mod procedura;
pub mod profili;
pub mod pulizia;
pub mod registro;
pub mod util;

pub use config::{Config, Contesto};
pub use errore::{Errore, Res};

pub const VERSIONE: &str = env!("CARGO_PKG_VERSION");
