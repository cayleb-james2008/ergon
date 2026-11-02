(() => {
  const sel = document.querySelector('.phone .tabbar button[aria-selected="true"]');
  const un = document.querySelector('.phone .tabbar button[aria-selected="false"]');
  const op = (b) => { const s = getComputedStyle(b, '::before'); return { content: s.content, opacity: s.opacity, background: s.backgroundColor, transition: s.transitionProperty + ' / ' + s.transitionDuration }; };
  return JSON.stringify({ selected: op(sel), unselected: op(un) });
})()
