# Folio

Unique job codes, standard folders, a client address book, guided reorganization, archive and safe cleanup for firms and freelancers. A desktop app for **Windows, macOS and Linux**, inspired by the **ISO 9001** requirements on documented information. Everything is customizable: name, colors, naming rules, project types. The interface is available in **Italian and English**.

**It never deletes anything:** it moves files and folders, never overwrites, every operation starts as a simulation and can be undone.

## Install

Download the installer from the project's **Releases** page:

| System | File | Notes |
|---|---|---|
| Windows 10/11 | `Folio_x.y.z_x64-setup.exe` or `.msi` | If "Windows protected your PC" (SmartScreen) appears: *More info* → *Run anyway*. The installer is not signed. |
| macOS | `Folio_x.y.z_aarch64.dmg` (Apple Silicon) or `_x64.dmg` (Intel) | If the system blocks it: right-click the app → *Open* (first time only). |
| Linux | `.AppImage` (make it executable and run it), `.deb` or `.rpm` | Needs `webkit2gtk` (present on most desktop distributions). |

No Python or other software is needed.

## First start

1. Pick the language (Italiano / English) and a starting model (Professional firm, Research and consulting, Minimal).
2. In **Settings** check the main folder and press **Create structure**.
3. In **Clients** add records; in **New job** pick the client and subject, press *Simulate*, then *Create*.
4. To tidy existing work: **Reorganize** (map "now → after", simulation, run). For cleanup: **Cleanup**.

## Customize

Everything is under **Customize**; changes show immediately and apply after *Save*: appearance (name, colors, theme, font, language), vocabulary, naming templates, project types, statuses, register columns, cleanup rules, and ISO 9001 controls (responsible person, requirements review, revisions, approval before closing, nonconformities, retention, chronological log, documented procedure). Setups can be exported and imported as profiles.

## Language notes

Switching the language changes the interface, error messages, folder README files, the generated procedure and the log labels. Register column names in the CSV files (e.g. "Codice", "Stato") stay as they are, so files remain compatible between languages. Data you typed (client names, statuses, types) is never translated.

## Development

`cargo test -p commesse-core`, `cargo tauri dev`, `cargo tauri build`. See the Italian README for details on releases and signing.
