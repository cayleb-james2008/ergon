(() => {
  const out = { rules: [], synthetic: null, highlightKeyword: null };
  for (const ss of document.styleSheets) {
    try { for (const r of ss.cssRules) { if (r.selectorText && r.selectorText.includes('data-contrast="high"] .sidebar nav button')) out.rules.push({ sel: r.selectorText, css: r.cssText }); } } catch (e) {}
  }
  const d = document.createElement('div'); d.style.background = 'Highlight'; document.body.append(d);
  out.highlightKeyword = getComputedStyle(d).backgroundColor; d.remove();
  const host = document.createElement('div');
  host.innerHTML = '<div class="sidebar"><nav><button type="button" aria-current="page">Chat</button></nav></div>';
  document.body.append(host);
  document.documentElement.setAttribute('data-contrast', 'high');
  const b = host.querySelector('button');
  out.synthetic = { matches: b.matches('[data-contrast="high"] .sidebar nav button[aria-current="page"]'), bg: getComputedStyle(b).backgroundColor, border: getComputedStyle(b).borderTopColor };
  document.documentElement.removeAttribute('data-contrast'); host.remove();
  return JSON.stringify(out);
})()
