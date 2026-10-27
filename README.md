# Folio

Codici univoci, cartelle standard, anagrafica clienti, riordino guidato, archivio e pulizia sicura per studi e liberi professionisti. App desktop per **Windows, macOS e Linux**, ispirata ai requisiti della **ISO 9001** sulle informazioni documentate. Tutto è personalizzabile: nome, colori, nomenclature, tipi di progetto.

**Non elimina mai nulla:** sposta file e cartelle, non sovrascrive, ogni operazione parte in simulazione e si può annullare.

## Installare

Scarica l'installer dalla pagina **Releases** del progetto:

| Sistema | File | Note |
|---|---|---|
| Windows 10/11 | `Folio_x.y.z_x64-setup.exe` oppure `.msi` | Se compare «Windows ha protetto il PC» (SmartScreen): *Ulteriori informazioni* → *Esegui comunque*. Succede perché l'installer non è firmato. |
| macOS | `Folio_x.y.z_aarch64.dmg` (Apple Silicon) o `_x64.dmg` (Intel) | Se il sistema blocca l'apertura: tasto destro sull'app → *Apri* (la prima volta). |
| Linux | `.AppImage` (rendilo eseguibile e avvialo), `.deb` o `.rpm` | Serve `webkit2gtk` (già presente sulla maggior parte delle distribuzioni con desktop). |

*English version: [README.en.md](README.en.md). The app itself can be switched to English in Customize → Appearance → Language.*

Non serve installare Python o altro. Gli installer non sono firmati: la firma richiede un certificato a pagamento (vedi *Firma* più sotto).

## Primo avvio

1. Scegli un modello di partenza (Studio professionale, Ricerca e consulenza, Minimo).
2. In **Impostazioni** controlla la cartella principale (dove nasce l'organizzazione) e premi **Crea struttura**.
3. In **Clienti** inserisci le schede; in **Nuova commessa** scegli cliente e oggetto, premi *Simula* e poi *Crea*.
4. Per ordinare lavori già esistenti: **Riordino** (mappa «ora → dopo», simulazione, esecuzione). Per la pulizia: **Pulizia**.

## Personalizzare

Tutto in **Personalizza**; le modifiche si vedono subito e valgono dopo *Salva*.

- **Aspetto:** nome del programma, frase sotto il titolo, colore principale (tavolozza o scelta libera), tema chiaro/scuro/sistema, carattere, angoli, spaziature.
- **Parole:** come chiami «commessa» e «cliente» (progetto, pratica, lavoro, partner…), con il genere per concordare «Nuovo/Nuova».
- **Nomenclature:** modello del codice (`{anno}-{num}`, `{sigla}-{anno}-{num:4}`…) e del nome cartella (`{codice}_{ente}_{oggetto}`), cifre, stile dei nomi (`AcmeIdrica`, `acme_idrica`, `acme-idrica`), numerazione per anno e per tipo, prefisso delle versioni. L'anteprima mostra subito il risultato e segnala gli errori.
- **Tipi di progetto:** per ognuno cartella, sigla, sottocartelle, etichetta del committente, colore e modello del file di presentazione.
- **Registro:** stati, stato che consente l'archiviazione, colonne in più (CIG, importo, finanziatore…).
- **Qualità:** anni di conservazione, approvazione obbligatoria prima della chiusura, registro cronologico.
- **Pulizia:** soglia dei file grandi, cartelle escluse, categorie attive, parole da segnalare.
- **Profili:** applica un modello, esporta la tua configurazione in un file (senza i percorsi del tuo PC) o importane una ricevuta da altri.

## ISO 9001: che cosa fa e che cosa no

La norma non impone una struttura di cartelle: chiede che le informazioni documentate siano identificate, approvate, protette, conservate e rintracciabili. Il programma aiuta a soddisfare questi punti con controlli concreti:

| Clausola | Controllo nel programma |
|---|---|
| 7.5.1 Informazioni documentate | Procedura generata dalla tua configurazione (scheda *Qualità*), con revisioni che non si sovrascrivono |
| 7.5.2 Identificazione e descrizione | Codice univoco, modello dei nomi, versioni `v01, v02` senza «FINALE» |
| 7.5.3 Controllo | Registro cronologico delle operazioni (solo in coda), conservazione indicata nel registro, nessuna eliminazione |
| 8.2.3 Riesame dei requisiti | Casella e data di riesame, richiamate nella presentazione di ogni commessa |
| 5.3 Responsabilità | Responsabile di ogni commessa |
| 8.5.2 Rintracciabilità | Codice nel registro, nella cartella e nel registro cronologico |
| 8.5.3 Proprietà del cliente | Cartella degli input da non modificare |
| 8.5.6 Controllo delle modifiche | Revisioni con motivo; l'approvazione precedente decade |
| 8.6 Rilascio | Approvazione prima della chiusura (disattivabile) |
| 8.7 / 10.2 Non conformità | Registro con causa, azione correttiva e verifica dell'efficacia; con non conformità aperte non si chiude la commessa |

**Limiti:** il programma non sostituisce politica per la qualità, obiettivi, analisi dei rischi, audit interni e riesame di direzione, e non rende conforme né certifica un'organizzazione. Verifica con il tuo consulente o ente di certificazione che il tuo uso soddisfi i requisiti applicabili.

## Sicurezza dei dati

- Sposta, non elimina. Se due dischi diversi richiedono una copia, l'originale si toglie solo dopo una copia completa riuscita.
- Non sovrascrive: se la destinazione esiste, l'operazione si ferma.
- Simulazione prima di ogni spostamento; giornale per annullare l'ultima operazione.
- Con cartelle sincronizzate (Google Drive, OneDrive, Dropbox) gli spostamenti si propagano nel cloud; tieni la cartella principale dentro l'area sincronizzata e non metterci un repository Git.
- Nessun accesso alla rete: l'app non si collega a Internet.

## Dove sono i dati

| Sistema | Cartella |
|---|---|
| Windows | `%APPDATA%\commesse` |
| macOS | `~/Library/Application Support/commesse` |
| Linux | `~/.config/commesse` (o `$XDG_CONFIG_HOME`) |

Contiene `impostazioni.json`, `clienti.json`, `mappa.json` e il giornale. Si può cambiare con la variabile `COMMESSE_HOME`. Il registro e le registrazioni stanno nella cartella principale, in `00_GESTIONE` (nome modificabile).

Chi arriva dalla versione Python: la configurazione, le schede clienti e la mappa vengono rilette da sole; i registri esistenti restano validi (le colonne nuove si aggiungono vuote).

## Per sviluppatori

```
crates/core    nucleo in Rust (nessuna interfaccia): nomenclature, registro, clienti, giornale, operazioni, pulizia, qualità
src-tauri      shell Tauri: finestra e comandi nativi
ui             interfaccia (HTML, CSS, JavaScript; nessun passaggio di compilazione)
```

```
cargo test -p commesse-core                    # test del nucleo
cargo install tauri-cli --version "^2" --locked
cargo tauri dev                                # avvia l'app in sviluppo
cargo tauri build                              # crea gli installer per il sistema in uso
cargo run -p commesse-core --example devserver -- ui /tmp/dati   # solo per provare l'interfaccia in un browser (serve uno shim di window.__TAURI__)
```

Su Linux servono `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libssl-dev`. Rust 1.85 o successivo.

La logica sta tutta nel nucleo e l'interfaccia la raggiunge da un solo comando (`chiama`): i test del nucleo coprono ciò che gira nell'app.

### Pubblicare gli installer

`git tag v0.2.0 && git push --tags` avvia il workflow *Release*, che crea una bozza di release con gli installer per Windows, macOS (Apple Silicon e Intel) e Linux.

### Firma

Senza firma Windows (SmartScreen) e macOS (Gatekeeper) mostrano un avviso al primo avvio. Per eliminarlo servono un certificato di firma del codice per Windows e un Apple Developer ID con notarizzazione per macOS; i segreti si configurano come indicato nei commenti di `release.yml`.

## Licenza

MIT, vedi `LICENSE`.
