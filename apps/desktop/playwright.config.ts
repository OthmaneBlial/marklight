import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests', fullyParallel: true, reporter: 'list',
  outputDir: '../../artifacts/browser-tests',
  use: { baseURL: 'http://127.0.0.1:1420', headless: true, browserName: 'chromium',
    launchOptions: { executablePath: process.env.MARKLIGHT_CHROMIUM } },
  webServer: { command: 'npm run dev', url: 'http://127.0.0.1:1420', reuseExistingServer: false },
});
