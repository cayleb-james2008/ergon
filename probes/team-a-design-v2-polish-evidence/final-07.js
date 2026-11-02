(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const inputs = [...document.querySelectorAll('input')];
  const disabled = inputs.find((i) => i.disabled), readonly = inputs.find((i) => i.readOnly && !i.disabled);
  const snap = (i) => i ? { color: cs(i, 'color'), bg: cs(i, 'background-color'), border: cs(i, 'border-top-color') } : null;
  const before = { disabled: snap(disabled), readonly: snap(readonly) };
  document.documentElement.setAttribute('data-contrast', 'high');
  const after = { disabled: snap(disabled), readonly: snap(readonly) };
  document.documentElement.removeAttribute('data-contrast');
  return JSON.stringify({ url: location.href, beforeHighContrast: before, afterHighContrast: after, emptyState: !!document.querySelector('.empty'), labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE') });
})()
