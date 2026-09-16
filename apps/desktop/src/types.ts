export type Theme = 'system' | 'light' | 'dark';
export interface Config { theme: Theme; font_size: number; toc: boolean; zen_mode: boolean; recent: string[] }
export interface Heading { level: number; text: string; id: string }
export interface Payload {
  path: string; name: string; html: string; headings: Heading[];
  code_blocks: { language: string; code: string }[];
  metadata: { bytes: number; words: number; reading_minutes: number; title: string };
  recent: string[]; warning: string | null;
}
export type Navigation = { kind: 'anchor'; id: string } | { kind: 'external'; url: string } | { kind: 'markdown'; path: string; anchor: string | null };
