(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const parse = (s) => {
    const m = String(s).match(/rgba?\(([^)]+)\)/);
    if (!m) return null;
    const p = m[1].split(',').map(Number);
    return { r: p[0], g: p[1], b: p[2], a: p[3] === undefined ? 1 : p[3] };
  };
  const over = (bg, fg) => ({ r: Math.round(fg.r * fg.a + bg.r * (1 - fg.a)), g: Math.round(fg.g * fg.a + bg.g * (1 - fg.a)), b: Math.round(fg.b * fg.a + bg.b * (1 - fg.a)), a: 1 });
  const lum = (c) => { const f = (v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b); };
  const contrast = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
  const html = document.documentElement;
  const phone = document.querySelector('.phone');
  const card = document.querySelector('.card');
  const composer = document.querySelector('.composer');
  const input = document.querySelector('input[type="text"]');
  const phoneBg = parse(cs(phone, 'background-color'));
  const cardBg = parse(cs(card, 'background-color'));
  const composerBg = parse(cs(composer, 'background-color'));
  // composite the composer's own background over the card it sits on, then measure its border against that
  const composerComposite = (composerBg && cardBg) ? over(cardBg, composerBg) : composerBg;
  const composerBorder = parse(cs(composer, 'border-top-color'));
  const inputBorder = input ? parse(cs(input, 'border-top-color')) : null;
  const inputBg = input ? parse(cs(input, 'background-color')) : null;
  const inputComposite = (inputBg && cardBg) ? over(cardBg, inputBg) : inputBg;
  // live JS motion wiring: click the approval, then read the pill
  const pillBefore = document.querySelector('[data-approval] .pill')?.textContent?.trim() || null;
  const approve = document.querySelector('[data-approve]');
  if (approve) approve.click();
  const pillAfter = document.querySelector('[data-approval] .pill')?.textContent?.trim() || null;
  const tabs = [...document.querySelectorAll('[data-tabs] button[role="tab"]')];
  const tabBefore = tabs.map((b) => b.getAttribute('aria-selected'));
  if (tabs[1]) tabs[1].click();
  const tabAfter = tabs.map((b) => b.getAttribute('aria-selected'));
  const panels = ['panel-chat', 'panel-tasks', 'panel-make'].map((id) => document.getElementById(id)?.hidden ?? 'missing');
  return JSON.stringify({
    title: document.title,
    url: location.href,
    tokens: {
      surface1: cs(html, '--surface-1'), surface2: cs(html, '--surface-2'), surface3: cs(html, '--surface-3'),
      light1: cs(html, '--light-1'), fieldLine: cs(html, '--field-line'), lineBright: cs(html, '--line-bright'),
    },
    surfaces: {
      phoneBg: cs(phone, 'background-color'), phoneBlur: cs(phone, 'backdrop-filter'),
      cardBg: cs(card, 'background-color'), composerBg: cs(composer, 'background-color'),
    },
    composerBoundary: {
      border: cs(composer, 'border-top-color'),
      compositeBg: composerComposite ? `rgba(${composerComposite.r},${composerComposite.g},${composerComposite.b},1)` : null,
      contrast: (composerBorder && composerComposite) ? Number(contrast(composerBorder, composerComposite).toFixed(2)) : null,
    },
    fieldBoundary: {
      border: input ? cs(input, 'border-top-color') : null,
      compositeBg: inputComposite ? `rgba(${inputComposite.r},${inputComposite.g},${inputComposite.b},1)` : null,
      contrast: (inputBorder && inputComposite) ? Number(contrast(inputBorder, inputComposite).toFixed(2)) : null,
    },
    motion: {
      reduced: matchMedia('(prefers-reduced-motion: reduce)').matches,
      animationsNow: document.getAnimations().length,
      pillBefore, pillAfter, tabBefore, tabAfter, panelsAfter: panels,
    },
    bodyTextHas: {
      label: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE'),
      marlow: document.body.innerText.includes('Marlow'),
      chat: document.body.innerText.includes('Chat'),
    },
    textSample: document.body.innerText.slice(0, 240).replace(/\s+/g, ' ').trim(),
  });
})()
