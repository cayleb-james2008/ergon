(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const parse = (s) => { const str = String(s).trim(); const m = str.match(/rgba?\(([^)]+)\)/); if (m) { const p = m[1].split(',').map(Number); return { r: p[0], g: p[1], b: p[2], a: p[3] === undefined ? 1 : p[3] }; } const h = str.match(/^#([0-9a-f]{6})$/i); if (h) { const n = parseInt(h[1], 16); return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255, a: 1 }; } return null; };
  const over = (bg, fg) => ({ r: Math.round(fg.r * fg.a + bg.r * (1 - fg.a)), g: Math.round(fg.g * fg.a + bg.g * (1 - fg.a)), b: Math.round(fg.b * fg.a + bg.b * (1 - fg.a)), a: 1 });
  const lum = (c) => { const f = (v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b); };
  const contrast = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
  const html = document.documentElement;
  const tok = (n) => parse(cs(html, n));
  let stack = tok('--base'); stack = over(stack, tok('--ambient-cool')); stack = over(stack, tok('--ambient-accent')); stack = over(stack, tok('--surface-1')); stack = over(stack, tok('--surface-2'));
  const input = document.querySelector('input[type="text"]');
  const inputBg = over(stack, tok('--surface-1'));
  const border = input ? parse(cs(input, 'border-top-color')) : null;
  const comp = border ? over(inputBg, border) : null;
  const pool = document.querySelector('.light-pool');
  return JSON.stringify({
    url: location.href,
    fieldBoundary: { token: input ? cs(input, 'border-top-color') : null, contrast: (border && comp) ? Number(contrast(comp, inputBg).toFixed(2)) : null, floor: 3, pass: (border && comp) ? contrast(comp, inputBg) >= 3 : null },
    emptyState: { present: !!document.querySelector('.empty'), text: document.querySelector('.empty')?.innerText?.replace(/\s+/g, ' ').trim() || null },
    lightPool: pool ? { bg: cs(pool, 'background-color'), blur: cs(pool, 'backdrop-filter') } : null,
    floorsRows: document.body.innerText.includes('reduced motion') && document.body.innerText.includes('High Contrast'),
    labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE'),
  });
})()
