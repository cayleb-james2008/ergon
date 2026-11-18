//! Embedded web UI — plain HTML/CSS/JS, no framework, no build step.
//! Familiar chat shell: left sidebar with threads, one scrolling thread,
//! one composer row pinned at the bottom. Light + dark themes (dark is
//! the default), the switch in Settings; one agent per person, switched
//! in the sidebar; what the agent has learned is viewable under Memory.

pub const PHONE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>Ergon — your agent</title>
<script>
  try {
    var t = localStorage.getItem('ergon-theme');
    if (t === 'light' || t === 'dark') document.documentElement.setAttribute('data-theme', t);
  } catch (e) { /* dark stays the default */ }
</script>
<style>
  /* hallmark-rune · app: ergon (phone face) · vernacular: chief-of-staff's desk, paper-warm neutrals, terracotta anchor · nav: bottom tab bar · signature: work ledger · tokens: hr-template v1 · critique: weakest State coverage (per-region error text kept; no skeleton theatre added) */
  :root, [data-theme="dark"] {
    color-scheme: dark;
    --bg: #171512; --panel: #201d18; --panel-2: #2a2620; --field: #1b1915;
    --ink: #f0ede6; --muted: #a8a294; --faint: #8a8474; --rule: #35312a;
    --accent: #e08a67; --accent-fill: #b1572f; --accent-fill-hover: #9d4c27; --accent-ink: #ffffff;
    --user-bubble: #2b2926;
    --warn: #eec27f; --warn-bg: #2e2410; --warn-rule: #57431c;
    --err: #f0a3a3; --err-bg: #331a1a;
    --font-body: "Source Sans 3", "Segoe UI", "Trebuchet MS", "Helvetica Neue", Arial, sans-serif;
    --font-mono: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    --text-meta: 11px; --text-label: 12px; --text-small: 13px; --text-dense: 14px;
    --text-body: 15px; --text-title: 18px; --text-screen: 20px; --text-display: 24px;
    --sp-1: 4px; --sp-2: 8px; --sp-3: 12px; --sp-4: 16px; --sp-5: 24px; --sp-6: 32px; --sp-7: 48px;
    --r-control: 6px; --r-card: 8px; --r-panel: 12px; --r-chip: 999px;
    --dur-micro: 120ms; --dur-short: 200ms; --dur-panel: 280ms;
    --ease-out: cubic-bezier(0.23, 1, 0.32, 1);
  }
  [data-theme="light"] {
    color-scheme: light;
    --bg: #f7f6f2; --panel: #ffffff; --panel-2: #efe9df; --field: #ffffff;
    --ink: #201d18; --muted: #6d675c; --faint: #6e6759; --rule: #ddd8ce;
    --accent: #a34a28; --accent-fill: #a34a28; --accent-fill-hover: #8d3f21; --accent-ink: #ffffff;
    --user-bubble: #e9e5db;
    --warn: #7a4b12; --warn-bg: #fdf3e3; --warn-rule: #ecd9ae;
    --err: #8c1d1d; --err-bg: #fdecec;
  }
  * { box-sizing: border-box; }
  html, body { height: 100%; margin: 0; overflow-x: clip; }
  body { font-family: var(--font-body);
         background: var(--bg); color: var(--ink); font-size: var(--text-body); line-height: 1.55;
         display: flex; flex-direction: column; }
  ::selection { background: var(--accent-fill); color: var(--accent-ink); }
  html { scrollbar-color: var(--rule) transparent; caret-color: var(--accent); }
  :focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  button { font: inherit; }
  header { padding: var(--sp-4) var(--sp-4) var(--sp-3); border-bottom: 1px solid var(--rule); }
  header h1 { font-size: var(--text-title); margin: 0; font-weight: 600; line-height: 1.2; }
  header p { margin: 2px 0 0; font-size: var(--text-label); color: var(--muted); }
  #no-key-banner { display: none; background: var(--warn-bg); color: var(--warn);
                   border-bottom: 1px solid var(--warn-rule); padding: var(--sp-3) var(--sp-4); font-size: var(--text-small); }
  main { flex: 1; overflow-y: auto; padding: var(--sp-4) var(--sp-4) 84px; }
  .tab { display: none; flex-direction: column; gap: var(--sp-4); }
  .tab.on { display: flex; }
  .card { background: var(--panel); border: 1px solid var(--rule); border-radius: var(--r-panel); padding: var(--sp-4); }
  .card h3 { margin: 0 0 var(--sp-2); font-size: var(--text-body); font-weight: 600; }
  .card h4 { margin: 0 0 var(--sp-2); font-size: var(--text-label); font-weight: 600; color: var(--faint); }
  .card p { margin: var(--sp-1) 0; line-height: 1.55; }
  .card ul { margin: var(--sp-2) 0; padding-left: var(--sp-4); line-height: 1.6; }
  .quiet { color: var(--muted); font-size: var(--text-dense); }
  .settled { color: var(--faint); font-size: var(--text-small); }
  .row-actions { display: flex; gap: var(--sp-2); margin-top: var(--sp-3); }
  .act-btn { border-radius: var(--r-card); padding: var(--sp-3) var(--sp-4); font-size: var(--text-small); font-weight: 600; cursor: pointer;
             border: 1px solid var(--rule); background: none; color: var(--ink); min-height: 44px; }
  .act-btn.primary { background: var(--accent-fill); border-color: var(--accent-fill); color: var(--accent-ink); }
  .act-btn.primary:hover { background: var(--accent-fill-hover); }
  .item { background: var(--field); border: 1px solid var(--rule); border-radius: var(--r-card); padding: var(--sp-3); margin-top: var(--sp-2); }
  .item .i-kind { font-size: var(--text-meta); color: var(--muted); font-weight: 600; }
  .item .i-name { font-size: var(--text-dense); font-weight: 600; margin-top: 1px; }
  .item .i-desc { font-size: var(--text-small); color: var(--muted); line-height: 1.45; margin-top: 2px; }
  #messages { display: flex; flex-direction: column; gap: var(--sp-3); }
  .msg { border-radius: var(--r-panel); padding: var(--sp-3); font-size: var(--text-body); line-height: 1.55;
         white-space: pre-wrap; word-wrap: break-word; }
  .msg.user { background: var(--user-bubble); align-self: flex-end; max-width: 88%; }
  .msg.assistant { background: var(--panel); border: 1px solid var(--rule); align-self: flex-start; max-width: 94%; }
  /* The signature, same as the desktop face: step rows are the work
     ledger — mono, on a rail, on the record. */
  .msg.step { background: transparent; color: var(--muted); font-size: var(--text-label);
              font-family: var(--font-mono); align-self: flex-start; padding: 0 var(--sp-1) 0 var(--sp-3);
              border-left: 2px solid var(--rule); border-radius: 0; }
  .msg.error { background: var(--err-bg); color: var(--err); align-self: flex-start; }
  .empty { color: var(--faint); text-align: center; margin-top: 8vh; line-height: 1.6; }
  #composer { display: flex; gap: var(--sp-2); margin-top: var(--sp-3); }
  #composer-input { flex: 1; resize: none; border: 1px solid var(--rule); border-radius: var(--r-panel);
                    padding: var(--sp-3); font-size: var(--text-body); font-family: inherit; min-height: 44px;
                    background: var(--field); color: var(--ink); }
  #send-btn { background: var(--ink); color: var(--bg); border: 0; border-radius: var(--r-card);
              padding: var(--sp-3) var(--sp-4); font-size: var(--text-dense); font-weight: 600; cursor: pointer; min-height: 44px; }
  #send-btn:disabled { opacity: 0.45; cursor: default; }
  nav { position: fixed; left: 0; right: 0; bottom: 0; display: flex; background: var(--panel);
        border-top: 1px solid var(--rule); padding-bottom: env(safe-area-inset-bottom); }
  .tab-btn { flex: 1; background: none; border: 0; color: var(--muted); padding: var(--sp-2) 2px var(--sp-2);
             font-size: var(--text-meta); font-weight: 600; cursor: pointer; display: flex; flex-direction: column;
             align-items: center; gap: 3px; position: relative; min-height: 48px; }
  .tab-btn svg { width: 20px; height: 20px; }
  .tab-btn[aria-selected="true"] { color: var(--accent); }
  .tab-badge { position: absolute; top: 4px; right: calc(50% - 20px); background: var(--accent-fill);
               color: var(--accent-ink); border-radius: var(--r-chip); font-size: var(--text-meta); padding: 0 var(--sp-2); font-weight: 600; display: none; font-variant-numeric: tabular-nums; }
  .tab-badge.on { display: block; }
  @media (prefers-reduced-motion: reduce) {
    *, *::before, *::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; scroll-behavior: auto !important; }
  }
</style>
</head>
<body>
  <header>
    <h1>Ergon</h1>
    <p id="who">Your agent that finishes the work</p>
  </header>
  <div id="no-key-banner">Ergon is not connected to a model yet, so the agent cannot run. Nothing here is simulated: without a key, Ergon will not invent answers.</div>
  <main>
    <section class="tab" id="tab-brief" aria-label="Brief">
      <div class="card">
        <h4>Morning brief</h4>
        <h3 id="brief-headline">Loading…</h3>
        <div id="brief-body"></div>
      </div>
      <div class="card">
        <h4>Offers from your agent</h4>
        <div id="offers-list"><p class="quiet">Loading…</p></div>
      </div>
    </section>
    <section class="tab on" id="tab-chat" aria-label="Chat">
      <div id="messages" aria-live="polite">
        <div class="empty">What should your agent finish for you?</div>
      </div>
      <form id="composer">
        <label class="sr-only" for="composer-input" style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)">Describe a task for your agent</label>
        <textarea id="composer-input" rows="1" placeholder="Describe a task…"></textarea>
        <button id="send-btn" type="submit">Send</button>
      </form>
    </section>
    <section class="tab" id="tab-decisions" aria-label="Decisions">
      <div class="card">
        <h4>Decisions waiting on you</h4>
        <div id="decisions-list"><p class="quiet">Loading…</p></div>
      </div>
    </section>
    <section class="tab" id="tab-suite" aria-label="Suite">
      <div class="card">
        <h4>My suite</h4>
        <div id="suite-list"><p class="quiet">Loading…</p></div>
      </div>
    </section>
  </main>
  <nav aria-label="Phone views">
    <button class="tab-btn" type="button" data-tab="brief" aria-selected="false"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8.5" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 7.5V12l3 2" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>Brief</button>
    <button class="tab-btn" type="button" data-tab="chat" aria-selected="true"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 6h16M4 12h16M4 18h9" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>Chat</button>
    <button class="tab-btn" type="button" data-tab="decisions" aria-selected="false"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 4v6M12 10l7 4M12 10l-7 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/><circle cx="12" cy="12" r="2.2" fill="currentColor"/></svg>Decisions<span class="tab-badge" id="dec-badge"></span></button>
    <button class="tab-btn" type="button" data-tab="suite" aria-selected="false"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="4" width="7" height="7" rx="1.6" fill="currentColor"/><rect x="13" y="4" width="7" height="7" rx="1.6" fill="currentColor"/><rect x="4" y="13" width="7" height="7" rx="1.6" fill="currentColor"/><rect x="13" y="13" width="7" height="7" rx="1.6" fill="currentColor"/></svg>Suite</button>
  </nav>
<script>
  let currentThread = null;
  let busy = false;
  const messagesEl = document.getElementById('messages');

  function esc(s) {
    const d = document.createElement('div');
    d.textContent = s == null ? '' : String(s);
    return d.innerHTML;
  }

  function addMsg(role, content) {
    const empty = messagesEl.querySelector('.empty');
    if (empty) empty.remove();
    const div = document.createElement('div');
    div.className = 'msg ' + role;
    div.textContent = content;
    messagesEl.appendChild(div);
    return div;
  }

  const tabBtns = document.querySelectorAll('.tab-btn');
  function showTab(name) {
    tabBtns.forEach((b) => b.setAttribute('aria-selected', b.getAttribute('data-tab') === name ? 'true' : 'false'));
    for (const t of ['brief', 'chat', 'decisions', 'suite']) {
      document.getElementById('tab-' + t).classList.toggle('on', t === name);
    }
    if (name === 'brief') { loadBrief(); loadOffers(); }
    if (name === 'decisions') loadDecisions();
    if (name === 'suite') loadSuite();
  }
  tabBtns.forEach((b) => b.addEventListener('click', () => showTab(b.getAttribute('data-tab'))));

  async function loadStatus() {
    try {
      const r = await fetch('/api/status');
      const s = await r.json();
      if (!s.model_configured) document.getElementById('no-key-banner').style.display = 'block';
      if (s.person && s.person.name) document.getElementById('who').textContent = 'Working for ' + s.person.name + (s.person.role ? ' — ' + s.person.role : '');
    } catch (e) { /* best effort */ }
  }

  async function loadBrief() {
    const head = document.getElementById('brief-headline');
    const bodyEl = document.getElementById('brief-body');
    try {
      const r = await fetch('/api/brief');
      const b = await r.json();
      head.textContent = b.headline || '';
      let html = '';
      if ((b.finished || []).length) {
        html += '<h4>Finished recently</h4><ul>' + b.finished.map((f) => '<li>' + esc(f.task) + ' <span class="settled">(' + esc(f.date) + ')</span></li>').join('') + '</ul>';
      }
      if ((b.learned_recently || []).length) {
        html += '<h4>Learned recently</h4><ul>' + b.learned_recently.map((l) => '<li>' + esc(l) + '</li>').join('') + '</ul>';
      }
      bodyEl.innerHTML = html || '<p class="quiet">Nothing to report yet.</p>';
    } catch (e) {
      head.textContent = 'The brief could not be loaded.';
    }
  }

  async function loadOffers() {
    const el = document.getElementById('offers-list');
    try {
      const r = await fetch('/api/offers');
      const data = await r.json();
      const list = (data.offers || []).filter((o) => o.status === 'pending').reverse();
      if (!list.length) { el.innerHTML = '<p class="quiet">No offers right now.</p>'; return; }
      el.innerHTML = '';
      for (const o of list) {
        const div = document.createElement('div');
        div.className = 'item';
        div.innerHTML = '<div class="i-name">' + esc(o.title) + '</div><div class="i-desc">' + esc(o.detail) + '</div>' +
          '<div class="row-actions"><button class="act-btn primary" data-offer="' + esc(o.id) + '" data-how="accept">Yes, do it</button>' +
          '<button class="act-btn" data-offer="' + esc(o.id) + '" data-how="dismiss">No thanks</button></div>';
        el.appendChild(div);
      }
      el.querySelectorAll('[data-offer]').forEach((btn) => btn.addEventListener('click', async () => {
        btn.disabled = true;
        await fetch('/api/offers/' + encodeURIComponent(btn.getAttribute('data-offer')) + '/' + btn.getAttribute('data-how'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
        loadOffers();
      }));
    } catch (e) {
      el.innerHTML = '<p class="quiet">Offers could not be loaded.</p>';
    }
  }

  async function loadDecisions() {
    const el = document.getElementById('decisions-list');
    try {
      const r = await fetch('/api/decisions');
      const data = await r.json();
      const list = (data.decisions || []).slice().reverse();
      const pending = list.filter((d) => d.status === 'pending').length;
      const badge = document.getElementById('dec-badge');
      badge.textContent = pending;
      badge.classList.toggle('on', pending > 0);
      if (!list.length) { el.innerHTML = '<p class="quiet">Nothing waiting. Consequential actions land here for a quick yes or no.</p>'; return; }
      el.innerHTML = '';
      for (const d of list) {
        const div = document.createElement('div');
        div.className = 'item';
        let inner = '<div class="i-kind">' + esc(d.kind) + '</div><div class="i-name">' + esc(d.summary) + '</div>';
        if (d.status === 'pending') {
          inner += '<div class="row-actions"><button class="act-btn primary" data-decide="' + esc(d.id) + '" data-how="approve">Approve</button>' +
                   '<button class="act-btn" data-decide="' + esc(d.id) + '" data-how="deny">Deny</button></div>';
        } else {
          inner += '<div class="settled">' + esc(d.status) + (d.outcome ? ' — ' + esc(d.outcome) : '') + '</div>';
        }
        div.innerHTML = inner;
        el.appendChild(div);
      }
      el.querySelectorAll('[data-decide]').forEach((btn) => btn.addEventListener('click', async () => {
        btn.disabled = true;
        await fetch('/api/decisions/' + encodeURIComponent(btn.getAttribute('data-decide')) + '/' + btn.getAttribute('data-how'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
        loadDecisions();
      }));
    } catch (e) {
      el.innerHTML = '<p class="quiet">Decisions could not be loaded.</p>';
    }
  }

  async function loadSuite() {
    const el = document.getElementById('suite-list');
    try {
      const r = await fetch('/api/suite');
      const data = await r.json();
      const installed = data.installed || [];
      if (!installed.length) { el.innerHTML = '<p class="quiet">Nothing in the suite yet.</p>'; return; }
      el.innerHTML = installed.map((i) =>
        '<div class="item"><div class="i-kind">' + esc(i.kind) + '</div><div class="i-name">' + esc(i.name) + '</div><div class="i-desc">' + esc(i.description) + '</div></div>'
      ).join('');
    } catch (e) {
      el.innerHTML = '<p class="quiet">The suite could not be loaded.</p>';
    }
  }

  document.getElementById('composer').addEventListener('submit', async (ev) => {
    ev.preventDefault();
    if (busy) return;
    const input = document.getElementById('composer-input');
    const sendBtn = document.getElementById('send-btn');
    const text = input.value.trim();
    if (!text) return;
    busy = true;
    if (!currentThread) {
      const r = await fetch('/api/threads', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
      const t = await r.json();
      currentThread = t.id;
    }
    input.value = '';
    addMsg('user', text);
    sendBtn.disabled = true;
    input.disabled = true;
    sendBtn.textContent = 'Working…';
    const working = addMsg('step', 'Agent is working…');
    try {
      const r = await fetch('/api/chat', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ thread_id: currentThread, message: text })
      });
      const data = await r.json();
      working.remove();
      if (!r.ok) {
        addMsg('error', data.error || 'The agent could not run.');
      } else {
        messagesEl.innerHTML = '';
        for (const m of data.messages) addMsg(m.role, m.content);
      }
    } catch (e) {
      working.remove();
      addMsg('error', 'Could not reach the Ergon server.');
    }
    busy = false;
    sendBtn.disabled = false;
    input.disabled = false;
    sendBtn.textContent = 'Send';
  });

  loadStatus();
  loadDecisions();
</script>
</body>
</html>
"##;

pub const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ergon — your agent that finishes the work</title>
<script>
  try {
    var t = localStorage.getItem('ergon-theme');
    if (t === 'light' || t === 'dark') document.documentElement.setAttribute('data-theme', t);
  } catch (e) { /* dark stays the default */ }
</script>
<style>
  /* hallmark-rune · app: ergon · vernacular: chief-of-staff's desk, paper-warm neutrals, terracotta anchor · nav: sidebar rail (the thread list is the work) + face bar · signature: work ledger · tokens: hr-template v1 · critique: weakest State coverage (per-region error text kept; no skeleton theatre added) */
  :root, [data-theme="dark"] {
    color-scheme: dark;
    --bg: #171512; --panel: #201d18; --panel-2: #2a2620; --sidebar: #100f0d; --field: #1b1915;
    --ink: #f0ede6; --muted: #a8a294; --faint: #8a8474; --rule: #35312a;
    --accent: #e08a67; --accent-fill: #b1572f; --accent-fill-hover: #9d4c27; --accent-ink: #ffffff;
    --user-bubble: #2b2926;
    --warn: #eec27f; --warn-bg: #2e2410; --warn-rule: #57431c;
    --err: #f0a3a3; --err-bg: #331a1a;
    --font-body: "Source Sans 3", "Segoe UI", "Trebuchet MS", "Helvetica Neue", Arial, sans-serif;
    --font-mono: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    --text-meta: 11px; --text-label: 12px; --text-small: 13px; --text-dense: 14px;
    --text-body: 15px; --text-title: 18px; --text-screen: 20px; --text-display: 24px;
    --sp-1: 4px; --sp-2: 8px; --sp-3: 12px; --sp-4: 16px; --sp-5: 24px; --sp-6: 32px; --sp-7: 48px;
    --r-control: 6px; --r-card: 8px; --r-panel: 12px; --r-chip: 999px;
    --dur-micro: 120ms; --dur-short: 200ms; --dur-panel: 280ms;
    --ease-out: cubic-bezier(0.23, 1, 0.32, 1);
  }
  [data-theme="light"] {
    color-scheme: light;
    --bg: #f7f6f2; --panel: #ffffff; --panel-2: #efe9df; --sidebar: #edeae3; --field: #ffffff;
    --ink: #201d18; --muted: #6d675c; --faint: #6e6759; --rule: #ddd8ce;
    --accent: #a34a28; --accent-fill: #a34a28; --accent-fill-hover: #8d3f21; --accent-ink: #ffffff;
    --user-bubble: #e9e5db;
    --warn: #7a4b12; --warn-bg: #fdf3e3; --warn-rule: #ecd9ae;
    --err: #8c1d1d; --err-bg: #fdecec;
  }
  * { box-sizing: border-box; }
  html, body { height: 100%; margin: 0; overflow-x: clip; }
  body { font-family: var(--font-body);
         display: flex; background: var(--bg); color: var(--ink);
         font-size: var(--text-body); line-height: 1.55; }
  ::selection { background: var(--accent-fill); color: var(--accent-ink); }
  html { scrollbar-color: var(--rule) transparent; caret-color: var(--accent); }
  :focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  button { font: inherit; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden;
             clip: rect(0 0 0 0); white-space: nowrap; }

  #sidebar { width: 264px; flex: 0 0 264px; background: var(--sidebar); color: var(--ink);
             display: flex; flex-direction: column; padding: var(--sp-4) var(--sp-3); gap: var(--sp-3);
             border-right: 1px solid var(--rule); min-height: 0; }
  #sidebar h1 { font-size: var(--text-title); margin: 0 var(--sp-2); font-weight: 600; letter-spacing: -0.01em; line-height: 1.2; }
  #sidebar .tagline { font-size: var(--text-label); color: var(--muted); margin: 0 var(--sp-2); line-height: 1.5; }
  .side-label { font-size: var(--text-label); color: var(--faint); margin: 0 var(--sp-2) var(--sp-1); }
  #person-select { width: 100%; background: var(--field); color: var(--ink);
                   border: 1px solid var(--rule); border-radius: var(--r-card); padding: var(--sp-2);
                   font-size: var(--text-dense); min-height: 40px; }
  #new-chat-btn { display: flex; align-items: center; justify-content: center; gap: var(--sp-2);
                  background: var(--accent-fill); color: var(--accent-ink); border: 0; border-radius: var(--r-card);
                  padding: var(--sp-3); font-size: var(--text-dense); font-weight: 600; cursor: pointer; min-height: 44px; }
  #new-chat-btn:hover { background: var(--accent-fill-hover); }
  #new-chat-btn svg { width: 14px; height: 14px; flex: none; }
  #thread-list { list-style: none; margin: 0; padding: 0; overflow-y: auto; flex: 1; min-height: 48px; }
  #thread-list li { margin: 0; }
  .thread-btn { display: block; width: 100%; text-align: left; background: none; border: 0;
                padding: var(--sp-2) var(--sp-3); border-radius: var(--r-card); font-size: var(--text-dense); cursor: pointer;
                color: var(--muted); min-height: 36px; }
  .thread-btn .t-title { display: block; white-space: nowrap; overflow: hidden;
                         text-overflow: ellipsis; }
  .thread-btn .t-sub { display: block; font-size: var(--text-label); color: var(--faint); margin-top: 2px; }
  .thread-btn.active { background: var(--panel-2); color: var(--ink); }
  .thread-btn:hover { color: var(--ink); }
  #side-error { display: none; color: var(--err); font-size: var(--text-label); margin: 0 var(--sp-2); }

  #settings { border-top: 1px solid var(--rule); padding-top: var(--sp-3); display: flex;
              flex-direction: column; gap: var(--sp-3); }
  #settings h2 { font-size: var(--text-label); font-weight: 600; color: var(--faint); margin: 0 var(--sp-2); }
  .set-row { display: flex; align-items: center; gap: var(--sp-2); padding: 0 var(--sp-2); }
  .set-row .set-name { font-size: var(--text-small); color: var(--muted); flex: 1; }
  .theme-btn { background: none; border: 1px solid var(--rule); color: var(--muted);
               border-radius: var(--r-control); padding: var(--sp-1) var(--sp-3); font-size: var(--text-small);
               cursor: pointer; min-height: 32px; }
  .theme-btn[aria-pressed="true"] { background: var(--ink); color: var(--bg); border-color: var(--ink); }
  .side-btn { background: none; border: 1px solid var(--rule); color: var(--ink);
              border-radius: var(--r-card); padding: var(--sp-2) var(--sp-3); font-size: var(--text-small); cursor: pointer;
              margin: 0 var(--sp-2); text-align: left; min-height: 36px; }
  .side-btn:hover { border-color: var(--muted); }
  #memory-panel { display: none; margin: 0 var(--sp-2); background: var(--field);
                  border: 1px solid var(--rule); border-radius: var(--r-card); padding: var(--sp-2) var(--sp-3);
                  font-size: var(--text-label); line-height: 1.5; max-height: 320px; overflow-y: auto;
                  word-wrap: break-word; color: var(--muted); }
  #memory-panel.open { display: block; }
  #memory-panel .mem-head { font-size: var(--text-label); font-weight: 600; color: var(--ink); margin: var(--sp-2) 0 var(--sp-1); }
  #memory-panel .mem-head:first-child { margin-top: 0; }
  #memory-panel .mem-list { margin: 0; padding-left: var(--sp-4); line-height: 1.6; }
  #memory-panel p { margin: var(--sp-1) 0; }
  #memory-panel .mem-file { margin-top: var(--sp-2); border-top: 1px solid var(--rule); padding-top: var(--sp-2); }
  #memory-panel .mem-file summary { cursor: pointer; color: var(--faint); min-height: 28px; }
  #memory-panel .mem-file pre { white-space: pre-wrap; word-wrap: break-word; font-family: var(--font-mono);
                                font-size: var(--text-meta); margin: var(--sp-2) 0 0; color: var(--muted); }
  #person-form { display: none; flex-direction: column; gap: var(--sp-2); padding: 0 var(--sp-2); }
  #person-form.open { display: flex; }
  #person-form input { background: var(--field); color: var(--ink); border: 1px solid var(--rule);
                       border-radius: var(--r-card); padding: var(--sp-2); font-size: var(--text-small); width: 100%; min-height: 36px; }
  #person-form button { background: var(--ink); color: var(--bg); border: 0; border-radius: var(--r-card);
                        padding: var(--sp-2); font-size: var(--text-small); font-weight: 600; cursor: pointer; min-height: 36px; }

  main { flex: 1; display: flex; flex-direction: column; min-width: 0; }

  /* faces — three views over the same core */
  #face-bar { display: flex; gap: var(--sp-1); padding: var(--sp-3) var(--sp-5) 0; max-width: 960px; margin: 0 auto; width: 100%; }
  .face-btn { display: inline-flex; align-items: center; gap: var(--sp-2); background: none; border: 0;
              border-bottom: 2px solid transparent; color: var(--muted); padding: var(--sp-2) var(--sp-3);
              font-size: var(--text-dense); font-weight: 600; cursor: pointer; min-height: 40px; }
  .face-btn svg { width: 15px; height: 15px; flex: none; }
  .face-btn[aria-selected="true"] { color: var(--ink); border-bottom-color: var(--accent); }
  .face-badge { background: var(--accent-fill); color: var(--accent-ink); border-radius: var(--r-chip); font-size: var(--text-meta);
                padding: 1px var(--sp-2); font-weight: 600; display: none; font-variant-numeric: tabular-nums; }
  .face-badge.on { display: inline-block; }
  .face { display: none; flex: 1; flex-direction: column; min-height: 0; }
  .face.on { display: flex; }
  #face-graph, #face-today { overflow-y: auto; padding: var(--sp-5); }
  .face-inner { max-width: 960px; margin: 0 auto; width: 100%; display: flex; flex-direction: column; gap: var(--sp-5); }
  .card { background: var(--panel); border: 1px solid var(--rule); border-radius: var(--r-panel); padding: var(--sp-4) var(--sp-5); }
  .card h3 { margin: 0 0 var(--sp-2); font-size: var(--text-body); font-weight: 600; }
  .card h4 { margin: 0 0 var(--sp-2); font-size: var(--text-label); font-weight: 600; color: var(--faint); }
  .card p { margin: var(--sp-1) 0; line-height: 1.55; }
  .card ul { margin: var(--sp-2) 0; padding-left: var(--sp-4); line-height: 1.6; }
  .quiet { color: var(--muted); font-size: var(--text-dense); }
  .row-actions { display: flex; gap: var(--sp-2); margin-top: var(--sp-3); }
  .act-btn { border-radius: var(--r-card); padding: var(--sp-2) var(--sp-4); font-size: var(--text-small); font-weight: 600; cursor: pointer;
             border: 1px solid var(--rule); background: none; color: var(--ink); min-height: 36px; }
  .act-btn.primary { background: var(--accent-fill); border-color: var(--accent-fill); color: var(--accent-ink); }
  .act-btn.primary:hover { background: var(--accent-fill-hover); }
  .act-btn:hover { border-color: var(--muted); }
  .settled { color: var(--faint); font-size: var(--text-small); }
  #graph-svg { width: 100%; height: auto; display: block; }
  .graph-scroll { overflow-x: auto; }
  #graph-summary { font-variant-numeric: tabular-nums; }
  #graph-summary .fact { padding: 0 var(--sp-3); border-left: 1px solid var(--rule); white-space: nowrap; }
  #graph-summary .fact:first-child { padding-left: 0; border-left: 0; }
  .g-node rect { fill: var(--panel); stroke: var(--rule); }
  .g-node text { fill: var(--ink); font-size: var(--text-label); }
  .g-node .g-kind { fill: var(--faint); font-size: var(--text-meta); }
  .g-edge { stroke: var(--rule); stroke-width: 1.2; }
  .stream-item { border-left: 2px solid var(--rule); padding: 2px 0 var(--sp-4) var(--sp-4); margin-left: var(--sp-1); }
  .stream-item .s-kind { font-size: var(--text-meta); color: var(--muted); font-weight: 600; }
  .stream-item .s-title { font-size: var(--text-dense); margin: 2px 0; }
  .stream-item .s-detail { font-size: var(--text-small); color: var(--muted); line-height: 1.5; white-space: pre-wrap; }

  /* suite panel */
  #suite-panel { display: none; margin: 0 var(--sp-2); flex-direction: column; gap: var(--sp-3); }
  #suite-panel.open { display: flex; }
  .suite-item { background: var(--field); border: 1px solid var(--rule); border-radius: var(--r-card); padding: var(--sp-2) var(--sp-3); }
  .suite-item .si-name { font-size: var(--text-small); font-weight: 600; }
  .suite-item .si-kind { font-size: var(--text-meta); color: var(--muted); font-weight: 600; }
  .suite-item .si-desc { font-size: var(--text-label); color: var(--muted); line-height: 1.45; margin-top: 2px; }
  .suite-item .row-actions { margin-top: var(--sp-2); }
  .suite-item input { background: var(--bg); color: var(--ink); border: 1px solid var(--rule);
                      border-radius: var(--r-control); padding: var(--sp-2); font-size: var(--text-label); width: 100%; margin-top: var(--sp-2); min-height: 36px; }
  #no-key-banner { display: none; background: var(--warn-bg); color: var(--warn);
                   border-bottom: 1px solid var(--warn-rule); padding: var(--sp-3) var(--sp-5); font-size: var(--text-dense); }
  #thread { flex: 1; overflow-y: auto; padding: var(--sp-5) var(--sp-5) var(--sp-3); }
  #messages { max-width: 760px; margin: 0 auto; display: flex; flex-direction: column; gap: var(--sp-4); }
  .msg { border-radius: var(--r-panel); padding: var(--sp-3) var(--sp-4); font-size: var(--text-body); line-height: 1.55;
         white-space: pre-wrap; word-wrap: break-word; }
  .msg.user { background: var(--user-bubble); align-self: flex-end; max-width: 85%; }
  .msg.assistant { background: var(--panel); border: 1px solid var(--rule);
                   align-self: flex-start; max-width: 92%; }
  /* The signature: step rows are the work ledger — mono, on a rail,
     each tool call on the record under the answer it produced. */
  .msg.step { background: transparent; color: var(--muted); font-size: var(--text-small);
              font-family: var(--font-mono);
              align-self: flex-start; padding: 0 var(--sp-4) 0 var(--sp-3);
              border-left: 2px solid var(--rule); border-radius: 0; margin-left: var(--sp-2); }
  .msg.error { background: var(--err-bg); color: var(--err); align-self: flex-start; }
  .empty { color: var(--faint); text-align: center; margin-top: 12vh; font-size: var(--text-body); line-height: 1.6; }
  #composer { border-top: 1px solid var(--rule); background: var(--bg); padding: var(--sp-3) var(--sp-5) var(--sp-4); }
  #composer-inner { max-width: 760px; margin: 0 auto; display: flex; gap: var(--sp-2); align-items: flex-end; }
  #composer-input { flex: 1; resize: none; border: 1px solid var(--rule); border-radius: var(--r-panel);
                    padding: var(--sp-3); font-size: var(--text-body); font-family: inherit; min-height: 44px;
                    max-height: 160px; background: var(--field); color: var(--ink); }
  #send-btn { background: var(--ink); color: var(--bg); border: 0; border-radius: var(--r-card);
              padding: var(--sp-3) var(--sp-5); font-size: var(--text-dense); font-weight: 600; cursor: pointer; min-height: 44px; }
  #send-btn:disabled { opacity: 0.45; cursor: default; }

  @media (max-width: 768px) {
    body { flex-direction: column; }
    /* The face bar wraps instead of overflowing narrow screens. */
    #face-bar { flex-wrap: wrap; padding: var(--sp-2) var(--sp-3) 0; }
    /* The graph re-lays itself out compactly at narrow widths
       (see loadGraph) rather than shrinking past readability. */
    #graph-summary .fact { white-space: normal; }
    #face-graph, #face-today { padding: var(--sp-4) var(--sp-3); }
    /* The rail collapses to a compact head block — never a tall
       slab of navigation above the work. */
    #sidebar { width: 100%; flex: none; border-right: 0;
               border-bottom: 1px solid var(--rule); gap: var(--sp-2); }
    #sidebar .tagline { display: none; }
    #thread-list { flex: none; max-height: 22vh; }
    #settings { flex-direction: row; flex-wrap: wrap; align-items: center; }
    /* Touch targets reach the 44px floor on small screens. */
    .theme-btn, .act-btn, .side-btn, .face-btn, .thread-btn,
    #person-select, #person-form input, #person-form button,
    .suite-item input { min-height: 44px; }
  }
  @media (prefers-reduced-motion: reduce) {
    *, *::before, *::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; scroll-behavior: auto !important; }
  }
</style>
</head>
<body>
  <aside id="sidebar">
    <h1>Ergon</h1>
    <p class="tagline">An agent for every employee. Describe the work — it finishes it.</p>
    <div>
      <p class="side-label" id="person-label">Working for</p>
      <select id="person-select" aria-labelledby="person-label"></select>
    </div>
    <button id="new-chat-btn" type="button"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2.4" fill="none" stroke-linecap="round"/></svg>New chat</button>
    <ul id="thread-list"></ul>
    <p id="side-error" role="alert"></p>
    <div id="settings">
      <h2>Settings</h2>
      <div class="set-row">
        <span class="set-name">Appearance</span>
        <button class="theme-btn" type="button" data-set-theme="dark" aria-pressed="true">Dark</button>
        <button class="theme-btn" type="button" data-set-theme="light" aria-pressed="false">Light</button>
      </div>
      <button class="side-btn" id="memory-btn" type="button" aria-expanded="false">What my agent has learned</button>
      <div id="memory-panel" role="region" aria-label="Agent memory"></div>
      <button class="side-btn" id="suite-btn" type="button" aria-expanded="false">My suite &amp; the library</button>
      <div id="suite-panel" role="region" aria-label="Suite and library"></div>
      <button class="side-btn" id="add-person-btn" type="button" aria-expanded="false">Add a person</button>
      <form id="person-form">
        <label class="sr-only" for="person-name">Name</label>
        <input id="person-name" type="text" placeholder="Name" autocomplete="off" maxlength="80">
        <label class="sr-only" for="person-role">Role</label>
        <input id="person-role" type="text" placeholder="Role (optional)" autocomplete="off" maxlength="120">
        <p id="person-error" role="alert" style="display:none; color: var(--err); font-size: var(--text-label); margin: 0;"></p>
        <button type="submit">Add person</button>
      </form>
    </div>
  </aside>
  <main>
    <div id="face-bar" role="tablist" aria-label="Views">
      <button class="face-btn" type="button" role="tab" data-face="graph" aria-selected="true"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="5" cy="12" r="2.4" fill="currentColor"/><circle cx="19" cy="5.5" r="2.4" fill="currentColor"/><circle cx="19" cy="18.5" r="2.4" fill="currentColor"/><path d="M7.2 10.8 16.8 6.6M7.2 13.2l9.6 4.2" stroke="currentColor" stroke-width="1.8"/></svg>Work Graph</button>
      <button class="face-btn" type="button" role="tab" data-face="chat" aria-selected="false"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 6h16M4 12h16M4 18h9" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>Conversation Spine</button>
      <button class="face-btn" type="button" role="tab" data-face="today" aria-selected="false"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8.5" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 7.5V12l3 2" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>Today River<span class="face-badge" id="badge-today"></span></button>
    </div>

    <section id="face-graph" class="face on" aria-label="Work graph">
      <div class="face-inner">
        <div class="card">
          <h3>The work, as a graph</h3>
          <p class="quiet" id="graph-summary">Loading…</p>
          <div class="graph-scroll"><svg id="graph-svg" role="img" aria-label="Graph of people, their finished work, the files it produced and the suite items they carry"></svg></div>
          <p class="quiet" id="graph-empty" style="display:none">No finished work yet — the graph fills in as agents finish jobs.</p>
        </div>
      </div>
    </section>

    <div id="face-chat" class="face">
      <div id="no-key-banner">Ergon is not connected to a model yet, so the agent cannot run. The person who set Ergon up needs to add the model key (ERGON_MODEL_API_KEY) on the server. Nothing here is simulated: without a key, Ergon will not invent answers — and anything you type is kept in the thread, not lost.</div>
      <div id="thread" aria-live="polite">
        <div id="messages">
          <div class="empty">What should your agent finish for you?<br><span id="empty-sub">Give it a task in plain words — it works step by step, right here, until the job is done.</span></div>
        </div>
      </div>
      <form id="composer">
        <div id="composer-inner">
          <label class="sr-only" for="composer-input">Describe a task for your agent</label>
          <textarea id="composer-input" rows="1" placeholder="Describe a task — e.g. organise my notes into a clean summary document"></textarea>
          <button id="send-btn" type="submit">Send</button>
        </div>
      </form>
    </div>

    <section id="face-today" class="face" aria-label="Today">
      <div class="face-inner">
        <div class="card" id="brief-card">
          <h4>Morning brief</h4>
          <h3 id="brief-headline">Loading…</h3>
          <div id="brief-body"></div>
        </div>
        <div class="card" id="decisions-card">
          <h4>Decisions waiting on you</h4>
          <div id="decisions-list"><p class="quiet">Loading…</p></div>
        </div>
        <div class="card" id="offers-card">
          <h4>Offers from your agent</h4>
          <div id="offers-list"><p class="quiet">Loading…</p></div>
        </div>
        <div class="card">
          <h4>Today's stream</h4>
          <div id="today-stream"><p class="quiet">Loading…</p></div>
        </div>
      </div>
    </section>
  </main>
<script>
  let currentThread = null;
  let busy = false;
  let modelConfigured = true;
  const root = document.documentElement;
  const messagesEl = document.getElementById('messages');
  const listEl = document.getElementById('thread-list');
  const inputEl = document.getElementById('composer-input');
  const sendBtn = document.getElementById('send-btn');
  const personSelect = document.getElementById('person-select');

  function addMsg(role, content) {
    const empty = messagesEl.querySelector('.empty');
    if (empty) empty.remove();
    const div = document.createElement('div');
    div.className = 'msg ' + role;
    div.textContent = content;
    messagesEl.appendChild(div);
    document.getElementById('thread').scrollTop = 1e9;
    return div;
  }

  /* appearance — the settings toggle; the choice persists on this device */
  const themeBtns = document.querySelectorAll('.theme-btn');
  function syncThemeBtns() {
    const cur = root.getAttribute('data-theme') === 'light' ? 'light' : 'dark';
    themeBtns.forEach((b) => b.setAttribute('aria-pressed', b.getAttribute('data-set-theme') === cur ? 'true' : 'false'));
  }
  themeBtns.forEach((b) => b.addEventListener('click', () => {
    const next = b.getAttribute('data-set-theme');
    root.setAttribute('data-theme', next);
    try { localStorage.setItem('ergon-theme', next); } catch (e) {}
    syncThemeBtns();
  }));
  syncThemeBtns();

  /* One empty state, worded once, used by every path that shows it.
     In the no-key state it must not promise work that cannot run. */
  function emptyHtml() {
    const sub = modelConfigured
      ? 'Give it a task in plain words — it works step by step, right here, until the job is done.'
      : 'Ergon is not connected to a model yet (see the note above). Anything you type is kept in the thread.';
    return '<div class="empty">What should your agent finish for you?<br>' + sub + '</div>';
  }

  function sideError(msg) {
    const el = document.getElementById('side-error');
    el.textContent = msg || '';
    el.style.display = msg ? 'block' : 'none';
  }

  async function loadStatus() {
    try {
      const r = await fetch('/api/status');
      const s = await r.json();
      modelConfigured = !!s.model_configured;
      if (!modelConfigured) {
        document.getElementById('no-key-banner').style.display = 'block';
        const sub = document.getElementById('empty-sub');
        if (sub) sub.textContent = 'Ergon is not connected to a model yet (see the note above). Anything you type is kept in the thread.';
      }
      if (s.person && s.person.name) {
        document.getElementById('person-label').textContent = 'Working for — ' + s.person.name;
      }
    } catch (e) { /* status is best-effort */ }
  }

  async function loadPeople() {
    try {
      const r = await fetch('/api/people');
      if (!r.ok) throw new Error('people');
      const data = await r.json();
      personSelect.innerHTML = '';
      for (const p of data.people) {
        const opt = document.createElement('option');
        opt.value = p.id;
        opt.textContent = p.role ? p.name + ' — ' + p.role : p.name;
        if (p.id === data.active) opt.selected = true;
        personSelect.appendChild(opt);
      }
      sideError('');
    } catch (e) {
      sideError('Could not load the people list — is the Ergon server running?');
    }
  }

  personSelect.addEventListener('change', async () => {
    await fetch('/api/active-person', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: personSelect.value })
    });
    currentThread = null;
    messagesEl.innerHTML = emptyHtml();
    /* Close the memory panel fully: class, ARIA state and the previous
       person's text all reset, so nothing stale is announced or shown. */
    const panel = document.getElementById('memory-panel');
    panel.classList.remove('open');
    panel.textContent = '';
    document.getElementById('memory-btn').setAttribute('aria-expanded', 'false');
    updateTodayBadge.dec = 0;
    updateTodayBadge.off = 0;
    updateTodayBadge(null, null);
    await loadThreads();
    await loadStatus();
    await loadSuite();
    if (currentFace === 'graph') loadGraph();
    if (currentFace === 'today') loadToday();
  });

  document.getElementById('add-person-btn').addEventListener('click', (ev) => {
    const form = document.getElementById('person-form');
    const open = form.classList.toggle('open');
    ev.currentTarget.setAttribute('aria-expanded', open ? 'true' : 'false');
    if (open) document.getElementById('person-name').focus();
  });

  document.getElementById('person-form').addEventListener('submit', async (ev) => {
    ev.preventDefault();
    const errEl = document.getElementById('person-error');
    const showErr = (msg) => { errEl.textContent = msg; errEl.style.display = msg ? 'block' : 'none'; };
    const name = document.getElementById('person-name').value.trim();
    const role = document.getElementById('person-role').value.trim();
    if (!name) { showErr('Give the person a name first.'); return; }
    let person;
    try {
      const r = await fetch('/api/people', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: name, role: role })
      });
      person = await r.json();
      if (!r.ok) { showErr(person.error || 'That person could not be added.'); return; }
    } catch (e) {
      showErr('Could not reach the Ergon server.');
      return;
    }
    showErr('');
    await fetch('/api/active-person', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: person.id })
    });
    document.getElementById('person-name').value = '';
    document.getElementById('person-role').value = '';
    document.getElementById('person-form').classList.remove('open');
    document.getElementById('add-person-btn').setAttribute('aria-expanded', 'false');
    currentThread = null;
    messagesEl.innerHTML = emptyHtml();
    /* Same full reset as the person-select handler: switching the
       active person must not leave the previous person's memory,
       suite or badges on screen. */
    const panel = document.getElementById('memory-panel');
    panel.classList.remove('open');
    panel.textContent = '';
    document.getElementById('memory-btn').setAttribute('aria-expanded', 'false');
    updateTodayBadge.dec = 0;
    updateTodayBadge.off = 0;
    updateTodayBadge(null, null);
    await loadPeople();
    await loadThreads();
    await loadStatus();
    await loadSuite();
    if (currentFace === 'graph') loadGraph();
    if (currentFace === 'today') loadToday();
  });

  document.getElementById('memory-btn').addEventListener('click', async (ev) => {
    /* Capture the button now: after an await, ev.currentTarget is
       gone, and the aria state must still flip. */
    const btn = ev.currentTarget;
    const panel = document.getElementById('memory-panel');
    if (panel.classList.contains('open')) {
      panel.classList.remove('open');
      btn.setAttribute('aria-expanded', 'false');
      return;
    }
    let data;
    try {
      const r = await fetch('/api/memory');
      if (!r.ok) throw new Error('memory');
      data = await r.json();
    } catch (e) {
      panel.textContent = 'Memory could not be loaded — is the Ergon server running?';
      panel.classList.add('open');
      btn.setAttribute('aria-expanded', 'true');
      return;
    }
    const content = (data.content || '').trim();
    /* The scaffold alone ("# Memory", or About with nothing learned
       yet) reads as a raw file dump; say plainly that there is
       nothing learned yet. Otherwise what the agent has learned is
       DRAWN as grouped facts — sections and their entries — with
       the verbatim file one tap away behind a disclosure, never
       replaced by the drawing. */
    const learned = content.split('\n').some((line) => {
      const t = line.trim();
      return t.startsWith('- ') && !t.startsWith('- Name:') && !t.startsWith('- Role:');
    });
    if (!learned) {
      panel.textContent = 'Nothing learned yet. Preferences and facts your agent picks up as it works will appear here.';
    } else {
      let html = '';
      let inList = false;
      for (const raw of content.split('\n')) {
        const line = raw.trim();
        if (line.startsWith('## ')) {
          if (inList) { html += '</ul>'; inList = false; }
          html += '<div class="mem-head">' + esc(line.slice(3)) + '</div>';
        } else if (line.startsWith('- ')) {
          if (!inList) { html += '<ul class="mem-list">'; inList = true; }
          html += '<li>' + esc(line.slice(2)) + '</li>';
        } else if (line && !line.startsWith('# ')) {
          if (inList) { html += '</ul>'; inList = false; }
          html += '<p>' + esc(line) + '</p>';
        }
      }
      if (inList) html += '</ul>';
      html += '<details class="mem-file"><summary>The memory file itself</summary><pre>' + esc(content) + '</pre></details>';
      panel.innerHTML = html;
    }
    panel.classList.add('open');
    btn.setAttribute('aria-expanded', 'true');
  });

  async function loadThreads() {
    try {
      const r = await fetch('/api/threads');
      if (!r.ok) throw new Error('threads');
      const data = await r.json();
      listEl.innerHTML = '';
      for (const t of data.threads) {
        const li = document.createElement('li');
        /* A real <button>: keyboard-reachable and focusable, unlike a
           clickable <li>. The sub-line (message count) tells threads
           with the same title apart. */
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'thread-btn' + (currentThread && t.id === currentThread ? ' active' : '');
        const title = document.createElement('span');
        title.className = 't-title';
        title.textContent = t.title || 'Untitled task';
        btn.appendChild(title);
        if (typeof t.messages === 'number') {
          const sub = document.createElement('span');
          sub.className = 't-sub';
          sub.textContent = t.messages === 1 ? '1 message' : t.messages + ' messages';
          btn.appendChild(sub);
        }
        btn.addEventListener('click', () => openThread(t.id));
        li.appendChild(btn);
        listEl.appendChild(li);
      }
      sideError('');
    } catch (e) {
      sideError('Could not load the conversation list — is the Ergon server running?');
    }
  }

  async function newThread() {
    const r = await fetch('/api/threads', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
    const t = await r.json();
    currentThread = t.id;
    messagesEl.innerHTML = emptyHtml();
    await loadThreads();
    inputEl.focus();
  }

  async function openThread(id) {
    let t;
    try {
      const r = await fetch('/api/threads/' + encodeURIComponent(id));
      if (!r.ok) throw new Error('open');
      t = await r.json();
    } catch (e) {
      /* A stale id must not be dressed up as an empty thread. */
      messagesEl.innerHTML = '';
      addMsg('error', 'That conversation could not be opened — it may have been removed.');
      return;
    }
    currentThread = id;
    messagesEl.innerHTML = '';
    for (const m of (t.messages || [])) addMsg(m.role, m.content);
    if (!messagesEl.children.length) messagesEl.innerHTML = emptyHtml();
    await loadThreads();
  }

  document.getElementById('new-chat-btn').onclick = newThread;

  document.getElementById('composer').addEventListener('submit', async (ev) => {
    ev.preventDefault();
    /* Busy guard: while a run is in flight, further submits (Enter
       included — the disabled Send button alone does not stop the
       keyboard path) are ignored, so runs never overlap on a thread. */
    if (busy) return;
    const text = inputEl.value.trim();
    if (!text) return;
    busy = true;
    if (!currentThread) await newThread();
    inputEl.value = '';
    addMsg('user', text);
    sendBtn.disabled = true;
    inputEl.disabled = true;
    sendBtn.textContent = 'Working…';
    const working = addMsg('step', 'Agent is working…');
    try {
      const r = await fetch('/api/chat', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ thread_id: currentThread, message: text })
      });
      const data = await r.json();
      working.remove();
      if (!r.ok) {
        addMsg('error', data.error || 'The agent could not run.');
      } else {
        messagesEl.innerHTML = '';
        for (const m of data.messages) addMsg(m.role, m.content);
      }
    } catch (e) {
      working.remove();
      addMsg('error', 'Could not reach the Ergon server.');
    }
    busy = false;
    sendBtn.disabled = false;
    inputEl.disabled = false;
    sendBtn.textContent = 'Send';
    await loadThreads();
    inputEl.focus();
  });

  inputEl.addEventListener('keydown', (ev) => {
    if (ev.key === 'Enter' && !ev.shiftKey) {
      ev.preventDefault();
      document.getElementById('composer').requestSubmit();
    }
  });

  /* ---------------- faces: three views over the same core -------- */
  const faceBtns = document.querySelectorAll('.face-btn');
  let currentFace = 'graph';
  try {
    const saved = localStorage.getItem('ergon-face');
    if (saved === 'graph' || saved === 'chat' || saved === 'today') currentFace = saved;
  } catch (e) { /* default stays graph */ }

  function showFace(name) {
    currentFace = name;
    try { localStorage.setItem('ergon-face', name); } catch (e) {}
    faceBtns.forEach((b) => b.setAttribute('aria-selected', b.getAttribute('data-face') === name ? 'true' : 'false'));
    for (const f of ['graph', 'chat', 'today']) {
      document.getElementById('face-' + f).classList.toggle('on', f === name);
    }
    if (name === 'graph') loadGraph();
    if (name === 'today') loadToday();
  }
  faceBtns.forEach((b) => b.addEventListener('click', () => showFace(b.getAttribute('data-face'))));

  function esc(s) {
    const d = document.createElement('div');
    d.textContent = s == null ? '' : String(s);
    return d.innerHTML;
  }
  /* esc() escapes for element content; attribute values also need
     the quote escaped, or a crafted field name from the library
     breaks out of the attribute. */
  function escAttr(s) {
    return esc(s).replace(/"/g, '&quot;');
  }

  /* ---------------- work graph ------------------------------------ */
  async function loadGraph() {
    const svg = document.getElementById('graph-svg');
    const summary = document.getElementById('graph-summary');
    const empty = document.getElementById('graph-empty');
    let data;
    try {
      const r = await fetch('/api/graph');
      if (!r.ok) throw new Error('graph');
      data = await r.json();
    } catch (e) {
      summary.textContent = 'The graph could not be loaded — is the Ergon server running?';
      return;
    }
    const nodes = data.nodes || [];
    const edges = data.edges || [];
    const workCount = nodes.filter((n) => n.kind === 'work').length;
    /* Facts get their own segments, separated by hairlines — never
       a middle-dot meta string (R8). */
    if (workCount) {
      summary.innerHTML = '';
      const facts = [
        nodes.filter((n) => n.kind === 'person').length + ' people',
        workCount + ' finished jobs',
        nodes.filter((n) => n.kind === 'file').length + ' files produced',
        nodes.filter((n) => n.kind === 'item').length + ' suite items in play',
      ];
      for (const f of facts) {
        const s = document.createElement('span');
        s.className = 'fact';
        s.textContent = f;
        summary.appendChild(s);
      }
    } else {
      summary.textContent = 'People, the work their agents finish, the files it produces, and the suite each person carries.';
    }
    empty.style.display = workCount ? 'none' : 'block';
    /* Layout by kind: person → suite item → work → file. Wide
       screens get four columns; narrow ones get a 2×2 arrangement
       at a readable scale instead of the four columns shrunk past
       legibility. */
    const compact = svg.clientWidth > 0 && svg.clientWidth < 560;
    const pos = {};
    let height;
    if (compact) {
      const colX = [92, 268];
      const cell = { person: [0, 0], item: [1, 0], work: [0, 1], file: [1, 1] };
      const counts = { person: 0, item: 0, work: 0, file: 0 };
      for (const n of nodes) { if (counts[n.kind] != null) counts[n.kind]++; }
      const row0H = Math.max(counts.person, counts.item, 1) * 52;
      const seen = { person: 0, item: 0, work: 0, file: 0 };
      for (const n of nodes) {
        const kind = cell[n.kind] ? n.kind : 'work';
        const row = seen[kind]++;
        pos[n.id] = {
          x: colX[cell[kind][0]],
          y: 34 + (cell[kind][1] ? row0H + 12 : 0) + row * 52,
          node: n,
        };
      }
      height = Math.max(120, 34 + row0H + 12 + Math.max(counts.work, counts.file, 1) * 52);
    } else {
      const cols = { person: 0, item: 1, work: 2, file: 3 };
      const colX = [70, 300, 530, 760];
      const perCol = { person: 0, item: 0, work: 0, file: 0 };
      for (const n of nodes) {
        const c = cols[n.kind] != null ? cols[n.kind] : 2;
        const row = perCol[n.kind] != null ? perCol[n.kind]++ : 0;
        pos[n.id] = { x: colX[c], y: 34 + row * 52, node: n };
      }
      height = Math.max(120, 34 + Math.max(1, ...Object.values(perCol)) * 52);
    }
    const NS = 'http://www.w3.org/2000/svg';
    svg.setAttribute('viewBox', '0 0 ' + (compact ? 360 : 920) + ' ' + height);
    svg.innerHTML = '';
    for (const e of edges) {
      const a = pos[e.from], b = pos[e.to];
      if (!a || !b) continue;
      const line = document.createElementNS(NS, 'line');
      if (compact) {
        line.setAttribute('x1', a.x); line.setAttribute('y1', a.y);
        line.setAttribute('x2', b.x); line.setAttribute('y2', b.y);
      } else {
        line.setAttribute('x1', a.x + 62); line.setAttribute('y1', a.y);
        line.setAttribute('x2', b.x - 62); line.setAttribute('y2', b.y);
      }
      line.setAttribute('class', 'g-edge');
      svg.appendChild(line);
    }
    for (const id of Object.keys(pos)) {
      const p = pos[id];
      const g = document.createElementNS(NS, 'g');
      g.setAttribute('class', 'g-node');
      const rect = document.createElementNS(NS, 'rect');
      rect.setAttribute('x', p.x - 62); rect.setAttribute('y', p.y - 16);
      rect.setAttribute('width', 124); rect.setAttribute('height', 32); rect.setAttribute('rx', 8);
      g.appendChild(rect);
      const kind = document.createElementNS(NS, 'text');
      kind.setAttribute('x', p.x); kind.setAttribute('y', p.y - 1);
      kind.setAttribute('text-anchor', 'middle'); kind.setAttribute('class', 'g-kind');
      kind.textContent = p.node.kind;
      g.appendChild(kind);
      const label = document.createElementNS(NS, 'text');
      label.setAttribute('x', p.x); label.setAttribute('y', p.y + 12);
      label.setAttribute('text-anchor', 'middle');
      const raw = p.node.label || '';
      label.textContent = raw.length > 18 ? raw.slice(0, 17) + '…' : raw;
      const title = document.createElementNS(NS, 'title');
      title.textContent = raw + (p.node.detail ? ' — ' + p.node.detail : '');
      g.appendChild(title);
      g.appendChild(label);
      svg.appendChild(g);
    }
  }

  /* ---------------- today: brief, decisions, offers, stream ------- */
  async function loadToday() {
    loadBrief();
    loadDecisions();
    loadOffers();
    loadStream();
  }

  async function loadBrief() {
    const head = document.getElementById('brief-headline');
    const bodyEl = document.getElementById('brief-body');
    try {
      const r = await fetch('/api/brief');
      if (!r.ok) throw new Error('brief');
      const b = await r.json();
      head.textContent = b.headline || '';
      let html = '';
      if ((b.finished || []).length) {
        html += '<h4>Finished recently</h4><ul>' + b.finished.map((f) => '<li>' + esc(f.task) + ' <span class="settled">(' + esc(f.date) + ')</span></li>').join('') + '</ul>';
      }
      if ((b.learned_recently || []).length) {
        html += '<h4>Learned recently</h4><ul>' + b.learned_recently.map((l) => '<li>' + esc(l) + '</li>').join('') + '</ul>';
      }
      /* Facts on their own lines — never a middle-dot meta string
         (R8). */
      html += '<p class="quiet">Your suite carries ' + (b.suite_size || 0) + ' items.</p>' +
        '<p class="quiet">The company library holds ' + (b.library_size || 0) + ' items.</p>' +
        '<p class="quiet">' + (b.pending_decisions || []).length + ' decisions and ' + (b.pending_offers || []).length + ' offers are waiting below.</p>';
      bodyEl.innerHTML = html;
    } catch (e) {
      head.textContent = 'The brief could not be loaded.';
      bodyEl.innerHTML = '';
    }
  }

  async function loadDecisions() {
    const el = document.getElementById('decisions-list');
    try {
      const r = await fetch('/api/decisions');
      if (!r.ok) throw new Error('decisions');
      const data = await r.json();
      const list = (data.decisions || []).slice().reverse();
      const pending = list.filter((d) => d.status === 'pending');
      const badge = document.getElementById('badge-today');
      updateTodayBadge(pending.length, null);
      if (!list.length) { el.innerHTML = '<p class="quiet">Nothing waiting. Consequential actions — like deleting a file — land here for a quick yes or no; everything else your agent just finishes.</p>'; return; }
      el.innerHTML = '';
      for (const d of list) {
        const div = document.createElement('div');
        div.className = 'suite-item';
        let inner = '<div class="si-kind">' + esc(d.kind) + '</div><div class="si-name">' + esc(d.summary) + '</div>';
        if (d.status === 'pending') {
          inner += '<div class="row-actions"><button class="act-btn primary" data-decide="' + esc(d.id) + '" data-how="approve">Approve</button>' +
                   '<button class="act-btn" data-decide="' + esc(d.id) + '" data-how="deny">Deny</button></div>';
        } else {
          inner += '<div class="settled">' + esc(d.status) + (d.outcome ? ' — ' + esc(d.outcome) : '') + '</div>';
        }
        div.innerHTML = inner;
        el.appendChild(div);
      }
      el.querySelectorAll('[data-decide]').forEach((btn) => btn.addEventListener('click', async () => {
        btn.disabled = true;
        await fetch('/api/decisions/' + encodeURIComponent(btn.getAttribute('data-decide')) + '/' + btn.getAttribute('data-how'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
        await loadDecisions();
        await loadStream();
        await loadThreads();
      }));
    } catch (e) {
      el.innerHTML = '<p class="quiet">Decisions could not be loaded.</p>';
    }
  }

  let lastOfferPending = null;
  function updateTodayBadge(decisions, offers) {
    if (decisions != null) updateTodayBadge.dec = decisions;
    if (offers != null) updateTodayBadge.off = offers;
    const total = (updateTodayBadge.dec || 0) + (updateTodayBadge.off || 0);
    const badge = document.getElementById('badge-today');
    badge.textContent = total;
    badge.classList.toggle('on', total > 0);
  }

  async function loadOffers() {
    const el = document.getElementById('offers-list');
    try {
      const r = await fetch('/api/offers');
      if (!r.ok) throw new Error('offers');
      const data = await r.json();
      const list = (data.offers || []).slice().reverse();
      updateTodayBadge(null, list.filter((o) => o.status === 'pending').length);
      if (!list.length) { el.innerHTML = '<p class="quiet">No offers right now. When your agent notices work you keep asking for, it offers to take it over — here.</p>'; return; }
      el.innerHTML = '';
      for (const o of list) {
        const div = document.createElement('div');
        div.className = 'suite-item';
        let inner = '<div class="si-name">' + esc(o.title) + '</div><div class="si-desc">' + esc(o.detail) + '</div>';
        if (o.status === 'pending') {
          inner += '<div class="row-actions"><button class="act-btn primary" data-offer="' + esc(o.id) + '" data-how="accept">Yes, do it</button>' +
                   '<button class="act-btn" data-offer="' + esc(o.id) + '" data-how="dismiss">No thanks</button></div>';
        } else {
          inner += '<div class="settled">' + esc(o.status) + '</div>';
        }
        div.innerHTML = inner;
        el.appendChild(div);
      }
      el.querySelectorAll('[data-offer]').forEach((btn) => btn.addEventListener('click', async () => {
        btn.disabled = true;
        await fetch('/api/offers/' + encodeURIComponent(btn.getAttribute('data-offer')) + '/' + btn.getAttribute('data-how'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
        await loadOffers();
        await loadSuite();
      }));
    } catch (e) {
      el.innerHTML = '<p class="quiet">Offers could not be loaded.</p>';
    }
  }

  async function loadStream() {
    const el = document.getElementById('today-stream');
    try {
      const r = await fetch('/api/today');
      if (!r.ok) throw new Error('today');
      const data = await r.json();
      const events = data.events || [];
      if (!events.length) { el.innerHTML = '<p class="quiet">Nothing yet today.</p>'; return; }
      el.innerHTML = events.map((e) =>
        '<div class="stream-item"><div class="s-kind">' + esc(e.kind.replace('_', ' ')) + '</div>' +
        '<div class="s-title">' + esc(e.title) + '</div>' +
        (e.detail ? '<div class="s-detail">' + esc(e.detail) + '</div>' : '') + '</div>'
      ).join('');
    } catch (e) {
      el.innerHTML = '<p class="quiet">The stream could not be loaded.</p>';
    }
  }

  /* ---------------- suite & library ------------------------------- */
  document.getElementById('suite-btn').addEventListener('click', async (ev) => {
    const btn = ev.currentTarget;
    const panel = document.getElementById('suite-panel');
    if (panel.classList.contains('open')) {
      panel.classList.remove('open');
      btn.setAttribute('aria-expanded', 'false');
      return;
    }
    await loadSuite();
    panel.classList.add('open');
    btn.setAttribute('aria-expanded', 'true');
  });

  async function loadSuite() {
    const panel = document.getElementById('suite-panel');
    let suite, library;
    try {
      const [rs, rl] = await Promise.all([fetch('/api/suite'), fetch('/api/library')]);
      suite = await rs.json();
      library = await rl.json();
    } catch (e) {
      panel.innerHTML = '<p class="quiet">The suite could not be loaded.</p>';
      return;
    }
    const installed = suite.installed || [];
    const installedIds = installed.map((i) => i.id);
    let html = '<p class="side-label" style="margin:0">In your suite</p>';
    if (!installed.length) html += '<p class="quiet">Nothing yet.</p>';
    for (const item of installed) {
      html += '<div class="suite-item"><span class="si-kind">' + esc(item.kind) + '</span>' +
        '<div class="si-name">' + esc(item.name) + '</div><div class="si-desc">' + esc(item.description) + '</div>';
      if (item.kind === 'app') {
        for (const f of (item.fields || [])) {
          html += '<label class="sr-only" for="appfield-' + esc(item.id) + '-' + esc(f) + '">' + esc(f) + '</label>' +
                  '<input id="appfield-' + escAttr(item.id) + '-' + escAttr(f) + '" data-app-field="' + escAttr(f) + '" type="text" placeholder="' + escAttr(f) + '">';
        }
        html += '<div class="row-actions"><button class="act-btn primary" data-run-app="' + esc(item.id) + '">Run</button>' +
                '<button class="act-btn" data-remove="' + esc(item.id) + '">Remove</button></div>';
      } else {
        html += '<div class="row-actions"><button class="act-btn" data-remove="' + esc(item.id) + '">Remove</button></div>';
      }
      html += '</div>';
    }
    const available = (library.items || []).filter((i) => !i.builtin && !installedIds.includes(i.id));
    html += '<p class="side-label" style="margin:8px 0 0">From the company library</p>';
    if (!available.length) html += '<p class="quiet">Everything in the library is already in your suite.</p>';
    for (const item of available) {
      html += '<div class="suite-item"><span class="si-kind">' + esc(item.kind) + '</span>' +
        '<div class="si-name">' + esc(item.name) + '</div><div class="si-desc">' + esc(item.description) +
        ' <span class="settled">by ' + esc(item.author) + '</span></div>' +
        '<div class="row-actions"><button class="act-btn" data-install="' + esc(item.id) + '">Add to my suite</button></div></div>';
    }
    panel.innerHTML = html;
    panel.querySelectorAll('[data-install]').forEach((btn) => btn.addEventListener('click', async () => {
      btn.disabled = true;
      await fetch('/api/suite/install', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ item_id: btn.getAttribute('data-install') }) });
      await loadSuite();
    }));
    panel.querySelectorAll('[data-remove]').forEach((btn) => btn.addEventListener('click', async () => {
      btn.disabled = true;
      await fetch('/api/suite/remove', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ item_id: btn.getAttribute('data-remove') }) });
      await loadSuite();
    }));
    panel.querySelectorAll('[data-run-app]').forEach((btn) => btn.addEventListener('click', async () => {
      const card = btn.closest('.suite-item');
      const fields = {};
      card.querySelectorAll('[data-app-field]').forEach((inp) => { fields[inp.getAttribute('data-app-field')] = inp.value; });
      btn.disabled = true;
      btn.textContent = 'Running…';
      try {
        const r = await fetch('/api/apps/' + encodeURIComponent(btn.getAttribute('data-run-app')) + '/run', {
          method: 'POST', headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ fields: fields })
        });
        const data = await r.json();
        if (!r.ok) { sideError(data.error || 'That app could not run.'); btn.disabled = false; btn.textContent = 'Run'; return; }
        currentThread = data.id;
        showFace('chat');
        messagesEl.innerHTML = '';
        for (const m of (data.messages || [])) addMsg(m.role, m.content);
        await loadThreads();
      } catch (e) {
        sideError('Could not reach the Ergon server.');
        btn.disabled = false;
        btn.textContent = 'Run';
      }
    }));
  }

  loadStatus();
  loadPeople();
  loadThreads();
  showFace(currentFace);
</script>
</body>
</html>
"##;

/*
 * hallmark-rune override log — Ergon (kept Reflex items, on the record)
 *
 * override · R9 · guard: the sidebar is a thread list, and the list IS
 *   the work (the category pattern the objective mandates) · reason:
 *   the rail is earned here; on small widths it collapses to a compact
 *   head block rather than being removed.
 * override · R1 · guard: step rows and the memory file view render in
 *   mono · reason: the ledger IS data returned by the API (tool calls
 *   with real targets, the verbatim memory file); every UI label,
 *   heading and button stays in the body face.
 * refused · R5 · tracked-caps kind labels ("WORK", "SKILL") read as
 *   emphasis: hierarchy comes from size, weight and position — the
 *   labels are sentence case, untracked, one step smaller.
 * refused · R4 · pill theme buttons: the theme buttons are discrete
 *   bordered buttons at the control radius; pills remain only on
 *   count badges, which are chips by job.
 * refused · R3 · kind labels in the accent: a category word is not
 *   action, selection or live state, so kind labels (suite, stream,
 *   phone items) render in the muted ink, not the accent.
 */
