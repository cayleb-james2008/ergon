(() => {
  const out = { sheetRules: 0, nested: [], hcRules: [] };
  for (const ss of document.styleSheets) {
    try {
      const walk = (rules, parent) => {
        for (const r of rules) {
          out.sheetRules++;
          if (r.parentRule) out.nested.push({ sel: r.selectorText, parent: r.parentRule.selectorText });
          if (r.selectorText && r.selectorText.includes('data-contrast')) out.hcRules.push({ sel: r.selectorText, parent: r.parentRule ? r.parentRule.selectorText : null });
          if (r.cssRules) walk(r.cssRules, r);
        }
      };
      walk(ss.cssRules, null);
    } catch (e) { out.err = String(e); }
  }
  const host = document.createElement('div');
  host.innerHTML = '<div class="sidebar"><nav><button type="button" aria-current="page">Chat</button></nav></div>';
  document.body.append(host);
  const btn = host.querySelector('button');
  out.matchesSimple = btn.matches('.sidebar nav button');
  document.documentElement.setAttribute('data-contrast', 'high');
  out.matchesHc = btn.matches('[data-contrast="high"] .sidebar nav button');
  out.bg = getComputedStyle(btn).backgroundColor;
  document.documentElement.removeAttribute('data-contrast');
  host.remove();
  return JSON.stringify(out);
})()
