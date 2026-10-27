'use strict';
const $ = id => document.getElementById(id);
const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const msgErr = e => (typeof e === 'string' ? e : (e && e.message) || String(e));

async function api(cmd, args = {}) {
  try { return await window.__TAURI__.core.invoke('chiama', { cmd, args }); } catch (e) { throw new Error(msgErr(e)); }
}
async function nativo(cmd, args = {}) {
  try { return await window.__TAURI__.core.invoke(cmd, args); } catch (e) { throw new Error(msgErr(e)); }
}

let S = null;   // stato salvato (S.config = configurazione in uso)
let D = null;   // bozza modificabile in Personalizza e Impostazioni

// ------------------------------------------------------------------ tema
const PALETTE = [['Verde bosco', '#1f5f55'], ['Blu', '#2f5d9e'], ['Terracotta', '#b5532f'], ['Viola', '#6b4fa3'], ['Grafite', '#3d4852'], ['Ocra', '#8a6212'], ['Rosa antico', '#a8425f']];
function hexRgb(h) { const n = parseInt(h.slice(1), 16); return [n >> 16 & 255, n >> 8 & 255, n & 255]; }
function rgbHex(c) { return '#' + c.map(v => v.toString(16).padStart(2, '0')).join(''); }
function rgbHsl([r, g, b]) {
  r /= 255; g /= 255; b /= 255;
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn, l = (mx + mn) / 2;
  let h = 0, s = 0;
  if (d) {
    s = d / (1 - Math.abs(2 * l - 1));
    h = mx === r ? ((g - b) / d) % 6 : mx === g ? (b - r) / d + 2 : (r - g) / d + 4;
    h *= 60; if (h < 0) h += 360;
  }
  return [h, s * 100, l * 100];
}
function hslRgb(h, s, l) {
  s /= 100; l /= 100;
  const c = (1 - Math.abs(2 * l - 1)) * s, x = c * (1 - Math.abs((h / 60) % 2 - 1)), m = l - c / 2;
  const t = h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x] : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
  return t.map(v => Math.round((v + m) * 255));
}
function lum([r, g, b]) { const f = v => { v /= 255; return v <= .03928 ? v / 12.92 : Math.pow((v + .055) / 1.055, 2.4); }; return .2126 * f(r) + .7152 * f(g) + .0722 * f(b); }
function testoSu(rgb) { const L = lum(rgb); return (1.05 / (L + .05)) >= ((L + .05) / 0.0575) ? '#ffffff' : '#101412'; }
const hsl = (h, s, l) => `hsl(${h.toFixed(0)} ${s.toFixed(0)}% ${l.toFixed(0)}%)`;

function modoEffettivo(t) {
  return t.modo === 'auto' ? (matchMedia('(prefers-color-scheme: dark)').matches ? 'scuro' : 'chiaro') : t.modo;
}
function applicaTema(t) {
  const root = document.documentElement, modo = modoEffettivo(t), scuro = modo === 'scuro';
  const base = hexRgb(t.accento), [h, s, l] = rgbHsl(base);
  const sn = Math.min(Math.max(s * .25, 6), 14);
  let acc = base;
  if (scuro) acc = hslRgb(h, Math.min(s, 70), Math.max(l, 64));
  const v = {
    '--accent': rgbHex(acc), '--accent-fg': testoSu(acc),
    '--bg': scuro ? hsl(h, sn, 9) : hsl(h, sn, 95), '--surface': scuro ? hsl(h, sn, 12.5) : hsl(h, sn, 98.5),
    '--fg': scuro ? hsl(h, sn, 91) : hsl(h, sn + 6, 12), '--muted': scuro ? hsl(h, sn, 64) : hsl(h, sn, 37),
    '--line': scuro ? hsl(h, sn, 23) : hsl(h, sn, 86),
    '--ok': scuro ? '#6cc08a' : '#2f7d4f', '--warn': scuro ? '#d9aa4c' : '#8a5f0e', '--bad': scuro ? '#e08577' : '#a63d32',
    '--ok-bg': scuro ? '#1f3427' : '#e1f0e6', '--warn-bg': scuro ? '#3a3019' : '#f6ebd0', '--bad-bg': scuro ? '#3d2420' : '#f5dfdb',
    '--code-bg': scuro ? '#0f1514' : '#1d2623', '--code-fg': '#d9e6e0',
    '--r': t.raggio + 'px', '--pad': t.compatto ? '11px' : '16px', '--gap': t.compatto ? '12px' : '18px',
    '--font': t.carattere === 'serif' ? 'Georgia,"Iowan Old Style",serif' : t.carattere === 'mono' ? 'ui-monospace,Consolas,monospace' : 'system-ui,"Segoe UI",sans-serif',
    '--font-titoli': t.titoli_serif ? 'Georgia,"Iowan Old Style",serif' : 'var(--font)',
  };
  for (const k in v) root.style.setProperty(k, v[k]);
  root.dataset.tema = modo;
  document.body.classList.toggle('compatto', !!t.compatto);
}
matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => D && applicaTema(D.tema));

// ------------------------------------------------------------------ vocabolario
const en = () => !!D && D.app.lingua === 'en';
const LOC = () => en() ? 'en' : 'it';
const cap = s => s ? s[0].toUpperCase() + s.slice(1) : s;
function voc(testo) {
  const v = D.vocabolario, c = D.cartelle;
  const m = {
    commessa: v.commessa, commesse: v.commesse, cliente: v.cliente, clienti: v.clienti,
    nuova_commessa: (en() ? 'New ' : v.genere_commessa === 'f' ? 'Nuova ' : 'Nuovo ') + v.commessa,
    nuovo_cliente: (en() ? 'New ' : v.genere_cliente === 'f' ? 'Nuova ' : 'Nuovo ') + v.cliente,
    archivio: c.archivio, gestione: c.gestione,
  };
  return testo.replace(/\{(\w+)\}/g, (x, k) => {
    if (k in m) return m[k];
    const lk = k[0].toLowerCase() + k.slice(1);
    return lk in m && lk !== k ? cap(m[lk]) : x;
  });
}
function applicaTesti() {
  I18N.set(en() ? 'en' : 'it');
  document.querySelectorAll('[data-t]').forEach(el => { el.textContent = voc(I18N.tr(el.dataset.t)); });
  $('h_nome').textContent = D.app.nome;
  $('h_sub').textContent = D.app.sottotitolo;
  document.title = D.app.nome;
  const tp = tipoSel();
  $('l_cliente').textContent = tp && tp.etichetta_ente ? tp.etichetta_ente : cap(D.vocabolario.cliente);
  $('c_tit').textContent = editC ? I18N.tr('Modifica scheda') : voc('{nuovo_cliente}');
}

// ------------------------------------------------------------------ utilità interfaccia
function ok(id, t) { $(id).className = 'msg okm'; $(id).textContent = t; }
function ko(id, t) { $(id).className = 'msg err'; $(id).textContent = t; }
function mostra(id, fn) { return fn().catch(e => ko(id, e.message)); }
function gb(b) { return b >= 1073741824 ? (b / 1073741824).toFixed(1) + ' GB' : b >= 1048576 ? Math.round(b / 1048576) + ' MB' : b >= 1024 ? Math.round(b / 1024) + ' KB' : b + ' B'; }
function elenco(es) {
  const c = {}; es.forEach(e => c[e.esito] = (c[e.esito] || 0) + 1);
  const rias = Object.entries(c).map(([k, v]) => k + ': ' + v).join('   ') || 'nessuna voce';
  const righe = es.slice(0, 400).map(e => '[' + e.esito + '] ' + e.src + (e.dst ? '\n      -> ' + e.dst : '') + (e.msg ? '   (' + e.msg + ')' : ''));
  return rias + '\n\n' + righe.join('\n') + (es.length > 400 ? '\n… altre ' + (es.length - 400) + ' voci' : '');
}
function azioni(l) { return l.map(a => a.azione + ': ' + a.percorso).join('\n'); }
function vaiTab(id) { document.querySelector('.tabs button[data-p="' + id + '"]').click(); }
const sporca = () => JSON.stringify(D) !== JSON.stringify(S.config);
function aggiornaBar() { $('salvabar').hidden = !sporca(); if (sporca()) { $('sv_msg').className = 'msg'; $('sv_msg').textContent = 'Modifiche non salvate'; } }
function usa(campo) { const n = S.config.nomenclatura, re = new RegExp('\\{' + campo + '(:\\d+)?\\}'); return re.test(n.codice) || re.test(n.cartella); }
function slug(s) { return s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, ''); }
function pulisci(s) {
  const p = s.normalize('NFD').replace(/[̀-ͯ]/g, '').split(/[^A-Za-z0-9]+/).filter(Boolean);
  const st = S.config.nomenclatura.stile_nomi;
  if (st === 'snake') return p.map(w => w.toLowerCase()).join('_');
  if (st === 'kebab') return p.map(w => w.toLowerCase()).join('-');
  if (st === 'maiuscolo') return p.map(w => w.toUpperCase()).join('');
  return p.map(w => w[0].toUpperCase() + w.slice(1)).join('');
}
const tipoSel = () => (S && S.config.tipi.find(t => t.id === $('tipo').value)) || null;

document.querySelectorAll('.tabs button').forEach(b => b.onclick = () => {
  document.querySelectorAll('.tabs button').forEach(x => { x.setAttribute('aria-selected', x === b); $(x.dataset.p).hidden = x !== b; });
  const p = b.dataset.p;
  if (p === 'p2') caricaMappa().catch(() => { });
  if (p === 'p4') caricaRegistro().catch(e => ko('reg_msg', e.message));
  if (p === 'p5') caricaClienti().catch(e => ko('c_msg', e.message));
  if (p === 'p7') anteprimaP();
  if (p === 'p8') caricaQualita().catch(e => ko('nc_msg', e.message));
});

// ------------------------------------------------------------------ clienti
let CL = [], editC = null, breveManuale = false;
function opzioniCliente(sel) {
  $('cliente').innerHTML = '<option value="">Scegli…</option>' + CL.map(c => '<option value="' + esc(c.breve) + '">' + esc(c.nome) + ' (' + esc(c.breve) + ')</option>').join('') + '<option value="__nuovo">+ ' + esc(voc('{nuovo_cliente}')) + '…</option>';
  $('cliente').value = CL.some(c => c.breve === sel) ? sel : '';
}
async function caricaClienti() {
  CL = await api('clienti');
  opzioniCliente($('cliente').value);
  const nomeTipo = id => (S.config.tipi.find(t => t.id === id) || { nome: id }).nome;
  $('clienti').innerHTML = CL.length ? CL.map(c => '<tr><td class="m nw">' + esc(c.breve) + '</td><td>' + esc(c.nome) + '</td><td>' + esc(nomeTipo(c.tipo)) + '</td><td class="n">' + c.commesse + (c.aperte ? ' (' + c.aperte + ' aperte)' : '') + '</td><td>' + esc(c.referente) + '</td><td>' + esc(c.note) + '</td><td><button class="btn alt pic" data-m="' + esc(c.breve) + '">Modifica</button></td></tr>').join('')
    : '<tr><td colspan="7" class="empty">Nessuna scheda ancora. Compila il modulo a sinistra e premi Salva.</td></tr>';
}
function formCliente(c) {
  editC = c ? c.breve : null; breveManuale = !!c;
  $('c_nome').value = c ? c.nome : ''; $('c_breve').value = c ? c.breve : ''; $('c_tipo').value = c ? c.tipo : S.config.tipi[0].id;
  $('c_ref').value = c ? c.referente : ''; $('c_note').value = c ? c.note : '';
  $('c_breve').disabled = !!(c && c.commesse > 0); $('c_del').hidden = !(c && c.commesse === 0);
  $('c_msg').textContent = ''; applicaTesti();
}
$('clienti').addEventListener('click', e => { const b = e.target.closest('button[data-m]'); if (!b) return; formCliente(CL.find(c => c.breve === b.dataset.m)); $('c_nome').focus(); });
$('c_nome').addEventListener('input', () => { if (!breveManuale) $('c_breve').value = pulisci($('c_nome').value); });
$('c_breve').addEventListener('input', () => { breveManuale = true; });
$('c_reset').onclick = () => formCliente(null);
$('c_save').onclick = () => mostra('c_msg', async () => {
  const v = await api('cliente_salva', { richiesta: { vecchio: editC, nome: $('c_nome').value, breve: $('c_breve').value, tipo: $('c_tipo').value, referente: $('c_ref').value, note: $('c_note').value } });
  const sel = $('cliente').value; await caricaClienti(); formCliente(null); ok('c_msg', 'Salvato: ' + v.breve); if (sel) opzioniCliente(sel);
});
$('c_del').onclick = () => mostra('c_msg', async () => { await api('cliente_elimina', { breve: editC }); await caricaClienti(); formCliente(null); ok('c_msg', 'Scheda eliminata'); });

// ------------------------------------------------------------------ nuova commessa
function riempiTipi() {
  const opz = S.config.tipi.map(t => '<option value="' + esc(t.id) + '">' + esc(t.nome) + ' (' + esc(t.dir) + ')</option>').join('');
  const prima = $('tipo').value;
  $('tipo').innerHTML = opz; $('c_tipo').innerHTML = opz;
  if (S.config.tipi.some(t => t.id === prima)) $('tipo').value = prima;
}
function formExtra() {
  const extra = S.config.registro.colonne_extra;
  const vecchi = {}; document.querySelectorAll('#extra_nuova input').forEach(i => vecchi[i.dataset.x] = i.value);
  $('extra_nuova').innerHTML = extra.map(c => '<label>' + esc(c) + '<input data-x="' + esc(c) + '" autocomplete="off" value="' + esc(vecchi[c] || '') + '"></label>').join('');
  $('extra_nuova').querySelectorAll('input').forEach(i => i.addEventListener('input', () => { }));
  const conAnno = usa('anno');
  $('anno').closest('label').hidden = !conAnno;
  $('riga_anno').style.gridTemplateColumns = conAnno ? '90px minmax(0,1fr)' : 'minmax(0,1fr)';
}
function richiesta(sim) {
  const extra = {}; document.querySelectorAll('#extra_nuova input').forEach(i => { if (i.value.trim()) extra[i.dataset.x] = i.value; });
  const c = $('cliente').value;
  return { tipo: $('tipo').value, anno: $('anno').value || String(S.anno), num: $('num').value ? +$('num').value : null, cliente: c && c !== '__nuovo' ? c : null, ogg: $('ogg').value, responsabile: $('resp').value, riesame: $('riesame').checked, extra, simula: sim };
}
let tAnt = null;
function anteprima() {
  clearTimeout(tAnt);
  tAnt = setTimeout(async () => {
    try {
      const r = await api('anteprima', { config: S.config, richiesta: richiesta(true) });
      $('cod').textContent = r.codice; $('nom').textContent = r.nome; $('nom').className = 'sub';
    } catch (e) { $('cod').textContent = '—'; $('nom').textContent = e.message; $('nom').className = 'sub err'; }
  }, 120);
}
async function aggiornaNumero() {
  try { $('num').value = await api('prossimo', { tipo: $('tipo').value, anno: $('anno').value || String(S.anno) }); } catch (e) { /* tipo in modifica */ }
  anteprima();
}
['anno', 'num', 'ogg'].forEach(i => $(i).addEventListener('input', anteprima));
$('anno').addEventListener('change', aggiornaNumero);
$('tipo').addEventListener('change', () => { applicaTesti(); aggiornaNumero(); });
$('cliente').addEventListener('change', () => {
  const v = $('cliente').value;
  if (v === '__nuovo') { $('cliente').value = ''; anteprima(); vaiTab('p5'); formCliente(null); $('c_nome').focus(); return; }
  const c = CL.find(x => x.breve === v);
  if (c && S.config.tipi.some(t => t.id === c.tipo)) { $('tipo').value = c.tipo; applicaTesti(); aggiornaNumero(); } else anteprima();
});
async function nuova(sim) {
  $('n_msg').textContent = '';
  await mostra('n_msg', async () => {
    const j = await api('nuova_commessa', { richiesta: richiesta(sim) });
    $('n_out').textContent = azioni(j.azioni);
    if (!sim) { ok('n_msg', 'Creata ' + j.codice); document.querySelectorAll('#extra_nuova input').forEach(i => i.value = ''); $('riesame').checked = false; await caricaClienti(); await aggiornaNumero(); } else ok('n_msg', 'Simulazione ok');
  });
}
$('n_sim').onclick = () => nuova(true); $('n_go').onclick = () => nuova(false);

// ------------------------------------------------------------------ riordino
let sel = new Set();
async function caricaMappa() {
  const m = await api('mappa');
  $('mappa').innerHTML = m.length ? m.map(r => {
    const cl = { pronto: 'p-ok', fatto: 'p-ok', manca: 'p-warn', conflitto: 'p-bad', errore: 'p-bad' }[r.stato];
    return '<tr><td><input type="checkbox" data-i="' + r.i + '" ' + (r.stato === 'pronto' ? '' : 'disabled') + (sel.has(r.i) ? ' checked' : '') + ' style="width:auto"></td><td class="m">' + esc(r.vecchio) + '</td><td class="m">' + esc(r.nuovo) + '</td><td><span class="pill ' + cl + '">' + r.stato + '</span></td><td>' + esc(r.nota) + '</td><td><button class="btn alt pic" data-x="' + r.i + '">Togli</button></td></tr>';
  }).join('') : '<tr><td colspan="6" class="empty">La mappa è vuota. Aggiungi sopra la prima riga.</td></tr>';
  $('mappa').querySelectorAll('input[type=checkbox]').forEach(c => c.onchange = () => { c.checked ? sel.add(+c.dataset.i) : sel.delete(+c.dataset.i); $('r_go').disabled = true; });
}
$('mappa').addEventListener('click', e => { const b = e.target.closest('button[data-x]'); if (!b) return; mostra('m_msg', async () => { await api('mappa_modifica', { richiesta: { azione: 'elimina', indice: +b.dataset.x } }); sel = new Set(); caricaMappa(); }); });
$('m_add').onclick = () => mostra('m_msg', async () => {
  await api('mappa_modifica', { richiesta: { vecchio: $('m_v').value, nuovo: $('m_n').value, nota: $('m_t').value } });
  $('m_v').value = $('m_n').value = $('m_t').value = ''; ok('m_msg', 'Aggiunta'); caricaMappa();
});
$('r_all').onclick = async () => { const m = await api('mappa'); sel = new Set(m.filter(r => r.stato === 'pronto').map(r => r.i)); $('r_go').disabled = true; caricaMappa(); };
async function riord(sim) {
  $('r_msg').textContent = '';
  await mostra('r_msg', async () => {
    if (!sel.size) throw new Error('Seleziona almeno una riga');
    const j = await api('riordina', { indici: [...sel], simula: sim });
    $('r_out').textContent = elenco(j.esiti);
    if (sim) { $('r_go').disabled = false; ok('r_msg', "Simulazione fatta: controlla l'elenco, poi esegui"); }
    else { $('r_go').disabled = true; ok('r_msg', 'Fatto. Giornale: ' + j.giornale); sel = new Set(); caricaMappa(); }
  });
}
$('r_sim').onclick = () => riord(true); $('r_go').onclick = () => riord(false);

// ------------------------------------------------------------------ pulizia
let timer = null;
async function sonda() {
  const s = await api('scansione_stato');
  $('s_msg').className = 'msg';
  $('s_msg').textContent = s.in_corso ? 'Scansione in corso… ' + s.visti.toLocaleString(LOC()) + ' file visti' : s.fine ? 'Finita: ' + s.visti.toLocaleString(LOC()) + ' file visti' : '';
  if (s.errore) ko('s_msg', s.errore);
  if (s.fine) $('cat').innerHTML = s.categorie.length ? s.categorie.map(c => '<tr><td>' + esc(c.nome) + '</td><td class="n">' + c.n.toLocaleString(LOC()) + '</td><td class="n">' + gb(c.bytes) + '</td><td><span class="pill ' + (c.sicura ? 'p-ok' : 'p-warn') + '">' + (c.sicura ? 'Sicuro' : 'Da verificare') + '</span></td></tr>').join('') : '<tr><td colspan="4" class="empty">Nessun file inutile trovato.</td></tr>';
  if (s.csv) $('s_csv').textContent = 'Elenco delle voci da verificare salvato in: ' + s.csv;
  if (!s.in_corso) { clearInterval(timer); $('s_go').disabled = false; if (s.fine) $('q_sim').disabled = false; }
}
$('s_go').onclick = async () => {
  $('s_go').disabled = true; $('q_sim').disabled = true; $('q_go').disabled = true;
  await mostra('s_msg', async () => { await api('scansione_avvia'); clearInterval(timer); timer = setInterval(() => sonda().catch(() => { }), 1200); await sonda(); });
  if ($('s_msg').classList.contains('err')) $('s_go').disabled = false;
};
async function quar(sim) {
  $('q_msg').textContent = '';
  await mostra('q_msg', async () => {
    const j = await api('quarantena', { simula: sim });
    $('q_out').textContent = 'Quarantena: ' + j.quarantena + '\n' + elenco(j.esiti);
    if (sim) { $('q_go').disabled = false; ok('q_msg', 'Simulazione fatta'); } else { $('q_go').disabled = true; ok('q_msg', 'Fatto. Giornale: ' + j.giornale); }
  });
}
$('q_sim').onclick = () => quar(true); $('q_go').onclick = () => quar(false);

// ------------------------------------------------------------------ registro e annulla
async function caricaRegistro() {
  const g = await api('giornali');
  $('g_ult').textContent = g.length ? 'Ultimo giornale: ' + g[0] : 'Nessun giornale: non è stato spostato nulla.';
  const R = await api('registro'), cfg = S.config, ex = cfg.registro.colonne_extra;
  $('reg_h').innerHTML = '<tr><th>Codice</th><th>Ente</th><th>Oggetto</th><th>Rev.</th><th>Responsabile</th><th>Riesame</th><th>Stato</th><th>Approvazione</th><th>Fattura</th>' + ex.map(c => '<th>' + esc(c) + '</th>').join('') + '<th>Conserva fino al</th><th>Chiusa il</th><th></th></tr>';
  const cols = 13 + ex.length;
  $('reg').innerHTML = R.righe.length ? R.righe.map(x => {
    const cod = esc(x['Codice']), arch = x['Stato'] === cfg.registro.stato_archiviato, chiusa = x['Stato'] === cfg.registro.stato_chiuso;
    const st = arch ? '<span class="pill p-warn">' + esc(x['Stato']) + '</span>' : '<select data-s data-r="' + cod + '">' + cfg.registro.stati.map(s => '<option' + (s === x['Stato'] ? ' selected' : '') + '>' + esc(s) + '</option>').join('') + '</select>';
    const appr = x['Approvata il'] ? '<span class="pill p-ok" title="' + esc(x['Approvata da']) + '">' + esc(x['Approvata il']) + '</span>' : '<span class="pill p-warn">da approvare</span>';
    const az = (arch ? '' : '<button class="btn alt pic" data-b="appr" data-a="' + cod + '">Approva…</button> <button class="btn alt pic" data-b="rev" data-a="' + cod + '"' + (chiusa ? ' disabled' : '') + '>Revisione…</button> ') + (chiusa ? '<button class="btn alt pic" data-a="' + cod + '">Archivia…</button>' : '');
    return '<tr><td class="m">' + cod + '</td><td>' + esc(x['Ente']) + '</td><td>' + esc(x['Oggetto']) + '</td><td class="n">' + esc(x['Revisione'] || '00') + '</td>'
      + '<td><input data-resp data-r="' + cod + '" value="' + esc(x['Responsabile'] || '') + '"' + (arch ? ' disabled' : '') + '></td>'
      + '<td><input type="checkbox" data-ries data-r="' + cod + '" style="width:auto"' + (x['Riesame il'] ? ' checked' : '') + (arch ? ' disabled' : '') + ' title="' + esc(x['Riesame il'] || 'non eseguito') + '"></td>'
      + '<td>' + st + '</td><td>' + appr + '</td>'
      + '<td><input data-f data-r="' + cod + '" value="' + esc(x['Fattura'] || '') + '" placeholder="n. fattura"' + (arch ? ' disabled' : '') + '></td>'
      + ex.map(c => '<td><input data-x="' + esc(c) + '" data-r="' + cod + '" value="' + esc(x[c] || '') + '"' + (arch ? ' disabled' : '') + '></td>').join('')
      + '<td class="n">' + esc(x['Conserva fino al'] || '') + '</td><td class="n">' + esc(x['Chiusa il'] || '') + '</td><td class="nw">' + az + '</td></tr>';
  }).join('') : '<tr><td colspan="' + cols + '" class="empty">Il registro è vuoto. Crea la prima voce dalla scheda iniziale.</td></tr>';
}
$('reg').addEventListener('change', async e => {
  const t = e.target, cod = t.dataset.r; if (!cod) return;
  const p = { codice: cod };
  if ('s' in t.dataset) p.stato = t.value; else if ('f' in t.dataset) p.fattura = t.value; else if ('resp' in t.dataset) p.responsabile = t.value; else if ('ries' in t.dataset) p.riesame = t.checked; else if (t.dataset.x) p.extra = { [t.dataset.x]: t.value };
  try { await api('registro_aggiorna', { richiesta: p }); ok('reg_msg', 'Salvato'); if (p.stato !== undefined || p.riesame !== undefined) caricaRegistro(); } catch (err) { ko('reg_msg', err.message); caricaRegistro(); }
});
// approvazione e nuova revisione: una piccola scheda con un campo di testo
let azione = null;
function apriAzione(tipo, cod) {
  azione = { tipo, cod };
  $('az_card').hidden = false; $('az_msg').textContent = ''; $('az_in').value = '';
  if (tipo === 'appr') { $('az_tit').textContent = 'Approvare ' + cod + '?'; $('az_desc').textContent = 'Verifica e rilascio (ISO 9001 8.6): scrivi chi approva. La data è quella di oggi.'; $('az_in').placeholder = 'Nome di chi approva'; }
  else { $('az_tit').textContent = 'Nuova revisione di ' + cod; $('az_desc').textContent = 'Una modifica ai requisiti (8.5.6) crea la revisione successiva e annulla l\'approvazione. Scrivi il motivo.'; $('az_in').placeholder = 'Motivo della modifica'; }
  $('az_in').focus();
}
$('az_no').onclick = () => { azione = null; $('az_card').hidden = true; };
$('az_ok').onclick = () => mostra('az_msg', async () => {
  const p = { codice: azione.cod };
  if (azione.tipo === 'appr') p.approva = $('az_in').value; else p.nuova_revisione = $('az_in').value;
  await api('registro_aggiorna', { richiesta: p });
  azione = null; $('az_card').hidden = true; ok('reg_msg', 'Registrato'); caricaRegistro();
});
let arCod = null;
async function archivia(cod, sim) {
  $('reg_msg').textContent = '';
  try {
    const j = await api('archivia', { codice: cod, simula: sim });
    $('ar_card').hidden = false; $('ar_out').textContent = elenco(j.esiti);
    if (sim) { arCod = cod; $('ar_go').disabled = !j.esiti.every(e => e.esito === 'simulato'); $('ar_tit').textContent = 'Archiviare ' + cod + '?'; }
    else { arCod = null; $('ar_card').hidden = true; ok('reg_msg', 'Archiviata ' + cod + '. Giornale: ' + j.giornale); caricaRegistro(); }
  } catch (err) { ko('reg_msg', err.message); }
}
$('reg').addEventListener('click', e => {
  const b = e.target.closest('button[data-a]'); if (!b) return;
  if (b.dataset.b) apriAzione(b.dataset.b, b.dataset.a); else archivia(b.dataset.a, true);
});
$('ar_go').onclick = () => { if (arCod) archivia(arCod, false); };
$('ar_no').onclick = () => { arCod = null; $('ar_card').hidden = true; };
async function ann(sim) {
  $('a_msg').textContent = '';
  await mostra('a_msg', async () => {
    const j = await api('annulla', { simula: sim });
    $('a_out').textContent = j.giornale + '\n' + elenco(j.esiti);
    if (sim) { $('a_go').disabled = false; ok('a_msg', 'Simulazione fatta'); } else { $('a_go').disabled = true; ok('a_msg', 'Annullato'); caricaRegistro(); }
  });
}
$('a_sim').onclick = () => ann(true); $('a_go').onclick = () => ann(false);

// ------------------------------------------------------------------ qualità
async function caricaQualita() {
  const R = await api('registro');
  $('nc_c').innerHTML = '<option value="">—</option>' + R.righe.map(r => '<option value="' + esc(r['Codice']) + '">' + esc(r['Codice']) + ' · ' + esc(r['Oggetto']) + '</option>').join('');
  const nc = await api('nc_elenco');
  $('nc').innerHTML = nc.length ? nc.map(x => {
    const c = esc(x['Codice NC']), chiusa = x['Stato'] === 'Chiusa';
    return '<tr><td class="m nw">' + c + '</td><td class="m">' + esc(x['Commessa']) + '</td><td>' + esc(x['Descrizione']) + '</td>'
      + '<td><input data-nc="' + c + '" data-k2="causa" value="' + esc(x['Causa']) + '"></td><td><input data-nc="' + c + '" data-k2="azione" value="' + esc(x['Azione correttiva']) + '"></td>'
      + '<td><select data-nc="' + c + '" data-k2="stato">' + ['Aperta', 'In corso', 'Chiusa'].map(s => '<option' + (s === x['Stato'] ? ' selected' : '') + '>' + s + '</option>').join('') + '</select></td>'
      + '<td>' + (x['Efficacia verificata il'] ? '<span class="pill p-ok">' + esc(x['Efficacia verificata il']) + '</span>' : (chiusa ? '<button class="btn alt pic" data-eff="' + c + '">Verificata</button>' : '<span class="sub">—</span>')) + '</td></tr>';
  }).join('') : '<tr><td colspan="7" class="empty">Nessuna non conformità registrata.</td></tr>';
  await caricaEventi();
}
async function caricaEventi() {
  const ev = await api('eventi');
  $('eventi').innerHTML = ev.length ? ev.map(x => '<tr><td class="nw">' + esc(x['Data e ora']) + '</td><td>' + esc(x['Utente']) + '</td><td>' + esc(x['Operazione']) + '</td><td class="m">' + esc(x['Codice']) + '</td><td>' + esc(x['Dettaglio']) + '</td></tr>').join('') : '<tr><td colspan="5" class="empty">Nessuna operazione registrata. Il registro nasce con la struttura base.</td></tr>';
}
$('ev_ref').onclick = () => caricaEventi().catch(e => ko('nc_msg', e.message));
$('nc_add').onclick = () => mostra('nc_msg', async () => {
  const r = await api('nc_nuova', { richiesta: { commessa: $('nc_c').value, descrizione: $('nc_d').value, responsabile: $('nc_r').value } });
  $('nc_d').value = ''; ok('nc_msg', 'Registrata ' + r['Codice NC']); await caricaQualita();
});
$('nc').addEventListener('change', e => {
  const t = e.target, c = t.dataset.nc; if (!c) return;
  mostra('nc_msg', async () => { await api('nc_aggiorna', { richiesta: { codice: c, [t.dataset.k2]: t.value } }); ok('nc_msg', 'Salvato'); if (t.dataset.k2 === 'stato') await caricaQualita(); }).then(() => { if ($('nc_msg').classList.contains('err')) caricaQualita(); });
});
$('nc').addEventListener('click', e => { const b = e.target.closest('button[data-eff]'); if (b) mostra('nc_msg', async () => { await api('nc_aggiorna', { richiesta: { codice: b.dataset.eff, efficacia: true } }); await caricaQualita(); }); });
$('pr_v').onclick = () => mostra('pr_msg', async () => { $('pr_out').textContent = await api('procedura_testo'); $('pr_out').hidden = false; $('pr_msg').textContent = ''; });
$('pr_go').onclick = () => mostra('pr_msg', async () => { const r = await api('procedura_crea', { simula: false }); ok('pr_msg', 'Emessa: ' + r.percorso); caricaEventi(); });

// ------------------------------------------------------------------ impostazioni e personalizzazione (bozza D)
const get = p => p.split('.').reduce((o, k) => o == null ? o : o[k], D);
const set = (p, v) => { const k = p.split('.'), l = k.pop(); k.reduce((o, x) => o[x], D)[l] = v; };
const leggiEl = el => el.type === 'checkbox' ? el.checked : (el.type === 'number' || el.type === 'range') ? +el.value : el.value;
const righe = el => el.value.split('\n').map(x => x.trim()).filter(Boolean);

function riempiFissi() {
  document.querySelectorAll('[data-k]').forEach(el => {
    const v = get(el.dataset.k);
    if (el.type === 'checkbox') el.checked = !!v; else if (el.tagName !== 'SELECT' || el.id !== 'r_chiuso') el.value = v;
  });
  document.querySelectorAll('[data-lista]').forEach(el => { el.value = (get(el.dataset.lista) || []).join('\n'); });
  riempiStati();
  $('a_rag_o').textContent = D.tema.raggio + ' px';
  $('i_dati').textContent = 'Configurazione, schede, mappa e giornale si trovano in: ' + S.home;
  $('cfg').textContent = 'Cartella principale: ' + S.config.cartelle.radice + (S.radice_esiste ? '' : ' (verrà creata)');
  disegnaPreset(); disegnaCategorie();
}
function riempiStati() {
  const st = D.registro.stati; let ch = D.registro.stato_chiuso;
  if (!st.includes(ch)) { ch = st[st.length - 1] || ''; D.registro.stato_chiuso = ch; }
  $('r_chiuso').innerHTML = st.map(s => '<option' + (s === ch ? ' selected' : '') + '>' + esc(s) + '</option>').join('');
}
function cambiato(k) {
  applicaTema(D.tema); applicaTesti(); aggiornaBar();
  $('a_rag_o').textContent = D.tema.raggio + ' px';
  if (k === 'app.lingua') { disegnaProfili(); disegnaPrima(); aggiornaNumero(); }
  if (k === 'registro.stati') riempiStati();
  if (k === 'tema.accento') disegnaPreset();
  anteprimaP();
}
function disegnaPreset() {
  $('a_preset').innerHTML = PALETTE.map(([n, c]) => '<button type="button" title="' + n + '" aria-label="' + n + '" data-c="' + c + '" aria-pressed="' + (D.tema.accento.toLowerCase() === c) + '" style="background:' + c + '"></button>').join('');
}
$('a_preset').addEventListener('click', e => { const b = e.target.closest('button[data-c]'); if (!b) return; D.tema.accento = b.dataset.c; $('a_acc').value = b.dataset.c; cambiato('tema.accento'); });
const CATS = [['cache_python', 'Cache Python'], ['cache_ide', 'Cache IDE (.idea)'], ['checkpoint_jupyter', 'Checkpoint Jupyter'], ['sistema', 'File di sistema'], ['lock_office', 'Lock di Office (~$)'], ['temporanei', 'Temporanei (.tmp)'], ['aux_xml', 'Sidecar GDAL (.aux.xml)']];
function disegnaCategorie() {
  $('u_cats').innerHTML = CATS.map(([id, n]) => '<label class="chk"><input type="checkbox" data-cat="' + id + '"' + (D.pulizia.disattivate.includes(id) ? '' : ' checked') + '> ' + n + '</label>').join('');
}
$('u_cats').addEventListener('change', e => {
  const id = e.target.dataset.cat; if (!id) return;
  const l = D.pulizia.disattivate.filter(x => x !== id); if (!e.target.checked) l.push(id);
  D.pulizia.disattivate = l; cambiato('pulizia.disattivate');
});
function chips(id, campi, inputId) {
  $(id).innerHTML = campi.map(c => '<button type="button" data-ins="{' + c + '}" data-in="' + inputId + '">{' + c + '}</button>').join('');
}
document.addEventListener('click', e => {
  const b = e.target.closest('button[data-ins]'); if (!b) return;
  const el = $(b.dataset.in), a = el.selectionStart ?? el.value.length, z = el.selectionEnd ?? a;
  el.value = el.value.slice(0, a) + b.dataset.ins + el.value.slice(z); el.focus();
  el.setSelectionRange(a + b.dataset.ins.length, a + b.dataset.ins.length); el.dispatchEvent(new Event('input'));
});
document.addEventListener('click', async e => {
  const b = e.target.closest('button[data-sfoglia]'); if (!b) return;
  const el = $(b.dataset.sfoglia);
  try { const r = await nativo('scegli_cartella', { titolo: 'Scegli una cartella', iniziale: el.value }); if (r) { el.value = r; el.dispatchEvent(new Event('input')); } } catch (err) { ko('sv_msg', err.message); $('salvabar').hidden = false; }
});

// anteprima delle nomenclature con la bozza
let tAp = null;
function anteprimaP() {
  clearTimeout(tAp);
  tAp = setTimeout(async () => {
    const t = D.tipi[0];
    if (!t) { $('an_cod').textContent = '—'; $('an_nom').textContent = 'Serve almeno un tipo'; return; }
    try {
      const r = await api('anteprima', { config: D, richiesta: { tipo: t.id, anno: String(S.anno), num: 7, ente: 'Acme Idrica', ogg: 'Relazione idraulica', extra: {}, simula: true } });
      $('an_cod').textContent = r.codice; $('an_nom').textContent = r.nome; $('an_nom').className = 'sub';
      $('an_sub').textContent = t.dir + '/' + r.nome + '/\n' + t.sub.map(s => '  ' + s + '/').join('\n') + (D.nomenclatura.crea_readme ? '\n  ' + D.nomenclatura.nome_readme : '');
    } catch (e) { $('an_cod').textContent = '—'; $('an_nom').textContent = e.message; $('an_nom').className = 'sub err'; $('an_sub').textContent = ''; }
  }, 150);
}

// tipi di progetto
function renderTipi() {
  const salvati = new Set(S.config.tipi.map(t => t.id));
  $('tipi').innerHTML = D.tipi.map((t, i) => `<div class="tipo" style="--tc:${esc(t.colore || D.tema.accento)}">
<div class="testa"><strong>${esc(t.nome || 'Senza nome')}</strong><span class="bar"><button class="btn alt pic" data-az="su" data-i="${i}" ${i === 0 ? 'disabled' : ''} aria-label="Sposta su">↑</button><button class="btn alt pic" data-az="giu" data-i="${i}" ${i === D.tipi.length - 1 ? 'disabled' : ''} aria-label="Sposta giù">↓</button><button class="btn alt pic" data-az="togli" data-i="${i}" ${D.tipi.length < 2 ? 'disabled' : ''}>Rimuovi</button></span></div>
<div class="griglia">
<label>Nome<input data-i="${i}" data-f="nome" value="${esc(t.nome)}"></label>
<label>Identificativo${salvati.has(t.id) ? ' (fisso)' : ''}<input data-i="${i}" data-f="id" value="${esc(t.id)}" ${salvati.has(t.id) ? 'disabled' : ''} spellcheck="false"></label>
<label>Cartella<input data-i="${i}" data-f="dir" value="${esc(t.dir)}" spellcheck="false"></label>
<label>Sigla (per {sigla})<input data-i="${i}" data-f="sigla" value="${esc(t.sigla)}" maxlength="8" spellcheck="false"></label>
<label>Come si chiama il committente<input data-i="${i}" data-f="etichetta_ente" value="${esc(t.etichetta_ente)}"></label>
<label>Colore del tipo<span class="bar"><input type="color" data-i="${i}" data-f="colore" value="${esc(t.colore || D.tema.accento)}"><button type="button" class="btn alt pic" data-az="nocolore" data-i="${i}">Nessuno</button></span></label>
</div>
<label>Sottocartelle, una per riga<textarea data-i="${i}" data-f="sub">${esc(t.sub.join('\n'))}</textarea></label>
<label>Modello del file di presentazione (vuoto = modello generale). Segnaposto: {codice} {ente} {oggetto} {tipo} {data} {stato} {versioni} {cartella}<textarea data-i="${i}" data-f="readme" style="font-family:var(--mono)">${esc(t.readme)}</textarea></label>
</div>`).join('');
}
$('tipi').addEventListener('input', e => {
  const el = e.target, i = +el.dataset.i, f = el.dataset.f; if (f === undefined) return;
  const t = D.tipi[i];
  if (f === 'sub') t.sub = righe(el); else t[f] = el.value;
  if (f === 'nome' && !S.config.tipi.some(x => x.id === t.id)) {
    t.id = slug(t.nome); const idEl = $('tipi').querySelector('[data-i="' + i + '"][data-f="id"]'); if (idEl) idEl.value = t.id;
  }
  if (f === 'colore') e.target.closest('.tipo').style.setProperty('--tc', el.value);
  cambiato('tipi');
});
$('tipi').addEventListener('click', e => {
  const b = e.target.closest('button[data-az]'); if (!b) return;
  const i = +b.dataset.i, az = b.dataset.az;
  if (az === 'su' && i > 0) [D.tipi[i - 1], D.tipi[i]] = [D.tipi[i], D.tipi[i - 1]];
  if (az === 'giu' && i < D.tipi.length - 1) [D.tipi[i + 1], D.tipi[i]] = [D.tipi[i], D.tipi[i + 1]];
  if (az === 'togli' && D.tipi.length > 1) D.tipi.splice(i, 1);
  if (az === 'nocolore') D.tipi[i].colore = '';
  renderTipi(); cambiato('tipi');
});
$('t_add').onclick = () => {
  let n = 1, id; do { id = 'tipo_' + n++; } while (D.tipi.some(t => t.id === id));
  const dir = String((D.tipi.length + 1) * 10) + '_NUOVO_TIPO';
  D.tipi.push({ id, nome: 'Nuovo tipo', dir, sigla: 'NUO' + (n - 1), sub: ['01_Input', '02_Lavoro', '03_Output'], etichetta_ente: 'Cliente', readme: '', colore: '' });
  D.tipi[D.tipi.length - 1].id = slug('Nuovo tipo') + '_' + (D.tipi.length);
  renderTipi(); cambiato('tipi');
  const ult = $('tipi').lastElementChild; if (ult) ult.scrollIntoView({ block: 'center' });
};

// profili
function profiliLingua() { return (en() ? S.profili_en : S.profili_it) || S.profili; }
function disegnaProfili() {
  $('pf_sel').innerHTML = profiliLingua().map(p => '<option value="' + esc(p.id) + '">' + esc(p.nome) + '</option>').join('');
  const d = () => { const p = profiliLingua().find(x => x.id === $('pf_sel').value); $('pf_desc').textContent = p ? p.descrizione : ''; };
  $('pf_sel').onchange = d; d();
}
function rinfrescaTutto() {
  applicaTema(D.tema); riempiFissi(); renderTipi(); riempiTipi(); formExtra(); applicaTesti(); aggiornaBar(); anteprima(); anteprimaP();
}
$('pf_app').onclick = () => mostra('pf_msg', async () => {
  D = await api('profilo_applica', { id: $('pf_sel').value, config: D });
  rinfrescaTutto(); ok('pf_msg', 'Modello applicato alla bozza: controlla e premi Salva');
});
$('pf_exp').onclick = () => mostra('pf_msg', async () => {
  const testo = await api('profilo_esporta', { config: D });
  const dove = await nativo('salva_file_testo', { nome: 'profilo-folio.json', testo });
  if (dove) ok('pf_msg', 'Esportato in ' + dove);
});
$('pf_imp').onclick = () => mostra('pf_msg', async () => {
  const testo = await nativo('apri_file_testo');
  if (!testo) return;
  D = await api('profilo_importa', { testo, config: D });
  rinfrescaTutto(); ok('pf_msg', 'Profilo importato nella bozza: controlla e premi Salva');
});

// salvataggio
$('sv_ok').onclick = async () => {
  try {
    await api('salva_config', { config: D });
    S = await api('stato'); D = structuredClone(S.config);
    rinfrescaTutto(); await caricaClienti().catch(() => { });
    $('salvabar').hidden = false; ok('sv_msg', 'Salvato');
    setTimeout(aggiornaBar, 1600);
  } catch (e) { ko('sv_msg', e.message); }
};
$('sv_no').onclick = () => { D = structuredClone(S.config); rinfrescaTutto(); };

async function struttura(sim) {
  $('b_msg').textContent = '';
  await mostra('b_msg', async () => {
    if (sporca()) throw new Error('Salva prima le modifiche (barra in basso)');
    const j = await api('crea_struttura', { simula: sim });
    $('b_out').textContent = azioni(j);
    ok('b_msg', sim ? 'Simulazione ok' : 'Struttura creata');
    if (!sim) { S = await api('stato'); riempiFissi(); }
  });
}
$('b_sim').onclick = () => struttura(true); $('b_go').onclick = () => struttura(false);
$('b_dati').onclick = () => nativo('apri_percorso', { percorso: S.home }).catch(e => ko('b_msg', e.message));

// ------------------------------------------------------------------ primo avvio
function disegnaPrima() {
  $('prima').hidden = !S.prima_volta;
  $('prima_profili').innerHTML = profiliLingua().map(p => '<div class="prof"><strong>' + esc(p.nome) + '</strong><span class="sub">' + esc(p.descrizione) + '</span><button class="btn alt" data-prof="' + esc(p.id) + '">Parti da qui</button></div>').join('');
}
document.querySelectorAll('[data-lingua]').forEach(b => b.addEventListener('click', () => { D.app.lingua = b.dataset.lingua; applicaTesti(); disegnaPrima(); disegnaProfili(); }));
$('prima_profili').addEventListener('click', async e => {
  const b = e.target.closest('button[data-prof]'); if (!b) return;
  try {
    D = await api('profilo_applica', { id: b.dataset.prof, config: D });
    await api('salva_config', { config: D });
    S = await api('stato'); D = structuredClone(S.config);
    disegnaPrima(); rinfrescaTutto(); vaiTab('p6');
  } catch (err) { b.textContent = err.message; }
});

// ------------------------------------------------------------------ avvio
function collega() {
  // Impostazioni: cartelle
  $('i_radice').dataset.k = 'cartelle.radice'; $('i_origine').dataset.k = 'cartelle.origine'; $('i_scan').dataset.lista = 'cartelle.radici_scansione';
  document.querySelectorAll('[data-k]').forEach(el => el.addEventListener('input', () => { set(el.dataset.k, leggiEl(el)); cambiato(el.dataset.k); }));
  document.querySelectorAll('[data-lista]').forEach(el => el.addEventListener('input', () => { set(el.dataset.lista, righe(el)); cambiato(el.dataset.lista); }));
  chips('ch_cod', ['anno', 'num', 'sigla'], 'n_cod');
  chips('ch_car', ['codice', 'ente', 'oggetto', 'anno', 'num', 'sigla', 'tipo', 'data'], 'n_car');
}
(async () => {
  if (!window.__TAURI__) { document.body.innerHTML = '<p style="padding:24px">Questa interfaccia funziona dentro l\'app Folio.</p>'; return; }
  try {
    S = await api('stato'); D = structuredClone(S.config);
    collega(); disegnaProfili(); disegnaPrima();
    rinfrescaTutto();
    await caricaClienti();
    $('anno').value = S.anno; formCliente(null); await aggiornaNumero();
  } catch (e) { document.body.innerHTML = '<p style="padding:24px">Impossibile avviare: ' + esc(e.message) + '</p>'; }
})();
