import './styles.css';
import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { resolveResource } from '@tauri-apps/api/path';
import { clearHighlights, highlight } from './search';
import type { Config, Heading, Navigation, Payload, Theme } from './types';

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const viewport = $('viewport');
const article = $('document');
const documentHeadings = new Map<string, HTMLElement>();
let headingElements: HTMLElement[] = [];
const native = isTauri();
let config: Config = { theme: 'system', font_size: 16, toc: true, zen_mode: false, recent: [] };
let current: Payload | null = null;
let matches: HTMLElement[][] = [];
let matchIndex = -1;
let moreMatches = false;
let searchGeneration = 0;
let searchActions = Promise.resolve();
let searchPending = false;
let actions = Promise.resolve();
let toastTimer: ReturnType<typeof setTimeout>;
let errorRetry: (() => void) | null = null;
let reloadTimer: ReturnType<typeof setTimeout>;
let searchTimer: ReturnType<typeof setTimeout>;
let preferencesTimer: ReturnType<typeof setTimeout>;
let openingDialog = false;
let openRequest = 0;
type RenderTimings = { ipc_ms: number; template_ms: number; decoration_ms: number; chunks_ms: number; attach_ms: number; outline_ms: number; recents_ms: number; scroll_ms: number; progress_ms: number; finish_ms: number; total_ms: number };
let startupTimings: RenderTimings | null = null;
type ChunkHeadings = { headings: HTMLElement[]; previous: HTMLElement | undefined };
let chunkHeadings = new WeakMap<HTMLElement, ChunkHeadings>();
let activeOutline: HTMLAnchorElement | null = null;
const outlineLinks = new Map<string, HTMLAnchorElement>();
const outlineIndexes = new Map<string, number>();
let outlineHeadings: Heading[] = [];
let outlineStart = 0;
let outlineQuery = '';
let activeHeadingId: string | undefined;
const outlinePageSize = 100;
type ReadingContext = { id: string | undefined; offset: number; scroll: number };
type Visit = { path: string; reading: ReadingContext };
type HistoryMode = 'push' | 'back' | 'forward';
const historyBack: Visit[] = [];
const historyForward: Visit[] = [];
const systemDark = matchMedia('(prefers-color-scheme: dark)');
const mobile = matchMedia('(max-width: 750px)');
const dark = () => config.theme === 'dark' || (config.theme === 'system' && systemDark.matches);

function notify(message: string, error = false) {
  clearTimeout(toastTimer); const toast = $('toast');
  toast.textContent = message; toast.classList.toggle('error', error); toast.hidden = false;
  toastTimer = setTimeout(() => { toast.hidden = true; }, error ? 8000 : 2000);
}
function showReaderError(message: string, retry?: () => void) {
  $('reader-error-text').textContent = message;
  errorRetry = retry ?? null;
  $('reader-retry').hidden = !retry;
  $('reader-error').hidden = false;
}
function clearReaderError() {
  $('reader-error').hidden = true;
  errorRetry = null;
}
function enqueue(action: () => Promise<void>) { actions = actions.then(action).catch(error => notify(String(error), true)); }
function yieldToUi() {
  // Message tasks keep working when macOS throttles background WebView timers.
  return new Promise<void>(resolve => {
    const channel = new MessageChannel();
    channel.port1.onmessage = () => { channel.port1.close(); channel.port2.close(); resolve(); };
    channel.port2.postMessage(null);
  });
}
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
  document.querySelector<HTMLElement>('.recent-section')!.hidden = paths.length === 0;
  for (const path of paths) {
    const li = document.createElement('li'); const button = document.createElement('button');
    button.textContent = path.split(/[/\\]/).at(-1) ?? path; button.title = path;
    button.onclick = () => open(path); li.append(button); $('recent').append(li);
  }
}
function scrollToHeading(id: string) {
  const heading = documentHeadings.get(id);
  if (heading) { heading.scrollIntoView({ block: 'start' }); heading.tabIndex = -1; heading.focus({ preventScroll: true }); }
  document.body.classList.remove('mobile-outline');
}
function remember(stack: Visit[], visit: Visit) {
  stack.push(visit);
  if (stack.length > 50) stack.shift();
}
function updateHistoryControls() {
  $<HTMLButtonElement>('history-back').disabled = !historyBack.length;
  $<HTMLButtonElement>('history-forward').disabled = !historyForward.length;
}
function restoreReading(reading: ReadingContext, update = true) {
  const heading = reading.id && documentHeadings.get(reading.id);
  viewport.scrollTop = heading
    ? viewport.scrollTop + heading.getBoundingClientRect().top - viewport.getBoundingClientRect().top - reading.offset
    : reading.scroll;
  if (update) updateProgress();
}
function jumpToHeading(id: string) {
  if (viewport.getAttribute('aria-busy') === 'true') return;
  const previous = current && { path: current.path, reading: context() };
  scrollToHeading(id);
  if (previous) { remember(historyBack, previous); historyForward.length = 0; updateHistoryControls(); }
}
function paintOutline() {
  const outline = $('outline'); outline.replaceChildren(); outlineLinks.clear(); activeOutline = null;
  const large = outlineHeadings.length > 400;
  $('outline-tools').hidden = !large;
  const matching = outlineQuery
    ? outlineHeadings.filter(heading => heading.text.toLocaleLowerCase().includes(outlineQuery))
    : outlineHeadings;
  const page = large ? matching.slice(outlineStart, outlineStart + outlinePageSize) : matching;
  const fragment = document.createDocumentFragment();
  for (const heading of page) {
    const link = document.createElement('a'); link.href = `#${heading.id}`; link.textContent = heading.text;
    link.dataset.heading = heading.id; link.style.paddingLeft = `${10 + (heading.level - 1) * 10}px`;
    link.onclick = event => { event.preventDefault(); jumpToHeading(heading.id); };
    if (heading.id === activeHeadingId) {
      link.classList.add('active'); link.setAttribute('aria-current', 'location'); activeOutline = link;
    }
    fragment.append(link); outlineLinks.set(heading.id, link);
  }
  if (!page.length) {
    const empty = document.createElement('p'); empty.className = 'muted';
    empty.textContent = outlineHeadings.length ? 'No matching headings.' : 'No headings in this document.';
    fragment.append(empty);
  }
  outline.append(fragment);
  if (large) {
    $('outline-range').textContent = matching.length
      ? `${outlineStart + 1}–${outlineStart + page.length} of ${matching.length.toLocaleString()}`
      : '0 headings';
    $<HTMLButtonElement>('outline-prev').disabled = outlineStart === 0;
    $<HTMLButtonElement>('outline-next').disabled = outlineStart + outlinePageSize >= matching.length;
  }
}
function headingAt(top: number) {
  if (headingElements.length > 400) {
    const rect = viewport.getBoundingClientRect();
    const x = rect.left + rect.width / 2;
    for (const offset of [0, 40, 100, 200]) {
      const y = Math.min(rect.bottom - 1, top + offset);
      if (y < rect.top || y >= rect.bottom) continue;
      const chunk = document.elementFromPoint(x, y)?.closest<HTMLElement>('.document-chunk');
      if (!chunk || !article.contains(chunk)) continue;
      const data = chunkHeadings.get(chunk);
      if (!data) continue;
      let low = 0; let high = data.headings.length;
      while (low < high) {
        const mid = (low + high) >>> 1;
        if (data.headings[mid].getBoundingClientRect().top <= top) low = mid + 1; else high = mid;
      }
      return data.headings[low - 1] ?? data.previous;
    }
    // A blank gap at the edge of the viewport has no document element.
    // Keep the last active heading until a chunk becomes visible again.
    return activeOutline?.dataset.heading ? documentHeadings.get(activeOutline.dataset.heading) : headingElements[0];
  }
  let low = 0; let high = headingElements.length;
  while (low < high) {
    const mid = (low + high) >>> 1;
    if (headingElements[mid].getBoundingClientRect().top <= top) low = mid + 1; else high = mid;
  }
  return headingElements[low - 1];
}
function context() {
  const top = viewport.getBoundingClientRect().top;
  const heading = headingAt(top + 50);
  return { id: heading?.dataset.headingId, offset: heading ? heading.getBoundingClientRect().top - top : 0, scroll: viewport.scrollTop };
}
function decorateCode(root: ParentNode, payload: Payload) {
  root.querySelectorAll<HTMLPreElement>('pre[data-code]').forEach(pre => {
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
  root.querySelectorAll('table').forEach(table => {
    const wrapper = document.createElement('div'); wrapper.className = 'table-scroll'; wrapper.tabIndex = 0; wrapper.setAttribute('aria-label', 'Scrollable Markdown table');
    table.before(wrapper); wrapper.append(table);
  });
  root.querySelectorAll('img').forEach(img => {
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
async function render(payload: Payload, preserve = false, anchor?: string | null, request = openRequest, saved?: ReadingContext) {
  const started = performance.now();
  const reading = saved ?? (preserve ? context() : null);
  const resetScroll = !preserve && viewport.scrollTop > 0;
  const template = document.createElement('template');
  template.innerHTML = payload.html;
  const content = template.content;
  const templated = performance.now();
  decorateCode(content, payload);
  const decorated = performance.now();
  if (request !== openRequest) return null;
  const nextHeadings = new Map<string, HTMLElement>();
  // Keep canonical Markdown anchors separate from the reader's DOM identifiers.
  for (const heading of content.querySelectorAll<HTMLElement>('h1[id],h2[id],h3[id],h4[id],h5[id],h6[id]')) {
    const id = heading.id;
    heading.dataset.headingId = id;
    heading.id = `markdown-heading-${id}`;
    nextHeadings.set(id, heading);
  }
  let body = content;
  const nextChunkHeadings = new WeakMap<HTMLElement, ChunkHeadings>();
  if (content.childElementCount > 400) {
    body = document.createDocumentFragment();
    let previous: HTMLElement | undefined;
    while (content.firstChild) {
      const chunk = document.createElement('div'); chunk.className = 'document-chunk';
      const headings: HTMLElement[] = [];
      for (let count = 0; count < 200 && content.firstChild; count++) {
        const node = content.firstChild;
        chunk.append(node);
        if (node instanceof HTMLElement && node.matches('h1,h2,h3,h4,h5,h6')) headings.push(node);
      }
      nextChunkHeadings.set(chunk, { headings, previous });
      previous = headings.at(-1) ?? previous;
      body.append(chunk);
      await yieldToUi();
      if (request !== openRequest) return null;
    }
  }
  const chunked = performance.now();
  current = payload; article.replaceChildren(body);
  chunkHeadings = nextChunkHeadings;
  const attached = performance.now();
  documentHeadings.clear();
  nextHeadings.forEach((heading, id) => documentHeadings.set(id, heading));
  headingElements = Array.from(documentHeadings.values());
  article.hidden = false; $('welcome').hidden = true;
  if (!preserve) viewport.focus({ preventScroll: true });
  $('file-name').textContent = payload.name; $('file-name').title = payload.path;
  document.title = `${payload.name} — Marklight`;
  $('status').textContent = `${payload.metadata.words.toLocaleString()} WORDS · ${payload.metadata.reading_minutes} MIN READ · LIVE RELOAD`;
  outlineHeadings = payload.headings;
  outlineIndexes.clear(); payload.headings.forEach((heading, index) => outlineIndexes.set(heading.id, index));
  outlineQuery = ''; outlineStart = 0; activeHeadingId = undefined;
  $<HTMLInputElement>('outline-filter').value = '';
  paintOutline();
  const outlined = performance.now();
  recents(payload.recent);
  const recented = performance.now();
  if (reading) restoreReading(reading, false);
  else if (resetScroll) viewport.scrollTop = 0;
  const scrolled = performance.now();
  if (!$('search-bar').hidden) scheduleSearch($<HTMLInputElement>('search-input').value, false);
  if (anchor) scrollToHeading(anchor);
  updateProgress();
  const progressed = performance.now();
  if (payload.warning) notify(payload.warning, true);
  const finished = performance.now();
  return {
    ipc_ms: 0, template_ms: templated - started,
    decoration_ms: decorated - templated, chunks_ms: chunked - decorated,
    attach_ms: attached - chunked, outline_ms: outlined - attached,
    recents_ms: recented - outlined, scroll_ms: scrolled - recented,
    progress_ms: progressed - scrolled, finish_ms: finished - outlined, total_ms: finished - started,
  } satisfies RenderTimings;
}
function open(path: string, anchor?: string | null, mode: HistoryMode = 'push', target?: Visit) {
  const request = ++openRequest;
  $('status').textContent = `OPENING ${path.split(/[/\\]/).at(-1) ?? path}…`;
  viewport.setAttribute('aria-busy', 'true');
  enqueue(async () => {
    if (request !== openRequest) return;
    try {
      const invoked = performance.now();
      const payload = await invoke<Payload>('open_document', { path, dark: dark() });
      const received = performance.now();
      if (request === openRequest) {
        const previous = current && { path: current.path, reading: context() };
        const timings = await render(payload, false, anchor, request, target?.reading);
        if (timings) {
          clearReaderError();
          if (!startupTimings) startupTimings = { ...timings, ipc_ms: received - invoked };
          if (mode === 'back') { historyBack.pop(); if (previous) remember(historyForward, previous); }
          else if (mode === 'forward') { historyForward.pop(); if (previous) remember(historyBack, previous); }
          else if (previous) { remember(historyBack, previous); historyForward.length = 0; }
          updateHistoryControls();
        }
      }
    } catch (error) {
      if (request === openRequest) {
        const name = path.split(/[/\\]/).at(-1) ?? path;
        showReaderError(`Could not open ${name}. ${String(error)}`, () => open(path, anchor, mode, target));
      }
    } finally {
      if (request === openRequest) {
        viewport.removeAttribute('aria-busy');
        if (current) $('status').textContent = `${current.metadata.words.toLocaleString()} WORDS · ${current.metadata.reading_minutes} MIN READ · LIVE RELOAD`;
        else $('status').textContent = 'LOCAL FILES. QUIET READING.';
        updateProgress();
      }
    }
  });
}
function navigateHistory(mode: 'back' | 'forward') {
  if (viewport.getAttribute('aria-busy') === 'true') return;
  const source = mode === 'back' ? historyBack : historyForward;
  const destination = mode === 'back' ? historyForward : historyBack;
  const target = source.at(-1);
  if (!target) return;
  if (current?.path === target.path) {
    const previous = { path: current.path, reading: context() };
    restoreReading(target.reading);
    source.pop(); remember(destination, previous); updateHistoryControls();
  } else open(target.path, null, mode, target);
}
async function chooseFile() {
  if (openingDialog) return;
  if (!native) { notify('Run the Tauri desktop app to open local files.'); return; }
  openingDialog = true;
  try { const path = await invoke<string | null>('choose_file'); if (path) open(path); }
  catch (error) { showReaderError(`Could not choose a file. ${String(error)}`, () => { void chooseFile(); }); }
  finally { openingDialog = false; }
}
async function openSample() {
  if (!native) { notify('Run the Tauri desktop app to read the included example.'); return; }
  const request = openRequest;
  const button = $<HTMLButtonElement>('welcome-sample');
  button.disabled = true;
  try {
    const path = await resolveResource('sample/sample.md');
    if (request === openRequest) open(path);
  } catch (error) {
    showReaderError(`Could not open the included example. ${String(error)}`, () => { void openSample(); });
  } finally { button.disabled = false; }
}
function reload() {
  if (!current || !native) return;
  clearTimeout(reloadTimer);
  const request = openRequest;
  const path = current.path;
  reloadTimer = setTimeout(() => enqueue(async () => {
    if (request !== openRequest || current?.path !== path) return;
    try {
      const payload = await invoke<Payload>('reload_document', { dark: dark() });
      if (request === openRequest && current?.path === path) {
        if (await render(payload, true, undefined, request)) clearReaderError();
      }
    } catch (error) {
      if (request === openRequest && current?.path === path) {
        showReaderError(`Could not reload ${current.name}. ${String(error)}`, reload);
      }
    }
  }), 120);
}
function updateProgress() {
  const range = viewport.scrollHeight - viewport.clientHeight;
  const top = viewport.getBoundingClientRect().top;
  const active = headingAt(top + 90)?.dataset.headingId ?? current?.headings[0]?.id;
  const previousActive = activeHeadingId;
  activeHeadingId = active;
  if (!outlineQuery && outlineHeadings.length > 400 && active && active !== previousActive) {
    const index = outlineIndexes.get(active);
    if (index !== undefined && (index < outlineStart || index >= outlineStart + outlinePageSize)) {
      outlineStart = Math.floor(index / outlinePageSize) * outlinePageSize;
      paintOutline();
    }
  }
  const next = active ? outlineLinks.get(active) ?? null : null;
  if (next !== activeOutline) {
    activeOutline?.classList.remove('active'); activeOutline?.removeAttribute('aria-current');
    next?.classList.add('active'); next?.setAttribute('aria-current', 'location'); activeOutline = next;
  }
  $('reading-progress').textContent = current ? `${range > 0 ? Math.round(viewport.scrollTop / range * 100) : 100}%` : '—';
}
function searchRoots() {
  const chunks = Array.from(article.children).filter((child): child is HTMLElement => child instanceof HTMLElement && child.classList.contains('document-chunk'));
  return chunks.length ? chunks : [article];
}
async function runSearch(generation: number, query: string, scroll: boolean) {
  const roots = searchRoots();
  for (const root of roots) {
    if (generation !== searchGeneration) return;
    clearHighlights(root);
    await yieldToUi();
  }
  if (!query || generation !== searchGeneration) {
    if (generation === searchGeneration) { searchPending = false; $('match-count').textContent = '0 matches'; }
    return;
  }
  const found: HTMLElement[][] = [];
  let hasMore = false;
  for (const root of roots) {
    if (generation !== searchGeneration) return;
    const result = highlight(root, query, 10000 - found.length);
    found.push(...result.matches);
    if (result.hasMore) { hasMore = true; break; }
    await yieldToUi();
  }
  if (generation !== searchGeneration) return;
  matches = found; moreMatches = hasMore; matchIndex = -1; searchPending = false;
  moveMatch(1, scroll);
}
function scheduleSearch(query: string, scroll = true, delay = 0) {
  clearTimeout(searchTimer);
  const generation = ++searchGeneration;
  matches = []; moreMatches = false; matchIndex = -1;
  searchPending = Boolean(query);
  $('match-count').textContent = query ? 'Searching…' : '0 matches';
  const start = () => {
    searchActions = searchActions.then(() => runSearch(generation, query, scroll))
      .catch(error => notify(String(error), true));
  };
  if (delay) searchTimer = setTimeout(start, delay); else start();
}
function openSearch() {
  $('search-bar').hidden = false;
  const input = $<HTMLInputElement>('search-input'); input.focus(); input.select();
  if (input.value) scheduleSearch(input.value, false);
}
function closeSearch() { $('search-bar').hidden = true; scheduleSearch('', false); viewport.focus(); }
function moveMatch(direction: number, scroll = true) {
  if (searchPending) return;
  if (matchIndex >= 0) matches[matchIndex]?.forEach(mark => mark.classList.remove('current-match'));
  if (!matches.length) { $('match-count').textContent = '0 matches'; return; }
  matchIndex = (matchIndex + direction + matches.length) % matches.length;
  matches[matchIndex].forEach(mark => mark.classList.add('current-match'));
  $('match-count').textContent = `${matchIndex + 1} / ${matches.length}${moreMatches ? '+' : ''}`;
  if (scroll) matches[matchIndex][0]?.scrollIntoView({ block: 'center' });
}
function toggleToc() {
  if (mobile.matches) {
    const keyboard = document.activeElement === $('toggle-toc');
    const opened = document.body.classList.toggle('mobile-outline');
    config.toc = true; savePreferences();
    if (keyboard) {
      if (opened) ($('outline').querySelector('a') ?? $('recent').querySelector('button'))?.focus();
      else $('toggle-toc').focus();
    }
  } else { config.toc = !config.toc; savePreferences(); }
}
function toggleZen() {
  const focused = document.activeElement;
  config.zen_mode = !config.zen_mode; savePreferences();
  if (config.zen_mode && focused instanceof Element && focused.closest('.toolbar, aside, footer')) viewport.focus();
  else if (!config.zen_mode && focused === $('leave-zen')) $('toggle-zen').focus();
}
function font(delta: number) { config.font_size = delta === 0 ? 16 : Math.max(12, Math.min(28, config.font_size + delta)); savePreferences(); }
$('open').onclick = chooseFile; $('welcome-open').onclick = chooseFile;
$('welcome-sample').onclick = () => { void openSample(); };
$('reader-retry').onclick = () => errorRetry?.();
$('reader-error-close').onclick = clearReaderError;
$('history-back').onclick = () => navigateHistory('back');
$('history-forward').onclick = () => navigateHistory('forward');
$<HTMLInputElement>('outline-filter').oninput = event => {
  outlineQuery = (event.target as HTMLInputElement).value.trim().toLocaleLowerCase();
  const activeIndex = activeHeadingId ? outlineIndexes.get(activeHeadingId) : undefined;
  outlineStart = outlineQuery || activeIndex === undefined ? 0 : Math.floor(activeIndex / outlinePageSize) * outlinePageSize;
  paintOutline(); $('outline').scrollTop = 0;
};
$('outline-prev').onclick = () => { outlineStart = Math.max(0, outlineStart - outlinePageSize); paintOutline(); $('outline').scrollTop = 0; };
$('outline-next').onclick = () => { outlineStart += outlinePageSize; paintOutline(); $('outline').scrollTop = 0; };
$('toggle-toc').onclick = toggleToc; $('toggle-zen').onclick = toggleZen; $('leave-zen').onclick = toggleZen;
$('toggle-search').onclick = openSearch; $('close-search').onclick = closeSearch;
$('next-match').onclick = () => moveMatch(1); $('previous-match').onclick = () => moveMatch(-1);
$('font-up').onclick = () => font(1); $('font-down').onclick = () => font(-1); $('font-reset').onclick = () => font(0);
$<HTMLSelectElement>('theme').onchange = event => { config.theme = (event.target as HTMLSelectElement).value as Theme; savePreferences(); reload(); };
$<HTMLInputElement>('search-input').oninput = event => scheduleSearch((event.target as HTMLInputElement).value, true, 100);
$('clear-recent').onclick = () => { enqueue(async () => { if (native) await invoke('clear_recent'); recents([]); }); };
viewport.onscroll = updateProgress;
systemDark.onchange = () => { if (config.theme === 'system') { applyPreferences(); reload(); } };
function followLink(href: string) {
  enqueue(async () => {
    try {
      const navigation = await invoke<Navigation>('follow_link', { href });
      if (navigation.kind === 'anchor') { jumpToHeading(navigation.id); clearReaderError(); }
      if (navigation.kind === 'markdown') open(navigation.path, navigation.anchor);
      if (navigation.kind === 'external') clearReaderError();
    } catch (error) {
      showReaderError(`Could not follow this link. ${String(error)}`, () => followLink(href));
    }
  });
}
article.onclick = event => {
  const link = (event.target as Element).closest('a'); if (!link) return;
  event.preventDefault(); const href = link.getAttribute('href'); if (!href) return;
  if (viewport.getAttribute('aria-busy') === 'true') return;
  followLink(href);
};
document.querySelector('.wordmark')!.addEventListener('click', event => { event.preventDefault(); viewport.scrollTop = 0; });
document.addEventListener('keydown', event => {
  const mod = event.metaKey || event.ctrlKey;
  if (event.altKey && !mod && event.key === 'ArrowLeft') { event.preventDefault(); navigateHistory('back'); }
  else if (event.altKey && !mod && event.key === 'ArrowRight') { event.preventDefault(); navigateHistory('forward'); }
  else if (mod && event.key.toLowerCase() === 'o') { event.preventDefault(); void chooseFile(); }
  else if (mod && event.key.toLowerCase() === 'f') { event.preventDefault(); openSearch(); }
  else if (mod && event.shiftKey && event.key.toLowerCase() === 't') { event.preventDefault(); toggleToc(); }
  else if (mod && event.shiftKey && event.key.toLowerCase() === 'z') { event.preventDefault(); toggleZen(); }
  else if (mod && ['+', '=', '-', '0'].includes(event.key)) { event.preventDefault(); font(event.key === '0' ? 0 : event.key === '-' ? -1 : 1); }
  else if (event.key === 'Escape') {
    if (!$('search-bar').hidden) closeSearch();
    else if (config.zen_mode) toggleZen();
    else if (document.body.classList.contains('mobile-outline')) { document.body.classList.remove('mobile-outline'); $('toggle-toc').focus(); }
  }
  else if (event.key === 'Enter' && event.target === $('search-input')) { event.preventDefault(); moveMatch(event.shiftKey ? -1 : 1); }
});
async function startup() {
  applyPreferences();
  if (!native) return;
  const pending = async () => { const path = await invoke<string | null>('take_pending'); if (path) open(path); };
  await listen('open-request', () => { void pending().catch(error => notify(String(error), true)); });
  await listen('menu-open', () => { void chooseFile(); });
  await listen<string>('reader-action', event => {
    if (event.payload === 'find') openSearch();
    else if (event.payload === 'outline') toggleToc();
    else if (event.payload === 'zen') toggleZen();
    else if (event.payload === 'larger') font(1);
    else if (event.payload === 'smaller') font(-1);
    else if (event.payload === 'reset') font(0);
  });
  await listen('document-changed', reload);
  await getCurrentWebviewWindow().onDragDropEvent(event => {
    $('drop-overlay').hidden = event.payload.type === 'leave' || event.payload.type === 'drop';
    if (event.payload.type === 'drop' && event.payload.paths[0]) open(event.payload.paths[0]);
  });
  config = await invoke<Config>('get_config'); applyPreferences(); recents(config.recent); await pending();
  await actions;
  // render/updateProgress already forces initial layout. Report DOM readiness;
  // background WebViews can suspend animation frames indefinitely.
  await invoke('reader_ready', { timings: startupTimings });
}
void startup().catch(error => notify(String(error), true));
