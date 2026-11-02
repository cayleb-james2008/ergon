(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const row = document.querySelector('.sidebar nav button[aria-current="page"]');
  const snap = () => row ? { text: row.textContent.trim(), bg: cs(row, 'background-color'), border: cs(row, 'border-top-color'), color: cs(row, 'color') } : null;
  const out = { url: location.href, normal: snap() };
  document.documentElement.setAttribute('data-contrast', 'high');
  out.withDataContrast = snap();
  document.documentElement.removeAttribute('data-contrast');
  return JSON.stringify(out);
})()
