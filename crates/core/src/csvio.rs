//! Lettura e scrittura dei file CSV (separatore «;», UTF-8 con BOM: si aprono in Excel).
use crate::errore::Res;
use crate::util::scrivi_atomico;
use std::collections::BTreeMap;
use std::path::Path;

pub type Riga = BTreeMap<String, String>;

pub fn leggi(p: &Path) -> Res<Vec<Riga>> {
    if !p.exists() {
        return Ok(vec![]);
    }
    let mut dati = std::fs::read(p)?;
    if dati.starts_with(&[0xEF, 0xBB, 0xBF]) {
        dati.drain(..3);
    }
    let mut r = csv::ReaderBuilder::new().delimiter(b';').flexible(true).from_reader(dati.as_slice());
    let intest: Vec<String> = r.headers()?.iter().map(|s| s.to_string()).collect();
    let mut out = vec![];
    for rec in r.records() {
        let rec = rec?;
        out.push(intest.iter().enumerate().map(|(i, h)| (h.clone(), rec.get(i).unwrap_or("").to_string())).collect());
    }
    Ok(out)
}

/// Riscrive il file. Le colonne presenti nelle righe ma non in `colonne` si conservano in coda, per non perdere dati.
pub fn scrivi(p: &Path, colonne: &[String], righe: &[Riga]) -> Res<()> {
    let mut cols = colonne.to_vec();
    for r in righe {
        for k in r.keys() {
            if !cols.contains(k) {
                cols.push(k.clone());
            }
        }
    }
    let mut buf: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
    {
        let mut w = csv::WriterBuilder::new().delimiter(b';').from_writer(&mut buf);
        w.write_record(&cols)?;
        for r in righe {
            w.write_record(cols.iter().map(|c| r.get(c).map(|s| s.as_str()).unwrap_or("")))?;
        }
        w.flush()?;
    }
    scrivi_atomico(p, &buf)
}

/// Aggiunge una riga in coda senza riscrivere il file (registro cronologico: non si modifica mai ciò che c'è già).
pub fn aggiungi_in_coda(p: &Path, colonne: &[&str], valori: &[&str]) -> Res<()> {
    use std::io::Write;
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let nuovo = !p.exists() || std::fs::metadata(p)?.len() == 0;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(p)?;
    let mut buf: Vec<u8> = if nuovo { vec![0xEF, 0xBB, 0xBF] } else { vec![] };
    {
        let mut w = csv::WriterBuilder::new().delimiter(b';').from_writer(&mut buf);
        if nuovo {
            w.write_record(colonne)?;
        }
        w.write_record(valori)?;
        w.flush()?;
    }
    f.write_all(&buf)?;
    Ok(())
}
