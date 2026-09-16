/** Search rendered text, including phrases spanning inline styles. No innerHTML. */
export function clearHighlights(root: HTMLElement): void {
  root.querySelectorAll('mark[data-match]').forEach(mark => mark.replaceWith(...mark.childNodes));
  root.normalize();
}
export function highlight(root: HTMLElement, query: string): HTMLElement[][] {
  clearHighlights(root);
  if (!query) return [];
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode: node => node.parentElement?.closest('.code-toolbar,button,.image-note') ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT,
  });
  const groups = new Map<Element, { node: Text; start: number; end: number }[]>();
  let node: Node | null;
  while ((node = walker.nextNode())) {
    const block = node.parentElement?.closest('p,h1,h2,h3,h4,h5,h6,li,pre,th,td,summary') ?? root;
    const list = groups.get(block) ?? [];
    const start = list.at(-1)?.end ?? 0;
    list.push({ node: node as Text, start, end: start + (node.textContent?.length ?? 0) });
    groups.set(block, list);
  }
  const matches: HTMLElement[][] = [];
  const pattern = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'giu');
  for (const list of groups.values()) {
    const text = list.map(item => item.node.data).join('');
    const spans: { start: number; end: number; index: number }[] = [];
    for (const match of text.matchAll(pattern)) {
      // Avoid creating unbounded DOM for pathological queries in huge documents.
      if (matches.length >= 10000) break;
      const index = matches.length;
      matches.push([]);
      spans.push({ start: match.index, end: match.index + match[0].length, index });
    }
    for (const item of list) {
      for (const span of spans.slice().reverse()) {
        const start = Math.max(span.start, item.start) - item.start;
        const end = Math.min(span.end, item.end) - item.start;
        if (end <= start) continue;
        const range = document.createRange();
        range.setStart(item.node, start); range.setEnd(item.node, end);
        const mark = document.createElement('mark');
        mark.dataset.match = String(span.index);
        range.surroundContents(mark); matches[span.index].unshift(mark);
      }
    }
  }
  return matches;
}
