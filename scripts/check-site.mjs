#!/usr/bin/env node
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createRequire } from 'node:module';
import { mkdir } from 'node:fs/promises';
const require = createRequire(new URL('../apps/desktop/package.json', import.meta.url));
const { chromium } = require('@playwright/test');
const root = new URL('../', import.meta.url).pathname;
const base = 'http://127.0.0.1:8787/site/';
const server = spawn('python3', ['-m','http.server','8787','--bind','127.0.0.1','--directory',root], { stdio: 'ignore' });
let browser;
try {
  for (let i=0; i<50; i++) {
    try { if ((await fetch(base)).ok) break; } catch {}
    if (i===49) throw new Error('Local site server did not start');
    await new Promise(resolve => setTimeout(resolve,100));
  }
  browser = await chromium.launch({ executablePath: process.env.MARKLIGHT_CHROMIUM, headless: true });
  await mkdir(`${root}artifacts/site-check`, { recursive: true });
  let copied = 0; let checked = 0;
  for (const width of [390,1280]) {
    const context = await browser.newContext({ viewport: { width, height: 900 }, permissions: ['clipboard-read','clipboard-write'] });
    for (const file of ['index.html','docs.html']) {
      const page = await context.newPage(); const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      page.on('console', message => { if (message.type()==='error') errors.push(message.text()); });
      await page.goto(base+file, { waitUntil: 'networkidle' });
      assert(await page.evaluate(() => document.documentElement.scrollWidth === innerWidth), `${file}: overflow at ${width}`);
      await page.evaluate(async () => { await Promise.all([...document.images].map(img => { img.loading='eager'; return img.decode(); })); });
      assert(await page.evaluate(() => [...document.images].every(img => img.complete && img.naturalWidth > 0)), 'broken image');
      if (width===390) {
        const snippets = page.locator('.code-snippet');
        for (let i=0; i<await snippets.count(); i++) {
          const snippet = snippets.nth(i); const expected = await snippet.locator('code').textContent();
          await snippet.locator('.copy-button').click();
          await page.waitForFunction(() => document.querySelector('.copy-button') !== null);
          await page.waitForFunction(async expected => (await navigator.clipboard.readText()) === expected, expected);
          assert.equal(await page.evaluate(() => navigator.clipboard.readText()),expected); copied++;
        }
      }
      if (file==='index.html') {
        await page.locator('[data-preview="dark"]').click();
        await page.locator('#reader-preview[src="images/reader-dark.png"]').waitFor();
        await page.locator('[data-preview="light"]').click();
      }
      const links = await page.evaluate(async () => {
        const issues=[]; let count=0;
        for (const el of document.querySelectorAll('a[href],img[src],link[rel="stylesheet"]')) {
          const url=new URL(el.getAttribute('href') ?? el.getAttribute('src'),location.href);
          if (url.origin!==location.origin) continue;
          const response=await fetch(url); count++;
          if (!response.ok) { issues.push(`${url.pathname}: ${response.status}`); continue; }
          if (url.hash) {
            const doc=new DOMParser().parseFromString(await response.text(),'text/html');
            if (!doc.getElementById(decodeURIComponent(url.hash.slice(1)))) issues.push(`Missing anchor ${url.hash}`);
          }
        }
        return {issues,count};
      });
      assert.deepEqual(links.issues,[]); checked+=links.count;
      await page.evaluate(() => scrollTo(0,0));
      await page.screenshot({ path: `${root}artifacts/site-check/${file.replace('.html','')}-${width}.png`, fullPage: true });
      assert.deepEqual(errors,[]); await page.close();
    }
    await context.close();
  }
  const nojs=await browser.newContext({javaScriptEnabled:false,viewport:{width:390,height:900}});
  for (const file of ['index.html','docs.html']) {
    const page=await nojs.newPage(); await page.goto(base+file);
    assert(await page.evaluate(() => document.documentElement.scrollWidth === innerWidth)); await page.close();
  }
  await nojs.close();
  console.log(`PASS: 390/1280 layouts, ${copied} exact snippet copies, ${checked} local references, themes, images, no console errors, JS-off mobile`);
} finally { if (browser) await browser.close(); server.kill('SIGTERM'); }
