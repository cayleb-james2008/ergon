(async () => {
  const rules = [];
  for (const ss of document.styleSheets) {
    try { for (const r of ss.cssRules) { if (r.selectorText && r.selectorText.includes('data-contrast="high"')) rules.push(r.selectorText); if (r.cssRules) for (const rr of r.cssRules) if (rr.selectorText && rr.selectorText.includes('data-contrast="high"')) rules.push(rr.selectorText); } } catch (e) {}
  }
  const cssText = await fetch('/comp-kit.css', { cache: 'no-store' }).then((r) => r.text());
  const row = document.querySelector('.sidebar nav button[aria-current="page"]');
  document.documentElement.dataset.contrast = 'high';
  await new Promise((r) => setTimeout(r, 50));
  const after = row ? { bg: getComputedStyle(row).backgroundColor, border: getComputedStyle(row).borderTopColor } : null;
  document.documentElement.dataset.contrast = '';
  return JSON.stringify({
    cssomHcRules: rules.filter((r) => r.includes('sidebar') || r.includes('tabbar') || r.includes('input')).slice(0, 8),
    fetchedCssHasChildRule: cssText.includes('[data-contrast="high"] .sidebar nav button[aria-current="page"]'),
    servedBytes: cssText.length,
    rowAfterHighContrast: after,
    hrefCss: [...document.querySelectorAll('link[rel="stylesheet"]')].map((l) => l.href),
  });
})()
