(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const row = document.querySelector('.sidebar nav button[aria-current="page"]');
  return JSON.stringify(row ? { text: row.textContent.trim(), bg: cs(row, 'background-color'), border: cs(row, 'border-top-color'), color: cs(row, 'color') } : null);
})()
