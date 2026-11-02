(() => {
  const cs = (el, p) => el ? getComputedStyle(el).getPropertyValue(p).trim() : null;
  const read = () => {
    const composer = document.querySelector('.phone .composer');
    const tabbar = document.querySelector('.phone .tabbar');
    const banner = document.querySelector('.banner');
    const phone = document.querySelector('.phone');
    return {
      composerShadow: cs(composer, 'box-shadow'), composerBlur: cs(composer, 'backdrop-filter'), composerBg: cs(composer, 'background-color'),
      tabbarShadow: cs(tabbar, 'box-shadow'), tabbarBlur: cs(tabbar, 'backdrop-filter'),
      bannerShadow: cs(banner, 'box-shadow'), bannerBlur: cs(banner, 'backdrop-filter'),
      phoneBg: cs(phone, 'background-color'), phoneBlur: cs(phone, 'backdrop-filter'),
    };
  };
  const before = read();
  document.documentElement.dataset.contrast = 'high';
  const after = read();
  document.documentElement.dataset.contrast = '';
  return JSON.stringify({ url: location.href, beforeHighContrast: before, afterHighContrast: after, labelPresent: document.body.innerText.includes('DESIGN COMP — NOT A BUILT SURFACE') });
})()
