(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const g = (sel) => { const el = document.querySelector(sel); return el ? { shadow: cs(el, 'box-shadow'), blur: cs(el, 'backdrop-filter'), bg: cs(el, 'background-color') } : null; };
  return JSON.stringify({ composer: g('.phone .composer'), tabbar: g('.phone .tabbar'), banner: g('.banner'), phone: g('.phone') });
})()
