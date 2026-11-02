(async () => {
  const txt = await fetch('/comp-kit.css', { cache: 'no-store' }).then((r) => r.text());
  const lines = txt.split('\n').filter((l) => l.includes('data-contrast="high"') && (l.includes('input') || l.includes('sidebar nav')));
  document.documentElement.setAttribute('data-contrast', 'high');
  const row = document.querySelector('.sidebar nav button[aria-current="page"]');
  const sel = '[data-contrast="high"] .sidebar nav button[aria-current="page"]';
  const gen = '[data-contrast="high"] .sidebar nav button';
  const out = {
    servedLines: lines,
    rowMatchesSelectedRule: row ? row.matches(sel) : null,
    rowMatchesGenericRule: row ? row.matches(gen) : null,
    rowBg: row ? getComputedStyle(row).backgroundColor : null,
    rowBorder: row ? getComputedStyle(row).borderTopColor : null,
    rowColor: row ? getComputedStyle(row).color : null,
    htmlAttr: document.documentElement.getAttribute('data-contrast'),
    ariaCurrent: row ? row.getAttribute('aria-current') : null,
  };
  document.documentElement.removeAttribute('data-contrast');
  return JSON.stringify(out);
})()
