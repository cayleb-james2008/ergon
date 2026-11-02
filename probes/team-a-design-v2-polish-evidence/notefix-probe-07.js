(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const inputs = [...document.querySelectorAll('input')];
  const disabled = inputs.find((i) => i.disabled);
  const readonly = inputs.find((i) => i.readOnly && !i.disabled);
  const snap = (i) => i ? { value: i.value, color: cs(i, 'color'), bg: cs(i, 'background-color'), border: cs(i, 'border-top-color'), borderStyle: cs(i, 'border-top-style') } : null;
  const before = { disabled: snap(disabled), readonly: snap(readonly) };
  document.documentElement.dataset.contrast = 'high';
  const after = { disabled: snap(disabled), readonly: snap(readonly) };
  document.documentElement.dataset.contrast = '';
  return JSON.stringify({ url: location.href, beforeHighContrast: before, afterHighContrast: after, emptyState: !!document.querySelector('.empty'), labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE') });
})()
