/** Search rendered text, including phrases spanning inline styles. No innerHTML. */
export function clearHighlights(root: HTMLElement): void {
  const parents = new Set<HTMLElement>();
  root.querySelectorAll('mark[data-match]').forEach(mark => {
    if (mark.parentElement) parents.add(mark.parentElement);
  });
  for (const parent of parents) {
    const fragment = document.createDocumentFragment();
    let text = '';
    const flush = () => { if (text) { fragment.append(document.createTextNode(text)); text = ''; } };
    for (const node of Array.from(parent.childNodes)) {
      if (node instanceof Text) text += node.data;
      else if (node instanceof HTMLElement && node.matches('mark[data-match]')) text += node.textContent ?? '';
      else { flush(); fragment.append(node); }
    }
    flush(); parent.replaceChildren(fragment);
  }
}
export function highlight(root: HTMLElement, query: string, limit = 10000): { matches: HTMLElement[][]; hasMore: boolean } {
  if (!query) return { matches: [], hasMore: false };
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
  let hasMore = false;
  const pattern = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'giu');
  for (const list of groups.values()) {
    const text = list.map(item => item.node.data).join('');
    const spans: { start: number; end: number; index: number }[] = [];
    for (const match of text.matchAll(pattern)) {
      // Avoid creating unbounded DOM for pathological queries in huge documents.
      if (matches.length >= limit) { hasMore = true; break; }
      const index = matches.length;
      matches.push([]);
      spans.push({ start: match.index, end: match.index + match[0].length, index });
    }
    let spanIndex = 0;
    for (const item of list) {
      while (spanIndex < spans.length && spans[spanIndex].end <= item.start) spanIndex++;
      const fragment = document.createDocumentFragment();
      let cursor = 0;
      let index = spanIndex;
      while (index < spans.length && spans[index].start < item.end) {
        const span = spans[index];
        const start = Math.max(span.start, item.start) - item.start;
        const end = Math.min(span.end, item.end) - item.start;
        if (end > start) {
          if (start > cursor) fragment.append(document.createTextNode(item.node.data.slice(cursor, start)));
          const mark = document.createElement('mark'); mark.dataset.match = String(span.index);
          mark.textContent = item.node.data.slice(start, end);
          fragment.append(mark); matches[span.index].push(mark); cursor = end;
        }
        if (span.end > item.end) break;
        index++;
      }
      spanIndex = index;
      if (fragment.childNodes.length) {
        if (cursor < item.node.data.length) fragment.append(document.createTextNode(item.node.data.slice(cursor)));
        item.node.replaceWith(fragment);
      }
    }
    if (hasMore) break;
  }
  return { matches, hasMore };
}
