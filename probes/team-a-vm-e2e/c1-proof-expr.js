(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const parse = (s) => {
    const str = String(s).trim();
    const m = str.match(/rgba?\(([^)]+)\)/);
    if (m) { const p = m[1].split(',').map(Number); return { r: p[0], g: p[1], b: p[2], a: p[3] === undefined ? 1 : p[3] }; }
    const h = str.match(/^#([0-9a-f]{6})$/i);
    if (h) { const n = parseInt(h[1], 16); return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255, a: 1 }; }
    return null;
  };
  const over = (bg, fg) => ({ r: Math.round(fg.r * fg.a + bg.r * (1 - fg.a)), g: Math.round(fg.g * fg.a + bg.g * (1 - fg.a)), b: Math.round(fg.b * fg.a + bg.b * (1 - fg.a)), a: 1 });
  const lum = (c) => { const f = (v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b); };
  const contrast = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
  const html = document.documentElement;
  const tok = (n) => parse(cs(html, n));
  const tokens = Object.fromEntries(['--base','--ambient-cool','--ambient-accent','--surface-1','--surface-2','--surface-3','--light-1','--field-line','--line-bright'].map((n) => [n, cs(html, n)]));
  const T = { base: parse(tokens['--base']), cool: parse(tokens['--ambient-cool']), accent: parse(tokens['--ambient-accent']), s1: parse(tokens['--surface-1']), s2: parse(tokens['--surface-2']), field: parse(tokens['--field-line']), bright: parse(tokens['--line-bright']) };
  // The real stack behind the composer: body base + both ambient washes + phone plane (surface-1) + composer plane (surface-2)
  let stack = T.base;
  stack = over(stack, T.cool); stack = over(stack, T.accent); stack = over(stack, T.s1); stack = over(stack, T.s2);
  const phone = document.querySelector('.phone');
  const composer = document.querySelector('.phone .composer');
  const card = document.querySelector('.phone .card');
  const composerBorder = parse(cs(composer, 'border-top-color'));
  const composerBorderComposite = composerBorder ? over(stack, composerBorder) : null;
  const composerContrast = composerBorderComposite ? Number(contrast(composerBorderComposite, stack).toFixed(2)) : null;
  const pillBefore = document.querySelector('[data-approval] .pill')?.textContent?.trim() || null;
  document.querySelector('[data-approve]')?.click();
  const pillAfter = document.querySelector('[data-approval] .pill')?.textContent?.trim() || null;
  const tabs = [...document.querySelectorAll('[data-tabs] button[role="tab"]')];
  const tabBefore = tabs.map((b) => b.getAttribute('aria-selected'));
  tabs[1]?.click();
  const tabAfter = tabs.map((b) => b.getAttribute('aria-selected'));
  return JSON.stringify({
    url: location.href,
    tokensResolved: Object.fromEntries(Object.entries(tokens).map(([k, v]) => [k, v === '' ? 'UNRESOLVED' : v])),
    surfaces: {
      phoneBg: cs(phone, 'background-color'), phoneBlur: cs(phone, 'backdrop-filter'),
      cardBg: cs(card, 'background-color'), composerBg: cs(composer, 'background-color'),
    },
    composerBoundary: {
      borderToken: cs(composer, 'border-top-color'),
      expected: tokens['--field-line'],
      compositedStack: `rgba(${stack.r},${stack.g},${stack.b},1)`,
      contrast: composerContrast,
      floor: 3,
      pass: composerContrast !== null && composerContrast >= 3,
    },
    motion: { reduced: matchMedia('(prefers-reduced-motion: reduce)').matches, animationsNow: document.getAnimations().length, pillBefore, pillAfter, tabBefore, tabAfter },
    labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE'),
    textSample: document.body.innerText.slice(0, 160).replace(/\s+/g, ' ').trim(),
  });
})()
