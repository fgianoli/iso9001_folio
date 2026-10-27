'use strict';
// Traduzione italiano → inglese dell'interfaccia. L'italiano è la lingua del codice; l'inglese si applica al volo
// ai testi della pagina (statici e generati) e ai messaggi di errore del nucleo.
(function () {
  const EX = {
    // intestazioni e schede
    'Riordino': 'Reorganize', 'Pulizia': 'Cleanup', 'Registro e annulla': 'Register & undo', 'Qualità': 'Quality', 'Personalizza': 'Customize', 'Impostazioni': 'Settings',
    'Da dove vuoi partire?': 'Where do you want to start?',
    'Scegli un modello di organizzazione. Poi potrai cambiare tutto in Personalizza: nome, colori, tipi di progetto, nomenclature.': 'Pick an organization model. You can change everything later in Customize: name, colors, project types, naming rules.',
    'Parti da qui': 'Start here',
    // form nuova
    'Tipo': 'Type', 'Anno': 'Year', 'Numero': 'Number', 'Oggetto': 'Subject', 'Relazione idraulica': 'Hydraulic report', 'Responsabile': 'Person in charge', 'Chi risponde del lavoro': 'Who is responsible for the work',
    'Riesame dei requisiti eseguito (ISO 9001 8.2.3)': 'Requirements review done (ISO 9001 8.2.3)',
    'Simula': 'Simulate', "Scegli il cliente, scrivi l'oggetto e premi Simula.": 'Pick the client, write the subject and press Simulate.',
    // clienti
    'Nome esteso': 'Full name', 'Nome breve, usato nelle cartelle': 'Short name, used in folder names', 'Tipo predefinito': 'Default type', 'Referente': 'Contact person', 'Note': 'Notes', 'Salva': 'Save',
    'Nuova scheda': 'New record', 'Elimina scheda': 'Delete record', 'Modifica scheda': 'Edit record', 'Breve': 'Short', 'Nome': 'Name',
    'Nessuna scheda ancora. Compila il modulo a sinistra e premi Salva.': 'No records yet. Fill in the form on the left and press Save.',
    'Modifica': 'Edit', 'Togli': 'Remove', 'Scheda eliminata': 'Record deleted', 'Salvato': 'Saved', 'Scegli…': 'Choose…',
    // riordino
    'Per riordinare cartelle esistenti, scrivi per ognuna dove si trova (relativo alla cartella da riordinare) e dove deve andare (relativo alla cartella principale). Poi seleziona, simula ed esegui.': 'To reorganize existing folders, write for each one where it is now (relative to the folder to reorganize) and where it should go (relative to the main folder). Then select, simulate and run.',
    'Aggiungi una riga alla mappa': 'Add a row to the map', 'Ora': 'Now', 'Dopo': 'After', 'Nota': 'Note', 'Aggiungi': 'Add', 'Stato': 'Status',
    'Seleziona i pronti': 'Select the ready ones', 'Simula selezionati': 'Simulate selected', 'Esegui selezionati': 'Run selected', 'Nessuna operazione ancora.': 'No operation yet.',
    'La mappa è vuota. Aggiungi sopra la prima riga.': 'The map is empty. Add the first row above.',
    'pronto': 'ready', 'fatto': 'done', 'manca': 'missing', 'conflitto': 'conflict', 'errore': 'error', 'Aggiunta': 'Added', 'Seleziona almeno una riga': 'Select at least one row',
    "Simulazione fatta: controlla l'elenco, poi esegui": 'Simulation done: check the list, then run', 'Simulazione fatta': 'Simulation done', 'Simulazione ok': 'Simulation OK',
    // pulizia
    'Scansiona': 'Scan', 'Categoria': 'Category', 'Voci': 'Items', 'Peso': 'Size', 'Rischio': 'Risk', 'Sicuro': 'Safe', 'Da verificare': 'To check',
    'Nessuna scansione ancora. Le cartelle da scansionare si scelgono in Impostazioni.': 'No scan yet. The folders to scan are chosen in Settings.',
    'Simula quarantena dei file sicuri': 'Simulate quarantine of safe files', 'Sposta in quarantena': 'Move to quarantine', 'Nessun file inutile trovato.': 'No useless files found.',
    'Cache Python': 'Python cache', 'Cache IDE (.idea)': 'IDE cache (.idea)', 'Checkpoint Jupyter': 'Jupyter checkpoints', 'File di sistema': 'System files', 'Lock di Office (~$)': 'Office locks (~$)', 'Temporanei (.tmp)': 'Temporary (.tmp)', 'Sidecar GDAL (.aux.xml)': 'GDAL sidecars (.aux.xml)',
    'Download di Drive/OneDrive': 'Drive/OneDrive downloads', 'Copie e backup': 'Copies and backups', 'Zip già estratto accanto': 'Zip already extracted next to it', 'Zip': 'Zip', 'File oltre la soglia': 'Files over the threshold', 'Nomi da verificare (regole personali)': 'Names to check (personal rules)',
    // registro
    "Annulla l'ultima operazione di spostamento": 'Undo the last move operation', 'Simula annullamento': 'Simulate undo', 'Annulla davvero': 'Undo for real', 'Annullato': 'Undone',
    'Nessun giornale: non è stato spostato nulla.': 'No journal: nothing has been moved.',
    'Conferma archiviazione': 'Confirm archiving', 'Chiudi': 'Close', 'Conferma': 'Confirm',
    'Codice': 'Code', 'Ente': 'Body', 'Rev.': 'Rev.', 'Riesame': 'Review', 'Approvazione': 'Approval', 'Fattura': 'Invoice', 'Conserva fino al': 'Keep until', 'Chiusa il': 'Closed on', 'n. fattura': 'invoice no.',
    'da approvare': 'to approve', 'Approva…': 'Approve…', 'Revisione…': 'Revision…', 'Archivia…': 'Archive…', 'non eseguito': 'not done',
    'Il registro è vuoto. Crea la prima voce dalla scheda iniziale.': 'The register is empty. Create the first entry from the first tab.',
    'Nome di chi approva': 'Name of the approver', 'Motivo della modifica': 'Reason for the change', 'Registrato': 'Recorded',
    'Verifica e rilascio (ISO 9001 8.6): scrivi chi approva. La data è quella di oggi.': 'Verification and release (ISO 9001 8.6): write who approves. The date is today.',
    "Una modifica ai requisiti (8.5.6) crea la revisione successiva e annulla l'approvazione. Scrivi il motivo.": 'A change to the requirements (8.5.6) creates the next revision and cancels the approval. Write the reason.',
    // impostazioni
    'Cartelle': 'Folders', "Cartella principale (qui nasce l'organizzazione)": 'Main folder (the organization is created here)', 'Cartella con i lavori esistenti da riordinare': 'Folder with the existing work to reorganize', 'Sfoglia…': 'Browse…',
    'Sottocartelle da scansionare per la pulizia (una per riga, relative alla cartella da riordinare)': 'Subfolders to scan for cleanup (one per line, relative to the folder to reorganize)',
    'Le modifiche si salvano con la barra in basso.': 'Changes are saved with the bar at the bottom.',
    'Struttura base': 'Base structure', 'Crea struttura': 'Create structure', 'Struttura creata': 'Structure created', 'Apri la cartella dei dati': 'Open the data folder',
    'Salva prima le modifiche (barra in basso)': 'Save your changes first (bar at the bottom)',
    // qualità
    'Qui si tengono le registrazioni della qualità richieste dalla ISO 9001: le non conformità con la loro azione correttiva (8.7, 10.2), il registro cronologico delle operazioni (7.5.3) e la procedura documentata (7.5.1). Il programma ti aiuta a tenerle in ordine; non sostituisce un sistema di gestione completo.': 'This is where the quality records required by ISO 9001 are kept: nonconformities with their corrective action (8.7, 10.2), the chronological log of operations (7.5.3) and the documented procedure (7.5.1). The program helps you keep them tidy; it does not replace a full management system.',
    'Registra una non conformità': 'Record a nonconformity', 'Che cosa non è conforme': 'What is not conforming', 'Registra': 'Record', 'Descrizione': 'Description', 'Causa': 'Cause', 'Azione correttiva': 'Corrective action', 'Efficacia': 'Effectiveness',
    'Aperta': 'Open', 'In corso': 'In progress', 'Chiusa': 'Closed', 'Verificata': 'Verified', 'Nessuna non conformità registrata.': 'No nonconformities recorded.',
    'Registro delle operazioni': 'Operations log', 'Aggiorna': 'Refresh', 'Data e ora': 'Date and time', 'Utente': 'User', 'Operazione': 'Operation', 'Dettaglio': 'Detail',
    'Nessuna operazione registrata. Il registro nasce con la struttura base.': 'No operation recorded. The log starts with the base structure.',
    'Procedura documentata': 'Documented procedure',
    'Genera dal tuo schema attuale un documento che descrive codifica, cartelle, controlli e registrazioni. Ogni emissione crea una nuova revisione nella cartella dei registri; le precedenti non si toccano.': 'Generates from your current setup a document describing coding, folders, controls and records. Each issue creates a new revision in the records folder; earlier ones are never touched.',
    'Mostra anteprima': 'Show preview', 'Emetti procedura': 'Issue procedure',
    // personalizza
    'Modelli di partenza e condivisione': 'Starting models and sharing',
    'Applica un modello pronto, oppure esporta la tua configurazione per usarla su un altro computer o condividerla (non contiene i percorsi del tuo PC).': 'Apply a ready-made model, or export your setup to use it on another computer or share it (it does not contain your PC paths).',
    'Applica il modello': 'Apply the model', 'Esporta in un file…': 'Export to a file…', 'Importa da un file…': 'Import from a file…',
    
    'Aspetto': 'Appearance', 'Lingua': 'Language', 'Nome del programma': 'Program name', 'Tema': 'Theme', 'Come il sistema': 'Same as the system', 'Chiaro': 'Light', 'Scuro': 'Dark', 'Carattere': 'Font', 'Del sistema': 'System font', 'Con grazie (serif)': 'Serif', 'A spaziatura fissa': 'Monospace',
    'Angoli arrotondati:': 'Rounded corners:', 'Frase sotto il titolo': 'Line under the title', 'Colore principale': 'Main color', 'Titoli con le grazie': 'Serif headings', 'Spaziature compatte': 'Compact spacing',
    'Verde bosco': 'Forest green', 'Blu': 'Blue', 'Terracotta': 'Terracotta', 'Viola': 'Purple', 'Grafite': 'Graphite', 'Ocra': 'Ochre', 'Rosa antico': 'Dusty rose',
    'Parole': 'Words', 'Come chiami le cose nel tuo lavoro. Compaiono in tutte le schede.': 'What you call things in your work. They appear in every tab.',
    'Una commessa si chiama': 'A job is called', 'Più commesse si chiamano': 'Several jobs are called', 'Un cliente si chiama': 'A client is called', 'Più clienti si chiamano': 'Several clients are called', 'Genere': 'Gender', 'femminile (nuova)': 'feminine (Italian)', 'maschile (nuovo)': 'masculine (Italian)',
    'Nomenclature': 'Naming rules', "Il codice è l'identificativo unico (ISO 9001, cl. 7.5); il nome della cartella lo contiene sempre. Clicca un campo per inserirlo.": 'The code is the unique identifier (ISO 9001, cl. 7.5); the folder name always contains it. Click a field to insert it.',
    'Codice': 'Code', 'Nome della cartella': 'Folder name', 'Cifre del numero': 'Number digits', 'Stile dei nomi': 'Name style', 'Prefisso delle versioni': 'Version prefix', 'Cifre delle versioni': 'Version digits', 'Nome del file di presentazione': 'Presentation file name',
    'La numerazione riparte da 1 a ogni anno': 'Numbering restarts at 1 every year', 'Numerazione separata per tipo (serve {sigla})': 'Separate numbering per type (needs {sigla})', 'Crea il file di presentazione in ogni cartella': 'Create the presentation file in every folder', 'Archivio diviso per anno': 'Archive split by year', 'Anteprima': 'Preview',
    'Tipi di progetto': 'Project types', 'Aggiungi un tipo': 'Add a type', 'Ogni tipo ha la sua cartella, le sue sottocartelle e un modello di presentazione.': 'Each type has its own folder, subfolders and a presentation template.',
    'Identificativo': 'Identifier', 'Identificativo (fisso)': 'Identifier (fixed)', 'Cartella': 'Folder', 'Sigla (per {sigla})': 'Code letters (for {sigla})', 'Come si chiama il committente': 'What the client is called', 'Colore del tipo': 'Type color', 'Nessuno': 'None', 'Sottocartelle, una per riga': 'Subfolders, one per line',
    'Modello del file di presentazione (vuoto = modello generale). Segnaposto: {codice} {ente} {oggetto} {tipo} {data} {stato} {versioni} {cartella}': 'Presentation file template (empty = general template). Placeholders: {codice} {ente} {oggetto} {tipo} {data} {stato} {versioni} {cartella}',
    'Rimuovi': 'Remove', 'Sposta su': 'Move up', 'Sposta giù': 'Move down', 'Senza nome': 'Unnamed',
    "Stati, uno per riga, nell'ordine di avanzamento (il primo è quello iniziale)": 'Statuses, one per line, in order of progress (the first is the initial one)', "Stato che consente l'archiviazione": 'Status that allows archiving', "Stato dopo l'archiviazione": 'Status after archiving',
    'Colonne in più, una per riga (es. CIG, Importo, Referente)': 'Extra columns, one per line (e.g. Tender ID, Amount, Contact)',
    'Cartelle di servizio': 'Service folders', 'Cartella del registro': 'Register folder', "Cartella dell'archivio": 'Archive folder', 'Prefisso della quarantena': 'Quarantine prefix',
    'Le cartelle già create non vengono rinominate: se cambi questi nomi, spostale tu prima di usarli.': 'Folders already created are not renamed: if you change these names, move them yourself before using them.',
    'Qualità (ISO 9001)': 'Quality (ISO 9001)', "Anni di conservazione dopo l'archiviazione (0 = non indicato)": 'Years of retention after archiving (0 = not specified)', "Si chiude solo dopo l'approvazione (rilascio, 8.6)": 'Closes only after approval (release, 8.6)', 'Registro cronologico delle operazioni (7.5.3)': 'Chronological log of operations (7.5.3)',
    'Segnala i file oltre (MB)': 'Flag files over (MB)', 'Cartelle da non visitare, una per riga': 'Folders not to visit, one per line', 'Parole nel nome file da segnalare (solo elenco, non si spostano), una per riga': 'Words in file names to flag (list only, never moved), one per line', 'Categorie sicure da considerare nella quarantena': 'Safe categories to include in quarantine',
    'Modifiche non salvate': 'Unsaved changes', 'Annulla modifiche': 'Discard changes',
    'Modello applicato alla bozza: controlla e premi Salva': 'Model applied to the draft: check it and press Save', 'Profilo importato nella bozza: controlla e premi Salva': 'Profile imported into the draft: check it and press Save',
    'Serve almeno un tipo': 'At least one type is needed',
    'AcmeIdrica': 'AcmeWater', 'acme-idrica': 'acme-water', 'acme_idrica': 'acme_water', 'ACMEIDRICA': 'ACMEWATER',
    // tooltip/placeholder
    'Colore principale ': 'Main color',
    // mappa stati esiti
    'simulato': 'simulated', 'ok': 'ok', 'esiste': 'exists',
    // messaggi con parti fisse
    'Nuovo tipo': 'New type',
    'Qui adatti il programma al tuo modo di lavorare. Le modifiche si vedono subito, ma valgono per le operazioni solo dopo Salva.': 'Here you adapt the program to the way you work. Changes show immediately, but apply to operations only after Save.',
    'Collegamento a {commessa} (facoltativo)': 'Link to {commessa} (optional)',
    'Con Google Drive, OneDrive o Dropbox attivi, ciò che sposti sul PC si sposta anche nel cloud. I file sicuri vanno in una quarantena datata dentro {archivio}, da cui li puoi recuperare. Zip, copie e download restano solo in un elenco CSV: li decidi tu.': 'With Google Drive, OneDrive or Dropbox active, what you move on the PC also moves in the cloud. Safe files go into a dated quarantine inside {archivio}, from which you can recover them. Zips, copies and downloads only end up in a CSV list: you decide.',
    'Crea nella cartella principale una cartella per ogni tipo, più {gestione} (registro) e {archivio}. Non tocca nulla di esistente.': 'Creates in the main folder one folder per type, plus {gestione} (records) and {archivio}. Touches nothing that already exists.',
    'Il nome breve finisce nel nome delle cartelle e non si cambia quando ha già {commesse}. Eliminare toglie solo la scheda, mai i file.': 'The short name goes into folder names and cannot be changed once it already has {commesse}. Deleting removes only the record, never the files.',
    'Registro: {commesse}': 'Register: {commesse}',
    'Sposta la cartella in {archivio} e aggiorna il registro. Si può annullare dal riquadro qui sopra.': 'Moves the folder into {archivio} and updates the register. It can be undone from the box above.',
    'Crea {commessa}': 'Create {commessa}',
  };
  const PT = [
    // messaggi dell'interfaccia
    [/^Salvato: (.+)$/, 'Saved: $1'], [/^Creata (.+)$/, 'Created $1'], [/^Fatto\. Giornale: (.+)$/, 'Done. Journal: $1'],
    [/^Archiviata (.+)\. Giornale: (.+)$/, 'Archived $1. Journal: $2'], [/^Archiviare (.+)\?$/, 'Archive $1?'], [/^Approvare (.+)\?$/, 'Approve $1?'],
    [/^Nuova revisione di (.+)$/, 'New revision of $1'], [/^Ultimo giornale: (.+)$/, 'Latest journal: $1'],
    [/^Scansione in corso… (.+) file visti$/, 'Scanning… $1 files seen'], [/^Finita: (.+) file visti$/, 'Done: $1 files seen'],
    [/^Elenco delle voci da verificare salvato in: (.+)$/, 'List of items to check saved in: $1'], [/^Registrata (.+)$/, 'Recorded $1'], [/^Emessa: (.+)$/, 'Issued: $1'], [/^Esportato in (.+)$/, 'Exported to $1'],
    [/^Configurazione, schede, mappa e giornale si trovano in: (.+)$/, 'Settings, records, map and journal are stored in: $1'],
    [/^Cartella principale: (.+?)( \(verrà creata\))?$/, (m, a, b) => 'Main folder: ' + a + (b ? ' (will be created)' : '')],
    [/^Collegamento a (.+) \(facoltativo\)$/, 'Link to $1 (optional)'], [/^Registro: (.+)$/, 'Register: $1'], [/^Crea (.+)$/, 'Create $1'],
    [/^Nuovo (.+)$/, 'New $1'], [/^Nuova (.+)$/, 'New $1'],
    [/^Angoli arrotondati: (\d+) px$/, 'Rounded corners: $1 px'],
    // errori del nucleo
    [/^Lingua non valida \(it, en\)$/, 'Invalid language (it, en)'], [/^Il file (.+) è aperto in un altro programma \(Excel\?\): chiudilo e riprova$/, 'The file $1 is open in another program (Excel?): close it and try again'],
    [/^copiato, ma l'originale non si è potuto togliere: (.+)$/, 'copied, but the original could not be removed: $1'],
    [/^Anno non valido$/, 'Invalid year'], [/^Numero non valido$/, 'Invalid number'], [/^Stato non valido$/, 'Invalid status'], [/^Riga non trovata$/, 'Row not found'], [/^Scheda non trovata$/, 'Record not found'],
    [/^Archiviata: annulla l'archiviazione per modificarla$/, 'Archived: undo the archiving to edit it'],
    [/^Argomento «(.+)» non valido: (.+)$/, 'Invalid argument "$1": $2'], [/^CSV non valido: (.+)$/, 'Invalid CSV: $1'], [/^JSON non valido: (.+)$/, 'Invalid JSON: $1'], [/^Modello non valido: (.+)$/, 'Invalid pattern: $1'],
    [/^Campo sconosciuto: (.+)$/, 'Unknown field: $1'], [/^Colonna sconosciuta: (.+)$/, 'Unknown column: $1'], [/^Comando sconosciuto: (.+)$/, 'Unknown command: $1'],
    [/^Cartella «(.+)»: nome non valido \(niente \/ \\ : \* \? " < > \|\)$/, 'Folder "$1": invalid name (no / \\ : * ? " < > |)'],
    [/^Ci sono (\d+) non conformità aperte: chiudile prima \(ISO 9001 8\.7\)$/, 'There are $1 open nonconformities: close them first (ISO 9001 8.7)'],
    [/^Colonna extra ripetuta: (.+)$/, 'Duplicate extra column: $1'], [/^Colonne extra: nomi non vuoti e senza «;»$/, 'Extra columns: non-empty names without ";"'],
    [/^Compila (.+) e oggetto$/, 'Fill in $1 and subject'], [/^Con (\d+) (.+) nel registro non si elimina$/, 'With $1 $2 in the register it cannot be deleted'],
    [/^Conservazione: massimo 99 anni$/, 'Retention: 99 years maximum'], [/^Descrivi la non conformità$/, 'Describe the nonconformity'],
    [/^Due tipi usano la stessa cartella: (.+)$/, 'Two types use the same folder: $1'], [/^Esegui prima la scansione$/, 'Run the scan first'],
    [/^Identificativo di tipo ripetuto: (.+)$/, 'Duplicate type identifier: $1'], [/^Il codice (.+) è già nel registro$/, 'Code $1 is already in the register'],
    [/^Il nome breve è già nelle cartelle di (\d+) (.+): non si può cambiare$/, 'The short name is already in the folders of $1 $2: it cannot be changed'],
    [/^Il nome dell'applicazione deve avere da 1 a 60 caratteri$/, 'The application name must be 1 to 60 characters'],
    [/^Il nome della cartella risulta troppo lungo o termina con un punto: accorcia l'oggetto o cambia la nomenclatura$/, 'The folder name is too long or ends with a dot: shorten the subject or change the naming rule'],
    [/^Il tipo «(.+)» non ha un nome$/, 'Type "$1" has no name'], [/^Indica chi approva$/, 'Say who approves'], [/^Indica il motivo della nuova revisione$/, 'Give the reason for the new revision'],
    [/^La cartella da riordinare non può essere vuota$/, 'The folder to reorganize cannot be empty'], [/^La cartella delle commesse non può essere vuota$/, 'The main folder cannot be empty'], [/^La cartella esiste già$/, 'The folder already exists'],
    [/^La colonna «(.+)» esiste già nel registro$/, 'The column "$1" already exists in the register'], [/^La soglia dei file grandi deve essere almeno 1 MB$/, 'The large-file threshold must be at least 1 MB'],
    [/^Le cartelle di gestione e di archivio devono avere nomi diversi$/, 'The register and archive folders must have different names'],
    [/^Le cifre del numero vanno da 1 a 6$/, 'Number digits range from 1 to 6'], [/^Le cifre della versione vanno da 1 a 4$/, 'Version digits range from 1 to 4'],
    [/^Lo stato «archiviato» deve essere diverso dagli altri stati$/, 'The "archived" status must differ from the other statuses'], [/^Lo stato «chiuso» deve essere uno degli stati$/, 'The "closed" status must be one of the statuses'],
    [/^Manca: (.+)$/, 'Missing: $1'],
    [/^Modello «(.+)»: campo vuoto$/, 'Pattern "$1": empty field'], [/^Modello «(.+)»: deve contenere \{(.+)\}$/, 'Pattern "$1": must contain {$2}'],
    [/^Modello «(.+)»: i caratteri < > : " \/ \\ \| \? \* non sono ammessi nei nomi di cartelle$/, 'Pattern "$1": the characters < > : " / \\ | ? * are not allowed in folder names'],
    [/^Modello «(.+)»: il campo \{(.+)\} qui non è disponibile \(usa: (.+)\)$/, 'Pattern "$1": the field {$2} is not available here (use: $3)'],
    [/^Modello «(.+)»: le cifre vanno da 1 a 9$/, 'Pattern "$1": digits range from 1 to 9'], [/^Modello «(.+)»: manca una \}$/, 'Pattern "$1": missing a }'], [/^Modello «(.+)»: \} senza \{$/, 'Pattern "$1": } without {'],
    [/^Modello «(.+)»: «(.+)» non è valido$/, 'Pattern "$1": "$2" is not valid'],
    [/^Nessuna operazione da annullare$/, 'No operation to undo'], [/^Nome breve già in anagrafica: (.+)$/, 'Short name already in the address book: $1'], [/^Nome del file README non valido$/, 'Invalid README file name'],
    [/^Per chiudere servono la causa e l'azione correttiva \(ISO 9001 10\.2\)$/, 'To close it, the cause and the corrective action are needed (ISO 9001 10.2)'],
    [/^Percorso vuoto$/, 'Empty path'], [/^Prefisso della quarantena non valido$/, 'Invalid quarantine prefix'],
    [/^Prima di chiudere serve l'approvazione \(verifica e rilascio, ISO 9001 8\.6\)$/, 'Approval is needed before closing (verification and release, ISO 9001 8.6)'],
    [/^Profilo sconosciuto: (.+)$/, 'Unknown profile: $1'], [/^Questa origine è già in mappa$/, 'This source is already in the map'], [/^Riapri la commessa prima di revisionarla$/, 'Reopen it before revising it'],
    [/^Serve almeno un tipo di progetto$/, 'At least one project type is needed'], [/^Servono il nome esteso e il nome breve$/, 'The full name and the short name are required'],
    [/^Si archivia solo da stato «(.+)» \((.+)\)$/, 'Archiving is only possible from status "$1" ($2)'], [/^Sigla ripetuta: (.+)$/, 'Duplicate code letters: $1'],
    [/^Stati: servono nomi non vuoti e senza «;»$/, 'Statuses: non-empty names without ";" are required'], [/^Stato ripetuto: (.+)$/, 'Duplicate status: $1'],
    [/^Stile dei nomi non valido \(pascal, snake, kebab, maiuscolo\)$/, 'Invalid name style (pascal, snake, kebab, uppercase)'],
    [/^Tema: carattere non valido \(sistema, serif, mono\)$/, 'Theme: invalid font (system, serif, mono)'], [/^Tema: il colore principale deve essere nel formato #rrggbb$/, 'Theme: the main color must be in #rrggbb format'],
    [/^Tema: il raggio degli angoli va da 0 a 20$/, 'Theme: the corner radius ranges from 0 to 20'], [/^Tema: modo non valido \(auto, chiaro, scuro\)$/, 'Theme: invalid mode (auto, light, dark)'],
    [/^Tipo non valido: (.+)$/, 'Invalid type: $1'], [/^Tipo «(.+)»: il colore deve essere #rrggbb$/, 'Type "$1": the color must be #rrggbb'],
    [/^Tipo «(.+)»: l'identificativo deve usare solo minuscole, cifre e _$/, 'Type "$1": the identifier may only use lowercase letters, digits and _'],
    [/^Tipo «(.+)»: la cartella coincide con gestione o archivio$/, 'Type "$1": the folder is the same as the register or archive folder'], [/^Tipo «(.+)»: nome cartella non valido$/, 'Type "$1": invalid folder name'],
    [/^Tipo «(.+)»: serve una sigla di lettere e cifre, perché il modello usa \{sigla\}$/, 'Type "$1": code letters are needed, because the pattern uses {sigla}'], [/^Tipo «(.+)»: sottocartella non valida «(.+)»$/, 'Type "$1": invalid subfolder "$2"'],
    [/^Usa un percorso relativo, senza '\.\.': (.+)$/, "Use a relative path, without '..': $1"],
    [/^Vocabolario: il genere di «(.+)» deve essere f o m$/, 'Vocabulary: the gender of "$1" must be f or m'], [/^Vocabolario: «(.+)» deve avere da 1 a 30 caratteri$/, 'Vocabulary: "$1" must be 1 to 30 characters'],
    [/^(.+) non trovata nel registro$/, '$1 not found in the register'], [/^(.+) non è nel registro$/, '$1 is not in the register'], [/^(.+) non presente in anagrafica$/, '$1 not in the address book'],
    [/^(.+) non trovata$/, '$1 not found'], [/^(.+) non è leggibile: (.+)$/, '$1 is not readable: $2'],
    [/^Il file è troppo grande per essere un profilo$/, 'The file is too large to be a profile'], [/^Percorso non consentito$/, 'Path not allowed'],
    // combinazioni usate nell'interfaccia
    [/^Impossibile avviare: (.+)$/, 'Cannot start: $1'],
  ];
  let lang = 'it';
  const orig = new WeakMap(), last = new WeakMap(), origA = new WeakMap(), tracked = new Set();
  function tr(s) {
    if (lang !== 'en' || typeof s !== 'string') return s;
    const t = s.trim(); if (!t) return s;
    let r = Object.prototype.hasOwnProperty.call(EX, t) ? EX[t] : undefined;
    if (r === undefined) for (const [re, rep] of PT) { if (re.test(t)) { r = t.replace(re, rep); break; } }
    return r === undefined ? s : s.replace(t, r);
  }
  const skip = n => {
    const e = n.parentElement; if (!e) return true;
    if (e.closest('script,style,pre,textarea,[data-nt],[data-t]')) return true;
    return e.tagName === 'TD' && !e.classList.contains('empty') && !e.hasAttribute('data-tr');
  };
  function dentro(n) {
    if (n.nodeType === 3) {
      if (skip(n)) return;
      if (orig.has(n) && n.nodeValue === last.get(n)) return;
      const v = tr(n.nodeValue);
      if (v !== n.nodeValue) { orig.set(n, n.nodeValue); last.set(n, v); n.nodeValue = v; tracked.add(new WeakRef(n)); }
    } else if (n.nodeType === 1) {
      if (!n.closest('[data-nt],[data-t]') || n.hasAttribute('data-t') === false) attr(n);
      for (const c of n.childNodes) dentro(c);
    }
  }
  const ATT = ['placeholder', 'title', 'aria-label'];
  function attr(e) {
    if (e.closest('[data-nt]')) return;
    for (const a of ATT) {
      if (!e.hasAttribute(a)) continue;
      const rec = origA.get(e) || {}; const cur = e.getAttribute(a);
      if (rec[a] && rec[a].last === cur) continue;
      const v = tr(cur);
      if (v !== cur) { rec[a] = { it: cur, last: v }; origA.set(e, rec); e.setAttribute(a, v); tracked.add(new WeakRef(e)); }
    }
  }
  function ripristina() {
    for (const w of tracked) {
      const n = w.deref(); if (!n) continue;
      if (n.nodeType === 3) { if (orig.has(n) && n.nodeValue === last.get(n)) n.nodeValue = orig.get(n); orig.delete(n); }
      else { const rec = origA.get(n) || {}; for (const a in rec) if (n.getAttribute(a) === rec[a].last) n.setAttribute(a, rec[a].it); origA.delete(n); }
    }
    tracked.clear();
  }
  const obs = new MutationObserver(ms => {
    if (lang !== 'en') return;
    for (const m of ms) {
      if (m.type === 'childList') m.addedNodes.forEach(dentro);
      else if (m.type === 'characterData') dentro(m.target);
      else if (m.type === 'attributes') attr(m.target);
    }
    obs.takeRecords();
  });
  function set(l) {
    if (l === lang) return;
    obs.disconnect();
    if (l === 'en') { lang = 'en'; dentro(document.body); document.documentElement.lang = 'en'; }
    else { ripristina(); lang = 'it'; document.documentElement.lang = 'it'; }
    obs.observe(document.body, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ATT });
  }
  window.I18N = { tr, set, get lang() { return lang; }, EX, PT };
})();
