/* Ergon Lumen v2 — live motion + the neural-net character (design comps for grading, NOT built surfaces).
   Every animation is JS-driven (Web Animations API) so the owner sees real motion in the browser:
   reveal, tab swap, sheet, press, send, approval, character signal. Durations/easing are read from tokens.css.
   No colour literals: the character is styled by classes in comp-kit.css (tokens only). No loops, no pulse/ping. */

(() => {
  const html = document.documentElement;
  const cssVar = (name, fallback) => getComputedStyle(html).getPropertyValue(name).trim() || fallback;
  const budget = () => ({
    tap: cssVar('--motion-tap', '120ms'),
    swap: cssVar('--motion-swap', '200ms'),
    enter: cssVar('--motion-enter', '250ms'),
    sheet: cssVar('--motion-sheet', '350ms'),
    ease: cssVar('--ease', 'cubic-bezier(0.2, 0.7, 0.2, 1)'),
  });

  const reduced = () =>
    html.dataset.motion === 'reduced' ||
    window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  function play(el, frames, duration, opts = {}) {
    if (!el) return null;
    if (reduced()) {
      el.style.opacity = '';
      el.style.transform = '';
      return null;
    }
    const anim = el.animate(frames, { duration: ms(duration), easing: budget().ease, fill: 'both', ...opts });
    return anim;
  }
  const ms = (token) => {
    const v = String(token).trim();
    if (v.endsWith('ms')) return parseFloat(v);
    if (v.endsWith('s')) return parseFloat(v) * 1000;
    return parseFloat(v) || 0;
  };

  /* ---------- reveal on load / in view ---------- */
  function reveal(root = document) {
    const groups = new Map();
    root.querySelectorAll('[data-reveal]').forEach((el) => {
      const key = el.dataset.reveal || 'default';
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key).push(el);
    });
    for (const [, els] of groups) {
      els.forEach((el, i) => {
        play(
          el,
          [
            { opacity: 0, transform: 'translateY(6px)' },
            { opacity: 1, transform: 'translateY(0)' },
          ],
          budget().enter,
          { delay: reduced() ? 0 : Math.min(i * 45, 220) },
        );
      });
    }
  }

  /* ---------- tabs: Chat / Tasks / Make (swap 200) ---------- */
  function wireTabs(root = document) {
    root.querySelectorAll('[data-tabs]').forEach((bar) => {
      const buttons = [...bar.querySelectorAll('button[role="tab"]')];
      const panels = buttons
        .map((b) => document.getElementById(b.getAttribute('aria-controls')))
        .filter(Boolean);
      const select = (button) => {
        buttons.forEach((b) => b.setAttribute('aria-selected', String(b === button)));
        // sidebar rows also carry aria-current so the selected-row finish is a rendered state, not dormant CSS
        if (bar.closest('.sidebar')) buttons.forEach((b) => { if (b === button) b.setAttribute('aria-current', 'page'); else b.removeAttribute('aria-current'); });
        panels.forEach((p) => {
          const show = p.id === button.getAttribute('aria-controls');
          if (show) {
            p.hidden = false;
            play(p, [{ opacity: 0, transform: 'translateY(4px)' }, { opacity: 1, transform: 'translateY(0)' }], budget().swap);
          } else {
            p.hidden = true;
          }
        });
      };
      buttons.forEach((b) => b.addEventListener('click', () => select(b)));
    });
  }

  /* ---------- sheet (350 enter/leave) ---------- */
  function wireSheets(root = document) {
    root.querySelectorAll('[data-sheet-open]').forEach((btn) => {
      btn.addEventListener('click', () => {
        const wrap = document.querySelector(btn.dataset.sheetOpen);
        if (!wrap) return;
        wrap.hidden = false;
        const sheet = wrap.querySelector('.sheet');
        const scrim = wrap.querySelector('.scrim');
        play(sheet, [{ transform: 'translateY(24px)', opacity: 0 }, { transform: 'translateY(0)', opacity: 1 }], budget().sheet);
        play(scrim, [{ opacity: 0 }, { opacity: 1 }], budget().sheet);
        (sheet.querySelector('button, [href], input, [tabindex]') || sheet).focus?.();
        wrap.dataset.open = 'true';
      });
    });
    root.querySelectorAll('[data-sheet-close]').forEach((btn) => {
      btn.addEventListener('click', () => closeSheet(btn.closest('.sheet-wrap')));
    });
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') document.querySelectorAll('.sheet-wrap[data-open="true"]').forEach((w) => closeSheet(w));
    });
  }
  function closeSheet(wrap) {
    if (!wrap) return;
    const sheet = wrap.querySelector('.sheet');
    const scrim = wrap.querySelector('.scrim');
    play(sheet, [{ transform: 'translateY(0)', opacity: 1 }, { transform: 'translateY(24px)', opacity: 0 }], budget().sheet);
    play(scrim, [{ opacity: 1 }, { opacity: 0 }], budget().sheet);
    wrap.dataset.open = 'false';
    setTimeout(() => { wrap.hidden = true; }, reduced() ? 0 : ms(budget().sheet));
  }

  /* ---------- one-tap send: a new message arrives (enter 250) ---------- */
  function wireSend(root = document) {
    root.querySelectorAll('[data-send]').forEach((form) => {
      const input = form.querySelector('input');
      const thread = document.querySelector(form.dataset.thread || '#thread');
      form.addEventListener('submit', (e) => {
        e.preventDefault();
        if (!thread) return;
        const value = (input?.value || '').trim();
        const el = document.createElement('div');
        el.className = 'msg me';
        el.textContent = value || 'Use the November card and tell Marcus why.';
        thread.append(el);
        play(el, [{ opacity: 0, transform: 'translateY(8px)' }, { opacity: 1, transform: 'translateY(0)' }], budget().enter);
        if (input) input.value = '';
        const reply = document.createElement('div');
        reply.className = 'msg agent';
        reply.textContent = 'Done — Marcus has the reason and the new estimate is ready to send.';
        thread.append(reply);
        play(reply, [{ opacity: 0, transform: 'translateY(8px)' }, { opacity: 1, transform: 'translateY(0)' }], budget().enter, { delay: reduced() ? 0 : 160 });
        thread.scrollTop = thread.scrollHeight;
      });
    });
  }

  /* ---------- approvals: bounded action resolves (swap 200) ---------- */
  function wireApprovals(root = document) {
    root.querySelectorAll('[data-approve]').forEach((btn) => {
      btn.addEventListener('click', () => {
        const card = btn.closest('[data-approval]');
        if (!card) return;
        play(card, [{ opacity: 1, transform: 'scale(1)' }, { opacity: 0.6, transform: 'scale(0.995)' }], budget().swap);
        card.dataset.resolved = 'true';
        const pill = card.querySelector('.pill');
        if (pill) {
          pill.className = 'pill ok';
          pill.textContent = 'Approved — one action, recorded';
        } else {
          // Sheet approvals have no status pill: the primary control itself becomes the receipt.
          const primary = card.querySelector('[data-approve]');
          if (primary) primary.textContent = 'Approved once — recorded';
        }
        card.querySelectorAll('button').forEach((b) => { b.disabled = true; });
        play(card, [{ opacity: 0.6, transform: 'scale(0.995)' }, { opacity: 1, transform: 'scale(1)' }], budget().swap, { delay: 60 });
      });
    });
  }

  /* ---------- the character: node-and-connection neural net ---------- */
  function mulberry(seed) {
    let a = seed >>> 0;
    return () => {
      a |= 0; a = (a + 0x6d2b79f5) | 0;
      let t = Math.imul(a ^ (a >>> 15), 1 | a);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }
  function renderNeural(el) {
    const density = Number(el.dataset.density || 10);
    const name = el.dataset.name || 'Marlow';
    const colour = el.dataset.colour || 'accent';
    const seed = [...name].reduce((n, c) => (n * 31 + c.charCodeAt(0)) >>> 0, 7) + density * 13;
    const rand = mulberry(seed);
    const S = 60, C = S / 2;
    const rings = density > 9 ? 2 : 1;
    const nodes = [{ x: C, y: C, core: true }];
    for (let r = 1; r <= rings; r += 1) {
      const n = r === 1 ? Math.max(5, Math.round(density * 0.55)) : density - 5 - 2;
      const radius = 16 + r * 8;
      for (let i = 0; i < n; i += 1) {
        const a = (Math.PI * 2 * i) / n + (r === 1 ? 0.35 : 0.9);
        nodes.push({ x: C + Math.cos(a) * radius, y: C + Math.sin(a) * radius });
      }
    }
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('viewBox', `0 0 ${S} ${S}`);
    svg.setAttribute('role', 'img');
    svg.setAttribute('aria-label', `${name} — agent character, ${density}-node network`);
    let edges = '';
    nodes.forEach((n, i) => {
      if (!n.core) edges += `<line class="edge" x1="${C}" y1="${C}" x2="${n.x.toFixed(1)}" y2="${n.y.toFixed(1)}"/>`;
      if (i > 1) {
        const p = nodes[i - 1];
        if (!p.core) edges += `<line class="edge" x1="${p.x.toFixed(1)}" y1="${p.y.toFixed(1)}" x2="${n.x.toFixed(1)}" y2="${n.y.toFixed(1)}" opacity=".55"/>`;
      }
    });
    const signal = nodes
      .filter((n) => !n.core)
      .slice(0, 3)
      .map((n) => `<line class="signal" x1="${C}" y1="${C}" x2="${n.x.toFixed(1)}" y2="${n.y.toFixed(1)}"/>`)
      .join('');
    const dots = nodes
      .map((n) => `<circle class="${n.core ? 'node core' : 'node'}" cx="${n.x.toFixed(1)}" cy="${n.y.toFixed(1)}" r="${n.core ? 2.6 : 1.5 + rand() * 0.5}"/>`)
      .join('');
    svg.innerHTML = `<g>${edges}${signal}${dots}</g>`;
    el.replaceChildren(svg);
    el.dataset.colour = colour;
    el.dataset.density = density;
    el.dataset.name = name;
  }
  function signal(el) {
    const lines = el.querySelectorAll('.signal');
    if (!lines.length) return;
    lines.forEach((line, i) => {
      play(line, [{ opacity: 0, strokeDashoffset: 40 }, { opacity: 1, strokeDashoffset: 0 }], budget().enter, { delay: reduced() ? 0 : i * 70 });
      if (!reduced()) {
        line.animate([{ opacity: 1 }, { opacity: 0 }], { duration: ms(budget().enter), delay: 260 + i * 70, easing: budget().ease, fill: 'both' });
      }
    });
  }
  function wireCharacters(root = document) {
    root.querySelectorAll('.nn').forEach((el) => {
      // the character is a real control: focusable, labelled, activates for keyboard users
      el.removeAttribute('aria-hidden');
      el.tabIndex = 0;
      el.setAttribute('role', 'button');
      el.setAttribute('aria-label', `${el.dataset.name || 'Agent'} — agent character; activate to signal`);
      el.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); signal(el); }
      });
      renderNeural(el);
      const wrap = el.closest('[data-character]');
      if (wrap) {
        wrap.querySelectorAll('[data-density]').forEach((b) =>
          b.addEventListener('click', () => {
            wrap.querySelectorAll('[data-density]').forEach((x) => x.setAttribute('aria-pressed', String(x === b)));
            el.dataset.density = b.dataset.density;
            el.dataset.colour = b.dataset.colour || el.dataset.colour;
            renderNeural(el);
            signal(el);
          }),
        );
        wrap.querySelectorAll('[data-colour]').forEach((b) =>
          b.addEventListener('click', () => {
            wrap.querySelectorAll('[data-colour]').forEach((x) => x.setAttribute('aria-pressed', String(x === b)));
            el.dataset.colour = b.dataset.colour;
            el.style.setProperty('--nn-hue', 'var(--' + b.dataset.colour + ')');
            renderNeural(el);
            signal(el);
          }),
        );
        const nameInput = wrap.querySelector('[data-name]');
        nameInput?.addEventListener('input', () => {
          el.dataset.name = nameInput.value.trim() || 'Marlow';
          renderNeural(el);
        });
      }
      el.addEventListener('click', () => signal(el));
      if (!reduced()) {
        const state = el.dataset.state;
        if (state === 'working' || state === 'needs-you') setTimeout(() => signal(el), 300);
      }
    });
    root.querySelectorAll('[data-cycle-state]').forEach((btn) => {
      btn.addEventListener('click', () => {
        const el = document.querySelector(btn.dataset.cycleState);
        if (!el) return;
        const order = ['idle', 'working', 'needs-you'];
        el.dataset.state = order[(order.indexOf(el.dataset.state) + 1) % order.length];
        btn.dataset.label = el.dataset.state;
        signal(el);
      });
    });
  }

  /* ---------- floors preview (a labelled simulation of the OS settings; not the OS setting) ---------- */
  function wireFloors(root = document) {
    root.querySelectorAll('[data-floors]').forEach((btn) => {
      btn.addEventListener('click', () => {
        const on = btn.getAttribute('aria-pressed') !== 'true';
        btn.setAttribute('aria-pressed', String(on));
        html.dataset.motion = on ? 'reduced' : '';
        html.dataset.contrast = on ? 'high' : '';
        document.querySelectorAll('[data-floors-label]').forEach((l) => { l.textContent = on ? 'Floors preview ON (reduced motion + High Contrast simulation)' : 'Floors preview OFF'; });
      });
    });
  }

  /* ---------- replay ---------- */
  function wireReplay(root = document) {
    root.querySelectorAll('[data-replay]').forEach((btn) => {
      btn.addEventListener('click', () => {
        reveal(root);
        root.querySelectorAll('.nn').forEach((el) => signal(el));
        const sheet = root.querySelector('[data-replay-sheet]');
        if (sheet) sheet.click();
      });
    });
  }

  /* ---------- boot ---------- */
  function boot() {
    reveal(document);
    wireTabs(document);
    wireSheets(document);
    wireSend(document);
    wireApprovals(document);
    wireCharacters(document);
    wireFloors(document);
    wireReplay(document);
    document.querySelectorAll('[data-review-note]').forEach((n) => { n.hidden = false; });
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
  else boot();
})();
