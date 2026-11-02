(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const inputs = [...document.querySelectorAll('input')];
  const disabled = inputs.find((i) => i.disabled), readonly = inputs.find((i) => i.readOnly && !i.disabled);
  const snap = (i) => i ? { value: i.value, color: cs(i, 'color'), bg: cs(i, 'background-color'), border: cs(i, 'border-top-color') } : null;
  return JSON.stringify({ disabled: snap(disabled), readonly: snap(readonly) });
})()
