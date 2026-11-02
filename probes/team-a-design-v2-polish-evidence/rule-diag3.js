(() => {
  const row = document.querySelector('.sidebar nav button[aria-current="page"]');
  const html = document.documentElement;
  html.setAttribute('data-contrast', 'high');
  const before = { bg: getComputedStyle(row).backgroundColor, border: getComputedStyle(row).borderTopColor, color: getComputedStyle(row).color, transition: getComputedStyle(row).transitionProperty + ' ' + getComputedStyle(row).transitionDuration, inline: row.getAttribute('style') || '' };
  row.style.background = 'Highlight';
  const inlineBg = getComputedStyle(row).backgroundColor;
  row.removeAttribute('style');
  const matched = [];
  for (const ss of document.styleSheets) {
    let idx = 0;
    try {
      for (const r of ss.cssRules) {
        idx++;
        if (!r.selectorText) continue;
        try { if (row.matches(r.selectorText) && r.selectorText.includes('sidebar')) matched.push({ sel: r.selectorText, css: r.cssText.slice(0, 160), idx }); } catch (e) {}
      }
    } catch (e) {}
  }
  html.removeAttribute('data-contrast');
  return JSON.stringify({ before, inlineBg, matchedRules: matched.slice(-8), htmlAttr: html.getAttribute('data-contrast') });
})()
