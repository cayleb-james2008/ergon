(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const inputs = [...document.querySelectorAll('input')];
  const disabled = inputs.find((i) => i.disabled);
  const readonly = inputs.find((i) => i.readOnly && !i.disabled);
  const snap = (i) => i ? { value: i.value, color: cs(i, 'color'), bg: cs(i, 'background-color'), border: cs(i, 'border-top-color') } : null;
  const out = { url: location.href, normal: { disabled: snap(disabled), readonly: snap(readonly) } };
  document.documentElement.setAttribute('data-contrast', 'high');
  out.withDataContrast = { disabled: snap(disabled), readonly: snap(readonly) };
  document.documentElement.removeAttribute('data-contrast');
  return JSON.stringify(out);
})()
