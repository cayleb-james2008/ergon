(() => {
  const html = document.documentElement;
  const row = document.querySelector('.sidebar nav button[aria-current="page"]');
  const unsel = document.querySelector('.sidebar nav button:not([aria-current])');
  const compose = () => ({
    htmlAttr: html.getAttribute('data-contrast'),
    rowBg: row ? getComputedStyle(row).backgroundColor : null,
    rowBorder: row ? getComputedStyle(row).borderTopColor : null,
    unselBg: unsel ? getComputedStyle(unsel).backgroundColor : null,
    rowMatches: row ? row.matches('[data-contrast="high"] .sidebar nav button[aria-current="page"]') : null,
    selMatches: row ? row.matches('.sidebar nav button[aria-current="page"]') : null,
    buttonFace: (() => { const b = document.createElement('button'); b.style.background = 'ButtonFace'; document.body.append(b); const v = getComputedStyle(b).backgroundColor; b.remove(); return v; })(),
    canvas: (() => { const b = document.createElement('div'); b.style.background = 'Canvas'; document.body.append(b); const v = getComputedStyle(b).backgroundColor; b.remove(); return v; })(),
    highlight: (() => { const b = document.createElement('div'); b.style.background = 'Highlight'; document.body.append(b); const v = getComputedStyle(b).backgroundColor; b.remove(); return v; })(),
  });
  const before = compose();
  html.setAttribute('data-contrast', 'high');
  const after = compose();
  html.removeAttribute('data-contrast');
  return JSON.stringify({ before, after });
})()
