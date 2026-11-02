(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const parse = (s) => { const str = String(s).trim(); const m = str.match(/rgba?\(([^)]+)\)/); if (m) { const p = m[1].split(',').map(Number); return { r: p[0], g: p[1], b: p[2], a: p[3] === undefined ? 1 : p[3] }; } const h = str.match(/^#([0-9a-f]{6})$/i); if (h) { const n = parseInt(h[1], 16); return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255, a: 1 }; } return null; };
  const over = (bg, fg) => ({ r: Math.round(fg.r * fg.a + bg.r * (1 - fg.a)), g: Math.round(fg.g * fg.a + bg.g * (1 - fg.a)), b: Math.round(fg.b * fg.a + bg.b * (1 - fg.a)), a: 1 });
  const lum = (c) => { const f = (v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b); };
  const contrast = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
  const html = document.documentElement;
  const tok = (n) => parse(cs(html, n));
  let stack = tok('--base'); stack = over(stack, tok('--ambient-cool')); stack = over(stack, tok('--ambient-accent')); stack = over(stack, tok('--surface-1'));
  const inputBg = over(stack, tok('--surface-1'));
  const field = document.querySelector('input[type="text"]:not(.composer input)') || document.querySelector('input[type="text"]');
  const fieldBorder = field ? parse(cs(field, 'border-top-color')) : null;
  const fieldComposite = fieldBorder ? over(inputBg, fieldBorder) : null;
  // open the approval sheet, then check the remember checkbox focus ring
  document.querySelector('[data-sheet-open]')?.click();
  const sheetOpen = document.querySelector('.sheet-wrap')?.dataset.open || null;
  const cb = document.querySelector('input[type="checkbox"]');
  if (cb) cb.focus();
  return JSON.stringify({
    url: location.href,
    fieldBoundary: { borderToken: sheetOpen ? null : cs(field, 'border-top-color'), contrast: (fieldBorder && fieldComposite) ? Number(contrast(fieldComposite, inputBg).toFixed(2)) : null, floor: 3 },
    sheet: { open: sheetOpen, rememberText: document.querySelector('.sheet label')?.innerText?.replace(/\s+/g, ' ').trim() || null },
    checkbox: cb ? { outlineWidth: cs(cb, 'outline-width'), outlineColor: cs(cb, 'outline-color'), accentColor: cs(cb, 'accent-color'), focusVisible: cb.matches(':focus-visible') } : null,
    labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE'),
  });
})()
