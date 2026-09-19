import { test, expect, type Page } from '@playwright/test';
import { readFileSync, mkdirSync } from 'node:fs';
const fixtures = Object.fromEntries(['gfm','basic','links','broken-link','alerts','huge','malicious-html','unicode','tables','code','ui-collisions'].map(name => [name, JSON.parse(readFileSync(new URL(`../../../artifacts/frontend-fixtures/${name}.json`, import.meta.url), 'utf8'))]));
async function reader(page: Page, initial = 'gfm', recentNames: string[] = []) {
  await page.addInitScript(({ fixtures, initial, recentNames }) => {
    const w = window as any;
    w.isTauri = true;
    let id = 0;
    const callbacks = new Map<number, Function>();
    const events = new Map<string, number[]>();
    let active = initial;
    let pending: string | null = fixtures[initial].path;
    const config = { theme: 'system', font_size: 16, toc: true, zen_mode: false, recent: recentNames.map(name => fixtures[name].path) as string[] };
    w.testCommands = [];
    w.testFailOpenPath = null;
    w.testReloadError = false;
    w.testReloadDelay = 0;
    w.testMissingLink = true;
    w.testEmit = (event: string, payload: unknown = null) => (events.get(event) ?? []).forEach(handler => callbacks.get(handler)?.({ event, payload }));
    w.testReload = null;
    w.__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
      convertFileSrc: (path: string, scheme: string) => `${scheme}://localhost/${path}`,
      transformCallback: (callback: Function) => { callbacks.set(++id, callback); return id; },
      unregisterCallback: (id: number) => callbacks.delete(id),
      invoke: async (command: string, args: any = {}) => {
        w.testCommands.push({ command, args });
        if (command === 'copy_text') return navigator.clipboard.writeText(args.text);
        if (command === 'plugin:event|listen') { const list = events.get(args.event) ?? []; list.push(args.handler); events.set(args.event, list); return args.handler; }
        if (command === 'get_config') return config;
        if (command === 'take_pending') { const result = pending; pending = null; return result; }
        if (command === 'save_preferences') { Object.assign(config, { theme: args.theme, font_size: args.fontSize, toc: args.toc, zen_mode: args.zenMode }); return; }
        if (command === 'clear_recent') { config.recent = []; return; }
        if (command === 'choose_file') return fixtures.basic.path;
        if (command === 'open_document') {
          if (w.testOpenDelay) await new Promise(resolve => setTimeout(resolve, w.testOpenDelay));
          if (w.testFailOpenPath === args.path) throw new Error('File unavailable');
          active = Object.keys(fixtures).find(name => fixtures[name].path === args.path) ?? '';
          if (!active) throw new Error('File not found');
          config.recent = [args.path, ...config.recent.filter(path => path !== args.path)].slice(0, 12);
          return { ...fixtures[active], html: args.dark ? fixtures[active].html_dark : fixtures[active].html, recent: config.recent };
        }
        if (command === 'reload_document') {
          if (w.testReloadDelay) await new Promise(resolve => setTimeout(resolve, w.testReloadDelay));
          if (w.testReloadError) throw new Error('File was removed');
          return { ...(w.testReload ?? fixtures[active]), html: w.testReload?.html ?? (args.dark ? fixtures[active].html_dark : fixtures[active].html), recent: config.recent };
        }
        if (command === 'follow_link') {
          if (args.href.startsWith('#')) return { kind: 'anchor', id: args.href.slice(1) };
          if (args.href === 'basic.md') return { kind: 'markdown', path: fixtures.basic.path, anchor: null };
          if (args.href === 'not-here.md') {
            if (w.testMissingLink) throw new Error('No such file');
            return { kind: 'markdown', path: fixtures.basic.path, anchor: null };
          }
          if (args.href.startsWith('https:')) return { kind: 'external', url: args.href };
          throw new Error('Unsafe link');
        }
      },
    };
    w.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  }, { fixtures, initial, recentNames });
  await page.goto('/');
  await expect(page.locator('article#document')).toBeVisible();
}

test('heading names cannot overwrite controls and anchors target the document', async ({ page }) => {
  await reader(page, 'ui-collisions');
  const article = page.locator('article#document');
  await expect(article.locator('h1')).toHaveText('Document');
  await expect(article.locator('h2').first()).toHaveText('Status');
  await expect(page.locator('footer #status')).toContainText('WORDS');
  await page.getByRole('button', { name: 'Increase font size' }).click();
  await expect(page.locator('footer #font-reset')).toHaveText('17px');
  await expect(article.locator('h2').nth(1)).toHaveText('Font reset');
  await page.locator('#outline a[data-heading="sidebar"]').click();
  await expect(article.locator('h2').last()).toBeFocused();
  await article.getByRole('link', { name: 'Return to Document' }).click();
  await expect(article.locator('h1')).toBeFocused();
  await article.getByRole('link', { name: 'Return to Status' }).click();
  await expect(article.locator('h2').first()).toBeFocused();
  expect(await page.evaluate(() => {
    const ids = Array.from(document.querySelectorAll('[id]'), el => el.id);
    return new Set(ids).size === ids.length;
  })).toBe(true);
});

test('recent files open chunked documents and a newer click wins during loading', async ({ page }) => {
  await reader(page, 'gfm', ['huge', 'basic']);
  await page.locator('#recent button').filter({ hasText: 'huge.md' }).click();
  await expect(page.locator('#file-name')).toHaveText('huge.md');
  expect(await page.locator('#document .document-chunk').count()).toBeGreaterThan(1);
  await page.locator('#document [data-heading-id="section-400"]').evaluate(element => element.scrollIntoView());
  await expect.poll(() => page.locator('#viewport').evaluate(element => element.scrollTop)).toBeGreaterThan(0);
  await page.locator('#recent button').filter({ hasText: 'basic.md' }).click();
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await expect.poll(() => page.locator('#viewport').evaluate(element => element.scrollTop)).toBe(0);
  await expect(page.locator('#document h1')).toHaveText('Marklight');
  await page.evaluate(() => { (window as any).testOpenDelay = 250; });
  await page.locator('#recent button').filter({ hasText: 'huge.md' }).click();
  await expect(page.locator('footer #status')).toHaveText('OPENING huge.md…');
  await expect(page.locator('#viewport')).toHaveAttribute('aria-busy', 'true');
  await page.locator('#recent button').filter({ hasText: 'basic.md' }).click();
  await expect(page.locator('#viewport')).not.toHaveAttribute('aria-busy', 'true');
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await expect(page.locator('#document h1')).toHaveText('Marklight');
  await expect(page.locator('footer #status')).toContainText('WORDS');
});

test('actual Rust HTML renders and exact original code copies; normal text selects', async ({ page, context }) => {
  const errors: string[] = []; page.on('pageerror', error => errors.push(error.message));
  await context.grantPermissions(['clipboard-read','clipboard-write']);
  await reader(page);
  await expect(page.locator('#document h1')).toHaveText('Marklight');
  await expect(page.locator('#document strong').first()).toHaveText('fast');
  await expect(page.locator('#document em')).toHaveText('quiet');
  await expect(page.locator('#document del')).toHaveText('Noise');
  await expect(page.locator('#document input:checked')).toHaveCount(1);
  await expect(page.locator('#document table')).toHaveCount(1);
  await page.getByRole('button', { name: 'Copy rust code' }).click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(fixtures.gfm.code_blocks[0].code);
  const selected = await page.evaluate(() => {
    const range = document.createRange();
    range.setStartBefore(document.querySelector('#document p')!);
    range.setEndAfter(document.querySelector('#document blockquote')!);
    const selection = window.getSelection()!; selection.removeAllRanges(); selection.addRange(range); return selection.toString();
  });
  expect(selected).toContain('fast and quiet'); expect(selected).toContain('pleasant to read');
  expect(errors).toEqual([]);
});

test('search spans styled text, counts, cycles and closes from keyboard', async ({ page }) => {
  await reader(page); await page.keyboard.press('Control+f');
  await page.locator('#search-input').fill('fast and quiet');
  await expect(page.locator('#match-count')).toHaveText('1 / 1');
  await expect(page.locator('#document mark')).toHaveCount(3);
  await page.locator('#search-input').fill('Marklight');
  await expect(page.locator('#match-count')).toHaveText('1 / 2');
  await page.keyboard.press('Enter'); await expect(page.locator('#match-count')).toHaveText('2 / 2');
  await page.keyboard.press('Shift+Enter'); await expect(page.locator('#match-count')).toHaveText('1 / 2');
  await page.keyboard.press('Escape'); await expect(page.locator('#search-bar')).toBeHidden();
  await expect(page.locator('#document mark')).toHaveCount(0);
});

test('large-document search can be replaced and reopened without stale highlights', async ({ page }) => {
  await reader(page, 'huge', ['basic']);
  await page.getByRole('button', { name: 'Search document' }).click();
  const input = page.locator('#search-input');
  await input.fill('Section');
  await expect(page.locator('#match-count')).toHaveText('1 / 800');
  await input.fill('Section 700');
  await expect(page.locator('#match-count')).toHaveText('1 / 1');
  await expect(page.locator('#document mark')).toHaveCount(1);
  await page.getByRole('button', { name: 'Close search' }).click();
  await expect(page.locator('#document mark')).toHaveCount(0);
  await page.getByRole('button', { name: 'Search document' }).click();
  await expect(page.locator('#match-count')).toHaveText('1 / 1');
  await page.locator('#recent button').filter({ hasText: 'basic.md' }).click();
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await expect(page.locator('#match-count')).toHaveText('0 matches');
  await expect(page.locator('#document mark')).toHaveCount(0);
});

test('search reports additional matches only when the display cap is exceeded', async ({ page }) => {
  test.setTimeout(60000);
  await reader(page);
  await page.evaluate(payload => {
    const w = window as any;
    w.testReload = { ...payload, html: `<p>${'needle '.repeat(10001)}</p>` };
    w.testEmit('document-changed');
  }, fixtures.gfm);
  await expect(page.locator('#document p').first()).toContainText('needle needle');
  await page.getByRole('button', { name: 'Search document' }).click();
  await page.locator('#search-input').fill('needle');
  await expect(page.locator('#match-count')).toHaveText('1 / 10000+', { timeout: 30000 });
  await expect(page.locator('#document mark')).toHaveCount(10000);
});

test('font, theme, outline, zen and narrow layout stay usable', async ({ page }) => {
  await reader(page); await page.keyboard.press('Control+=');
  await expect(page.locator('#font-reset')).toHaveText('17px');
  await page.keyboard.press('Control+0'); await expect(page.locator('#font-reset')).toHaveText('16px');
  await page.keyboard.press('Control+Shift+t'); await expect(page.locator('#sidebar')).toBeHidden();
  await page.keyboard.press('Control+Shift+t'); await expect(page.locator('#sidebar')).toBeVisible();
  await page.selectOption('#theme','dark'); await expect(page.locator('html')).toHaveAttribute('data-theme','dark');
  await page.keyboard.press('Control+Shift+z'); await expect(page.locator('.toolbar')).toBeHidden();
  await expect(page.locator('#document')).toBeVisible(); await page.keyboard.press('Escape');
  await expect(page.locator('.toolbar')).toBeVisible();
  await page.setViewportSize({ width: 400, height: 760 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth === innerWidth)).toBe(true);
  expect(await page.locator('#viewport').evaluate(el => el.scrollWidth === el.clientWidth)).toBe(true);
  await page.locator('#toggle-toc').click(); await expect(page.locator('#sidebar')).toBeVisible();
  await page.locator('#outline a').last().click(); await expect(page.locator('#sidebar')).toBeHidden();
});

test('outline navigation and reload preserve the active heading context', async ({ page }) => {
  await reader(page,'huge');
  await page.getByRole('searchbox', { name: 'Find a heading' }).fill('Section 400');
  await page.locator('#outline a[data-heading="section-400"]').click();
  const heading = page.locator('#document [data-heading-id="section-400"]');
  const before = await heading.evaluate(el => el.getBoundingClientRect().top - document.getElementById('viewport')!.getBoundingClientRect().top);
  await page.evaluate(payload => {
    const w = window as any;
    w.testReload = { ...payload, html: '<p>New material above your reading position.</p>'.repeat(40) + payload.html,
      metadata: { words: 12000, reading_minutes: 55 } };
    w.testEmit('document-changed');
  }, fixtures.huge);
  await expect(page.locator('#document p').first()).toHaveText('New material above your reading position.');
  const after = await heading.evaluate(el => el.getBoundingClientRect().top - document.getElementById('viewport')!.getBoundingClientRect().top);
  expect(Math.abs(after - before)).toBeLessThan(3);
  expect(await page.locator('#viewport').evaluate(el => el.scrollTop)).toBeGreaterThan(500);
});

test('reading history restores a file, heading offset and forward visit', async ({ page }) => {
  await reader(page, 'huge', ['basic']);
  const back = page.getByRole('button', { name: 'Back in reading history' });
  const forward = page.getByRole('button', { name: 'Forward in reading history' });
  await expect(back).toBeDisabled();
  await page.getByRole('searchbox', { name: 'Find a heading' }).fill('Section 400');
  await page.locator('#outline a[data-heading="section-400"]').click();
  const heading = page.locator('#document [data-heading-id="section-400"]');
  const offset = await heading.evaluate(element => element.getBoundingClientRect().top - document.getElementById('viewport')!.getBoundingClientRect().top);
  await page.locator('#recent button').filter({ hasText: 'basic.md' }).click();
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await back.click();
  await expect(page.locator('#file-name')).toHaveText('huge.md');
  const restored = await heading.evaluate(element => element.getBoundingClientRect().top - document.getElementById('viewport')!.getBoundingClientRect().top);
  expect(Math.abs(restored - offset)).toBeLessThan(3);
  await back.click();
  await expect.poll(() => page.locator('#viewport').evaluate(element => element.scrollTop)).toBe(0);
  await forward.click();
  await expect.poll(() => page.locator('#viewport').evaluate(element => element.scrollTop)).toBeGreaterThan(500);
  await page.keyboard.press('Alt+ArrowRight');
  await expect(page.locator('#file-name')).toHaveText('basic.md');
});

test('failed open and reload retain the document with a persistent retry', async ({ page }) => {
  await reader(page, 'gfm', ['basic']);
  await page.evaluate(path => { (window as any).testFailOpenPath = path; }, fixtures.basic.path);
  await page.locator('#recent button').filter({ hasText: 'basic.md' }).click();
  const error = page.locator('#reader-error');
  await expect(error).toContainText('Could not open basic.md');
  await expect(page.locator('#document h1')).toHaveText('Marklight');
  await expect(page.getByRole('button', { name: 'Back in reading history' })).toBeDisabled();
  await page.evaluate(() => { (window as any).testFailOpenPath = null; });
  await page.getByRole('button', { name: 'Retry' }).click();
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await expect(error).toBeHidden();
  await page.evaluate(() => { (window as any).testReloadError = true; (window as any).testEmit('document-changed'); });
  await expect(error).toContainText('Could not reload basic.md');
  await expect(page.locator('#document h1')).toHaveText('Marklight');
  await page.evaluate(payload => {
    const w = window as any;
    w.testReloadError = false;
    w.testReload = { ...payload, html: '<p>Recovered on disk.</p>' + payload.html };
  }, fixtures.basic);
  await page.getByRole('button', { name: 'Retry' }).click();
  await expect(page.locator('#document p').first()).toHaveText('Recovered on disk.');
  await expect(error).toBeHidden();
});

test('a missing relative link can be retried after its target appears', async ({ page }) => {
  await reader(page, 'broken-link');
  await page.getByRole('link', { name: 'Open missing page' }).click();
  await expect(page.locator('#reader-error')).toContainText('Could not follow this link');
  await expect(page.locator('#document h1')).toHaveText('Missing local page');
  await page.evaluate(() => { (window as any).testMissingLink = false; });
  await page.getByRole('button', { name: 'Retry' }).click();
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await expect(page.locator('#reader-error')).toBeHidden();
});

test('GFM alerts have visible labels and preserve a safe local link', async ({ page }) => {
  const external: string[] = [];
  page.on('request', request => { if (!request.url().startsWith('http://127.0.0.1:1420')) external.push(request.url()); });
  await reader(page, 'alerts');
  for (const [kind, title] of [['note','Note'], ['tip','Tip'], ['important','Important'], ['warning','Warning'], ['caution','Caution']]) {
    await expect(page.locator(`#document blockquote.markdown-alert-${kind} .markdown-alert-title`)).toHaveText(title);
  }
  await expect(page.locator('#document script')).toHaveCount(0);
  expect(external).toEqual([]);
  await page.getByRole('link', { name: 'setup guide' }).click();
  await expect(page.locator('#file-name')).toHaveText('basic.md');
});

test('an older reload response cannot replace a newer file choice', async ({ page }) => {
  await reader(page, 'gfm', ['basic']);
  await page.evaluate(() => { (window as any).testReloadDelay = 250; (window as any).testEmit('document-changed'); });
  await expect.poll(() => page.evaluate(() => (window as any).testCommands.some((call: any) => call.command === 'reload_document'))).toBe(true);
  await page.locator('#recent button').filter({ hasText: 'basic.md' }).click();
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await expect(page.locator('#document h1')).toHaveText('Marklight');
  await expect(page.locator('#reader-error')).toBeHidden();
});

test('scroll progress does not measure headings in offscreen chunks', async ({ page }) => {
  await reader(page, 'huge');
  await expect(page.locator('#outline a')).toHaveCount(100);
  await expect(page.locator('#outline-range')).toHaveText('1–100 of 801');
  await page.getByRole('button', { name: 'Next headings' }).click();
  await expect(page.locator('#outline-range')).toHaveText('101–200 of 801');
  await page.getByRole('searchbox', { name: 'Find a heading' }).fill('Section 700');
  await expect(page.locator('#outline a[data-heading="section-700"]')).toBeVisible();
  const measuredFarHeadings = await page.evaluate(() => {
    const headings = Array.from(document.querySelectorAll('#document [data-heading-id]'));
    const far = new Set(headings.slice(200));
    const original = Element.prototype.getBoundingClientRect;
    let count = 0;
    Element.prototype.getBoundingClientRect = function (...args) {
      if (far.has(this)) count++;
      return original.apply(this, args);
    };
    try { document.getElementById('viewport')!.dispatchEvent(new Event('scroll')); }
    finally { Element.prototype.getBoundingClientRect = original; }
    return count;
  });
  expect(measuredFarHeadings).toBe(0);
  await page.locator('#outline a[data-heading="section-700"]').click();
  await expect(page.locator('#document [data-heading-id="section-700"]')).toBeFocused();
  await expect(page.locator('#outline a[data-heading="section-700"]')).toHaveAttribute('aria-current', 'location');
});

test('native drop/menu wiring, relative Markdown links and recent history', async ({ page }) => {
  await reader(page,'links');
  await page.locator('#document a').filter({ hasText: 'Local' }).click();
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await expect(page.locator('#recent li')).toHaveCount(2);
  await page.locator('#recent button').filter({ hasText: 'links.md' }).click();
  await expect(page.locator('#file-name')).toHaveText('links.md');
  await page.evaluate((path) => (window as any).testEmit('tauri://drag-drop', { paths: [path], position: { x: 0, y: 0 } }),fixtures.unicode.path);
  await expect(page.locator('#file-name')).toHaveText('unicode.md');
  await page.evaluate(() => (window as any).testEmit('menu-open'));
  await expect(page.locator('#file-name')).toHaveText('basic.md');
  await page.evaluate(() => (window as any).testEmit('reader-action', 'larger'));
  await expect(page.locator('#font-reset')).toHaveText('17px');
  await page.evaluate(() => (window as any).testEmit('reader-action', 'reset'));
  await expect(page.locator('#font-reset')).toHaveText('16px');
  await page.evaluate(() => (window as any).testEmit('reader-action', 'zen'));
  await expect(page.locator('.toolbar')).toBeHidden();
  await page.evaluate(() => (window as any).testEmit('reader-action', 'zen'));
  await expect(page.locator('.toolbar')).toBeVisible();
  await page.locator('#clear-recent').click(); await expect(page.locator('#recent li')).toHaveCount(0);
});

test('hostile fixture cannot inject executable HTML or remote image requests', async ({ page }) => {
  const external: string[] = []; page.on('request', request => { if (!request.url().startsWith('http://127.0.0.1:1420')) external.push(request.url()); });
  await reader(page,'malicious-html');
  await expect(page.locator('#document script,#document iframe,#document svg,#document [onerror]')).toHaveCount(0);
  expect(external).toEqual([]);
});

test('capture actual reader UI at desktop/light, desktop/dark and narrow sizes', async ({ page }) => {
  await reader(page); mkdirSync(new URL('../../../docs/images',import.meta.url),{ recursive:true });
  await page.setViewportSize({ width: 1120, height: 850 });
  await page.selectOption('#theme','light');
  await page.screenshot({ path: '../../docs/images/reader-light.png' });
  await page.selectOption('#theme','dark');
  await expect(page.locator('#document pre span').first()).toHaveAttribute('style', /color:#b48ead/);
  await page.screenshot({ path: '../../docs/images/reader-dark.png' });
  await page.setViewportSize({ width: 400, height: 760 });
  await page.screenshot({ path: '../../docs/images/reader-narrow.png' });
});
