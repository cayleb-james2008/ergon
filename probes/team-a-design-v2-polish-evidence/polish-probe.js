(async () => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const parse = (s) => { const str = String(s).trim(); const m = str.match(/rgba?\(([^)]+)\)/); if (m) { const p = m[1].split(',').map(Number); return { r: p[0], g: p[1], b: p[2], a: p[3] === undefined ? 1 : p[3] }; } const h = str.match(/^#([0-9a-f]{6})$/i); if (h) { const n = parseInt(h[1], 16); return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255, a: 1 }; } return null; };
  const over = (bg, fg) => ({ r: Math.round(fg.r * fg.a + bg.r * (1 - fg.a)), g: Math.round(fg.g * fg.a + bg.g * (1 - fg.a)), b: Math.round(fg.b * fg.a + bg.b * (1 - fg.a)), a: 1 });
  const lum = (c) => { const f = (v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b); };
  const contrast = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
  const html = document.documentElement;
  const tok = (n) => cs(html, n);
  const parseTok = (n) => parse(tok(n));
  let stack = parseTok('--base'); stack = over(stack, parseTok('--ambient-cool')); stack = over(stack, parseTok('--ambient-accent')); stack = over(stack, parseTok('--surface-1')); stack = over(stack, parseTok('--surface-2'));
  const composer = document.querySelector('.phone .composer');
  const composerBorder = parse(cs(composer, 'border-top-color'));
  const composerContrast = composerBorder ? Number(contrast(over(stack, composerBorder), stack).toFixed(2)) : null;
  const cards = [...document.querySelectorAll('.card, .glass-1, .glass-2, .glass-3, .phone, .sheet')];
  const insetCount = cards.filter((el) => String(cs(el, 'box-shadow')).includes('inset')).length;
  const tabBtn = (i) => document.querySelectorAll('[data-tabs] button[role="tab"]')[i];
  const indicator = (i) => cs(tabBtn(i), '::before') && getComputedStyle(tabBtn(i), '::before').opacity;
  const tabBefore = [...document.querySelectorAll('[data-tabs] button')].map((b) => b.getAttribute('aria-selected'));
  tabBtn(1)?.click();
  const tabAfter = [...document.querySelectorAll('[data-tabs] button')].map((b) => b.getAttribute('aria-selected'));
  const nn = document.querySelector('.nn');
  nn?.focus();
  const pillBefore = document.querySelector('[data-approval] .pill')?.textContent?.trim() || null;
  document.querySelector('[data-approve]')?.click();
  const pillAfter = document.querySelector('[data-approval] .pill')?.textContent?.trim() || null;
  await new Promise((r) => setTimeout(r, 2500));
  const anims = document.getAnimations();
  return JSON.stringify({
    url: location.href,
    polishTokens: { depthShade: tok('--depth-shade'), depthLift: tok('--depth-lift') },
    surfaces: {
      cardsWithInsetRim: insetCount,
      cardsChecked: cards.length,
      phoneBg: cs(document.querySelector('.phone'), 'background-color'),
      phoneBlur: cs(document.querySelector('.phone'), 'backdrop-filter'),
      cardShadow: cs(document.querySelector('.card'), 'box-shadow'),
      composerBorder: cs(composer, 'border-top-color'),
    },
    composerBoundaryContrast: { contrast: composerContrast, floor: 3, pass: composerContrast !== null && composerContrast >= 3 },
    tabIndicator: { selectedBefore: indicator(0), unselectedBefore: indicator(1), tabsSwap: { before: tabBefore, after: tabAfter } },
    character: nn ? { role: nn.getAttribute('role'), tabIndex: nn.tabIndex, label: nn.getAttribute('aria-label'), outlineColor: cs(nn, 'outline-color'), outlineWidth: cs(nn, 'outline-width') } : null,
    typography: { kvTabular: cs(document.querySelector('.kv'), 'font-variant-numeric') },
    approval: { pillBefore, pillAfter },
    motionSettled: { animations: anims.length, running: anims.filter((a) => a.playState === 'running').length, infinite: anims.filter((a) => { const t = a.effect && a.effect.getTiming ? a.effect.getTiming() : {}; return t.iterations === Infinity; }).length },
    labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE'),
    textSample: document.body.innerText.slice(0, 140).replace(/\s+/g, ' ').trim(),
  });
})()
