(() => {
  const rows = [...document.querySelectorAll('.sidebar nav button')];
  return JSON.stringify(rows.map((b) => ({ text: b.textContent.trim(), current: b.getAttribute('aria-current'), cls: b.className, bg: getComputedStyle(b).backgroundColor })));
})()
