(async () => {
  const txt = await fetch('/comp-kit.css', { cache: 'no-store' }).then((r) => r.text());
  const hasChildRule = txt.includes('[data-contrast="high"] .sidebar nav button[aria-current="page"]');
  const hasDisabledRule = txt.includes('[data-contrast="high"] input:disabled');
  const host = document.createElement('div');
  host.innerHTML = '<div class="sidebar"><nav><button type="button" aria-current="page">Chat</button></nav></div>';
  document.body.append(host);
  const btn = host.querySelector('button');
  document.documentElement.setAttribute('data-contrast', 'high');
  const cs = getComputedStyle(btn);
  const out = { servedBytes: txt.length, hasChildRule, hasDisabledRule, matches: btn.matches('[data-contrast="high"] .sidebar nav button[aria-current="page"]'), bg: cs.backgroundColor, border: cs.borderTopColor, color: cs.color };
  document.documentElement.removeAttribute('data-contrast'); host.remove();
  return JSON.stringify(out);
})()
