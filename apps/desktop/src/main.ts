import './styles.css';
import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { clearHighlights, highlight } from './search';
import type { Config, Navigation, Payload, Theme } from './types';

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const viewport = $('viewport');
const article = $('document');
const native = isTauri();
let config: Config = { theme: 'system', font_size: 16, toc: true, zen_mode: false, recent: [] };
let current: Payload | null = null;
let matches: HTMLElement[][] = [];
let matchIndex = -1;
let actions = Promise.resolve();
let toastTimer: ReturnType<typeof setTimeout>;
let reloadTimer: ReturnType<typeof setTimeout>;
let searchTimer: ReturnType<typeof setTimeout>;
let preferencesTimer: ReturnType<typeof setTimeout>;
let openingDialog = false;
const systemDark = matchMedia('(prefers-color-scheme: dark)');
const mobile = matchMedia('(max-width: 750px)');
const dark = () => config.theme === 'dark' || (config.theme === 'system' && systemDark.matches);

function notify(message: string, error = false) {
  clearTimeout(toastTimer); const toast = $('toast');
  toast.textContent = message; toast.classList.toggle('error', error); toast.hidden = false;
  toastTimer = setTimeout(() => { toast.hidden = true; }, error ? 8000 : 2000);
}
function enqueue(action: () => Promise<void>) { actions = actions.then(action).catch(error => notify(String(error), true)); }
function applyPreferences() {
  document.documentElement.dataset.theme = dark() ? 'dark' : 'light';
  document.documentElement.style.setProperty('--font-size', `${config.font_size}px`);
  document.body.classList.toggle('toc-hidden', !config.toc);
  document.body.classList.toggle('zen', config.zen_mode);
  $('toggle-toc').setAttribute('aria-pressed', String(config.toc));
  $('toggle-zen').setAttribute('aria-pressed', String(config.zen_mode));
  $('leave-zen').hidden = !config.zen_mode;
  $<HTMLSelectElement>('theme').value = config.theme;
  $('font-reset').textContent = `${config.font_size}px`;
}
function savePreferences() {
  applyPreferences(); clearTimeout(preferencesTimer);
  if (native) preferencesTimer = setTimeout(() => {
    void invoke('save_preferences', { theme: config.theme, fontSize: config.font_size, toc: config.toc, zenMode: config.zen_mode }).catch(error => notify(String(error), true));
  }, 150);
}
function recents(paths: string[]) {
  config.recent = paths; $('recent').replaceChildren();
  for (const path of paths) {
    const li = document.createElement('li'); const button = document.createElement('button');
    button.textContent = path.split(/[/\\]/).at(-1) ?? path; button.title = path;
    button.onclick = () => open(path); li.append(button); $('recent').append(li);
  }
}
function scrollToHeading(id: string) {
  const heading = document.getElementById(id);
  if (heading && article.contains(heading)) { heading.scrollIntoView({ block: 'start' }); heading.tabIndex = -1; heading.focus({ preventScroll: true }); }
  document.body.classList.remove('mobile-outline');
}
function context() {
  const top = viewport.getBoundingClientRect().top;
  let heading: HTMLElement | undefined;
  for (const el of article.querySelectorAll<HTMLElement>('h1,h2,h3,h4,h5,h6')) {
    if (el.getBoundingClientRect().top <= top + 50) heading = el; else break;
  }
  return { id: heading?.id, offset: heading ? heading.getBoundingClientRect().top - top : 0, scroll: viewport.scrollTop };
}
function decorateCode(payload: Payload) {
  article.querySelectorAll<HTMLPreElement>('pre[data-code]').forEach(pre => {
    const block = payload.code_blocks[Number(pre.dataset.code)];
    if (!block) return;
    const wrapper = document.createElement('div'); wrapper.className = 'code-block';
    const toolbar = document.createElement('div'); toolbar.className = 'code-toolbar';
    const label = document.createElement('span'); label.textContent = block.language || 'plain text';
    const button = document.createElement('button'); button.textContent = 'Copy'; button.setAttribute('aria-label', `Copy ${block.language || 'plain text'} code`);
    button.onclick = async () => {
      try { if (native) await invoke('copy_text', { text: block.code }); else await navigator.clipboard.writeText(block.code); button.textContent = 'Copied'; setTimeout(() => { button.textContent = 'Copy'; }, 1500); }
      catch { notify('Unable to copy. Select the code and use Cmd/Ctrl+C.', true); }
    };
    toolbar.append(label, button); pre.before(wrapper); wrapper.append(toolbar, pre);
  });
  article.querySelectorAll('table').forEach(table => {
    const wrapper = document.createElement('div'); wrapper.className = 'table-scroll'; wrapper.tabIndex = 0; wrapper.setAttribute('aria-label', 'Scrollable Markdown table');
    table.before(wrapper); wrapper.append(table);
  });
  article.querySelectorAll('img').forEach(img => {
    if (img.getAttribute('src')?.startsWith('marklight-image://localhost/')) {
      const token = img.getAttribute('src')!.split('/').at(-1)!;
      img.src = convertFileSrc(token, 'marklight-image');
      img.onerror = () => unavailable();
    } else unavailable();
    function unavailable() {
      img.classList.add('blocked-image');
      if (img.nextElementSibling?.classList.contains('image-note')) return;
      const note = document.createElement('span'); note.className = 'image-note';
      note.textContent = `Image unavailable: ${img.alt || 'local image'} · Local raster images must be inside the document directory.`;
      img.after(note);
    }
  });
}
function render(payload: Payload, preserve = false, anchor?: string | null) {
  const reading = preserve ? context() : null;
  current = payload; article.innerHTML = payload.html;
  article.hidden = false; $('welcome').hidden = true;
  $('file-name').textContent = payload.name; $('file-name').title = payload.path;
  document.title = `${payload.name} — Marklight`;
  $('status').textContent = `${payload.metadata.words.toLocaleString()} WORDS · ${payload.metadata.reading_minutes} MIN READ · LIVE RELOAD`;
  decorateCode(payload);
  const outline = $('outline'); outline.replaceChildren();
  for (const heading of payload.headings) {
    const link = document.createElement('a'); link.href = `#${heading.id}`; link.textContent = heading.text;
    link.dataset.heading = heading.id; link.style.paddingLeft = `${10 + (heading.level - 1) * 10}px`;
    link.onclick = event => { event.preventDefault(); scrollToHeading(heading.id); }; outline.append(link);
  }
  if (!payload.headings.length) { const empty = document.createElement('p'); empty.className = 'muted'; empty.textContent = 'No headings in this document.'; outline.append(empty); }
  recents(payload.recent);
  if (reading) {
    const heading = reading.id && document.getElementById(reading.id);
    viewport.scrollTop = heading && article.contains(heading) ? viewport.scrollTop + heading.getBoundingClientRect().top - viewport.getBoundingClientRect().top - reading.offset : reading.scroll;
  } else viewport.scrollTop = 0;
  if (!$('search-bar').hidden) search(false);
  if (anchor) scrollToHeading(anchor);
  updateProgress();
  if (payload.warning) notify(payload.warning, true);
}
function open(path: string, anchor?: string | null) {
  enqueue(async () => { render(await invoke<Payload>('open_document', { path, dark: dark() }), false, anchor); });
}
async function chooseFile() {
  if (openingDialog) return;
  if (!native) { notify('Run the Tauri desktop app to open local files.'); return; }
  openingDialog = true;
  try { const path = await invoke<string | null>('choose_file'); if (path) open(path); }
  catch (error) { notify(String(error), true); } finally { openingDialog = false; }
}
function reload() {
  if (!current || !native) return;
  clearTimeout(reloadTimer);
  reloadTimer = setTimeout(() => enqueue(async () => { render(await invoke<Payload>('reload_document', { dark: dark() }), true); }), 120);
}
function updateProgress() {
  const range = viewport.scrollHeight - viewport.clientHeight;
  $('reading-progress').textContent = current ? `${range > 0 ? Math.round(viewport.scrollTop / range * 100) : 100}%` : '—';
  const top = viewport.getBoundingClientRect().top;
  let active = current?.headings[0]?.id;
  for (const heading of article.querySelectorAll<HTMLElement>('h1,h2,h3,h4,h5,h6')) {
    if (heading.getBoundingClientRect().top <= top + 90) active = heading.id; else break;
  }
  $('outline').querySelectorAll<HTMLAnchorElement>('a').forEach(link => {
    const selected = link.dataset.heading === active;
    link.classList.toggle('active', selected); if (selected) link.setAttribute('aria-current', 'location'); else link.removeAttribute('aria-current');
  });
}
function openSearch() { $('search-bar').hidden = false; $<HTMLInputElement>('search-input').focus(); $<HTMLInputElement>('search-input').select(); }
function closeSearch() { clearTimeout(searchTimer); $('search-bar').hidden = true; clearHighlights(article); matches = []; matchIndex = -1; viewport.focus(); }
function search(scroll = true) {
  matches = highlight(article, $<HTMLInputElement>('search-input').value); matchIndex = -1;
  moveMatch(1, scroll);
}
function moveMatch(direction: number, scroll = true) {
  matches.flat().forEach(mark => mark.classList.remove('current-match'));
  if (!matches.length) { $('match-count').textContent = '0 matches'; return; }
  matchIndex = (matchIndex + direction + matches.length) % matches.length;
  matches[matchIndex].forEach(mark => mark.classList.add('current-match'));
  $('match-count').textContent = `${matchIndex + 1} / ${matches.length}${matches.length === 10000 ? '+' : ''}`;
  if (scroll) matches[matchIndex][0]?.scrollIntoView({ block: 'center' });
}
function toggleToc() {
  if (mobile.matches) { document.body.classList.toggle('mobile-outline'); config.toc = true; } else config.toc = !config.toc;
  savePreferences();
}
function toggleZen() { config.zen_mode = !config.zen_mode; savePreferences(); }
function font(delta: number) { config.font_size = delta === 0 ? 16 : Math.max(12, Math.min(28, config.font_size + delta)); savePreferences(); }
$('open').onclick = chooseFile; $('welcome-open').onclick = chooseFile;
$('toggle-toc').onclick = toggleToc; $('toggle-zen').onclick = toggleZen; $('leave-zen').onclick = toggleZen;
$('toggle-search').onclick = openSearch; $('close-search').onclick = closeSearch;
$('next-match').onclick = () => moveMatch(1); $('previous-match').onclick = () => moveMatch(-1);
$('font-up').onclick = () => font(1); $('font-down').onclick = () => font(-1); $('font-reset').onclick = () => font(0);
$<HTMLSelectElement>('theme').onchange = event => { config.theme = (event.target as HTMLSelectElement).value as Theme; savePreferences(); reload(); };
$<HTMLInputElement>('search-input').oninput = () => { clearTimeout(searchTimer); searchTimer = setTimeout(() => search(), 100); };
$('clear-recent').onclick = () => { enqueue(async () => { if (native) await invoke('clear_recent'); recents([]); }); };
viewport.onscroll = updateProgress;
systemDark.onchange = () => { if (config.theme === 'system') { applyPreferences(); reload(); } };
article.onclick = event => {
  const link = (event.target as Element).closest('a'); if (!link) return;
  event.preventDefault(); const href = link.getAttribute('href'); if (!href) return;
  enqueue(async () => {
    const navigation = await invoke<Navigation>('follow_link', { href });
    if (navigation.kind === 'anchor') scrollToHeading(navigation.id);
    if (navigation.kind === 'markdown') open(navigation.path, navigation.anchor);
  });
};
document.querySelector('.wordmark')!.addEventListener('click', event => { event.preventDefault(); viewport.scrollTop = 0; });
document.addEventListener('keydown', event => {
  const mod = event.metaKey || event.ctrlKey;
  if (mod && event.key.toLowerCase() === 'o') { event.preventDefault(); void chooseFile(); }
  else if (mod && event.key.toLowerCase() === 'f') { event.preventDefault(); openSearch(); }
  else if (mod && event.shiftKey && event.key.toLowerCase() === 't') { event.preventDefault(); toggleToc(); }
  else if (mod && event.shiftKey && event.key.toLowerCase() === 'z') { event.preventDefault(); toggleZen(); }
  else if (mod && ['+', '=', '-', '0'].includes(event.key)) { event.preventDefault(); font(event.key === '0' ? 0 : event.key === '-' ? -1 : 1); }
  else if (event.key === 'Escape') { if (!$('search-bar').hidden) closeSearch(); else if (config.zen_mode) toggleZen(); else document.body.classList.remove('mobile-outline'); }
  else if (event.key === 'Enter' && event.target === $('search-input')) { event.preventDefault(); moveMatch(event.shiftKey ? -1 : 1); }
});
async function startup() {
  applyPreferences();
  if (!native) return;
  const pending = async () => { const path = await invoke<string | null>('take_pending'); if (path) open(path); };
  await listen('open-request', () => { void pending().catch(error => notify(String(error), true)); });
  await listen('menu-open', () => { void chooseFile(); });
  await listen('document-changed', reload);
  await getCurrentWebviewWindow().onDragDropEvent(event => {
    $('drop-overlay').hidden = event.payload.type === 'leave' || event.payload.type === 'drop';
    if (event.payload.type === 'drop' && event.payload.paths[0]) open(event.payload.paths[0]);
  });
  config = await invoke<Config>('get_config'); applyPreferences(); recents(config.recent); await pending();
  await actions;
  await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
  await invoke('reader_ready');
}
void startup().catch(error => notify(String(error), true));
