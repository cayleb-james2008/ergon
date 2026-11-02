(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const row = document.querySelector('.sidebar nav button[aria-current="page"]');
  const before = row ? { text: row.textContent.trim(), bg: cs(row, 'background-color'), border: cs(row, 'border-top-color') } : null;
  document.documentElement.setAttribute('data-contrast', 'high');
  const after = row ? { bg: cs(row, 'background-color'), border: cs(row, 'border-top-color') } : null;
  document.documentElement.removeAttribute('data-contrast');
  return JSON.stringify({ url: location.href, rowBefore: before, rowAfterHighContrast: after, labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE') });
})()
