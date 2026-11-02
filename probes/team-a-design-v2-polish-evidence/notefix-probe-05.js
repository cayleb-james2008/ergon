(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const row = document.querySelector('.sidebar nav button[aria-current="page"]');
  const before = row ? { bg: cs(row, 'background-color'), border: cs(row, 'border-color'), color: cs(row, 'color'), ariaCurrent: row.getAttribute('aria-current') } : null;
  const sidebar = document.querySelector('.sidebar');
  const sidebarBefore = { shadow: cs(sidebar, 'box-shadow'), blur: cs(sidebar, 'backdrop-filter') };
  document.documentElement.dataset.contrast = 'high';
  const after = row ? { bg: cs(row, 'background-color'), border: cs(row, 'border-color') } : null;
  const sidebarAfter = { shadow: cs(sidebar, 'box-shadow'), blur: cs(sidebar, 'backdrop-filter') };
  document.documentElement.dataset.contrast = '';
  // tab click moves aria-current to the selected sidebar row (N2a live proof)
  const buttons = [...document.querySelectorAll('.sidebar nav button')];
  buttons[1]?.click();
  const afterClick = buttons.map((b) => b.getAttribute('aria-current'));
  return JSON.stringify({ url: location.href, selectedRowBefore: before, selectedRowAfterHighContrast: after, sidebarBefore, sidebarAfterHighContrast: sidebarAfter, ariaCurrentAfterTabClick: afterClick, labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE') });
})()
