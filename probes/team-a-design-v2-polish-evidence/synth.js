(() => {
  const html = document.documentElement;
  html.setAttribute('data-contrast', 'high');
  const host = document.createElement('div');
  host.innerHTML = '<div class="sidebar"><nav><button type="button" aria-current="page">Chat</button><button type="button">Tasks</button></nav></div>';
  document.body.append(host);
  const sel = host.querySelector('button[aria-current="page"]');
  const unsel = host.querySelector('button:not([aria-current])');
  const d = document.createElement('input'); d.type = 'text'; d.disabled = true; d.value = 'x'; document.body.append(d);
  const r = document.createElement('input'); r.type = 'text'; r.readOnly = true; r.value = 'y'; document.body.append(r);
  const g = (el, p) => getComputedStyle(el).getPropertyValue(p).trim();
  const out = {
    selectedRow: { bg: g(sel, 'background-color'), border: g(sel, 'border-top-color'), color: g(sel, 'color'), matches: sel.matches('[data-contrast="high"] .sidebar nav button[aria-current="page"]') },
    unselectedRow: { bg: g(unsel, 'background-color'), border: g(unsel, 'border-top-color') },
    disabledInput: { border: g(d, 'border-top-color'), bg: g(d, 'background-color'), color: g(d, 'color') },
    readonlyInput: { border: g(r, 'border-top-color'), bg: g(r, 'background-color') },
  };
  host.remove(); d.remove(); r.remove(); html.removeAttribute('data-contrast');
  return JSON.stringify(out);
})()
