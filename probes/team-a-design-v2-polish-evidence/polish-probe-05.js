(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const inset = (sel) => [...document.querySelectorAll(sel)].filter((el) => String(cs(el, 'box-shadow')).includes('inset')).length;
  const selected = document.querySelector('.sidebar nav button[aria-current="page"]');
  return JSON.stringify({
    url: location.href,
    desktop: {
      insetSurfaces: { cards: inset('.card'), glass: inset('.glass-1, .glass-2, .glass-3'), frames: inset('.desktop'), sidebars: inset('.sidebar') },
      selectedRowBg: cs(selected, 'background-color'), selectedRowBorder: cs(selected, 'border-color'),
      kvTabular: cs(document.querySelector('.kv'), 'font-variant-numeric'),
      sidebarHoverRuleExists: [...document.styleSheets].length,
      phoneComposer: null,
    },
    labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE'),
  });
})()
