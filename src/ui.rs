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
  :root {
    color-scheme: dark;
    --bg: #171512; --panel: #201d18; --ink: #f0ede6;
    --muted: #a8a294; --faint: #8a8474; --rule: #35312a;
    --accent: #e08a67; --accent-bg: #b1572f; --accent-bg-hover: #9d4c27;
    --user-bubble: #2b2926; --warn-bg: #2e2410; --warn-ink: #eec27f; --warn-rule: #57431c;
    --err-bg: #331a1a; --err-ink: #f0a3a3; --field: #1b1915;
  }
  [data-theme="light"] {
    color-scheme: light;
    --bg: #f7f6f2; --panel: #ffffff; --ink: #201d18;
    --muted: #6d675c; --faint: #6e6759; --rule: #ddd8ce;
    --accent: #a34a28; --accent-bg: #a34a28; --accent-bg-hover: #8d3f21;
    --user-bubble: #e9e5db; --warn-bg: #fdf3e3; --warn-ink: #7a4b12; --warn-rule: #ecd9ae;
    --err-bg: #fdecec; --err-ink: #8c1d1d; --field: #ffffff;
  }
  * { box-sizing: border-box; }
  html, body { height: 100%; margin: 0; }
  body { font-family: -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
         background: var(--bg); color: var(--ink); font-size: 15px;
         display: flex; flex-direction: column; }
  :focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  button { font: inherit; }
  header { padding: 14px 16px 10px; border-bottom: 1px solid var(--rule); }
  header h1 { font-size: 17px; margin: 0; font-weight: 600; }
  header p { margin: 2px 0 0; font-size: 12px; color: var(--muted); }
  #no-key-banner { display: none; background: var(--warn-bg); color: var(--warn-ink);
                   border-bottom: 1px solid var(--warn-rule); padding: 10px 16px; font-size: 13px; }
  main { flex: 1; overflow-y: auto; padding: 14px 16px 84px; }
  .tab { display: none; flex-direction: column; gap: 14px; }
  .tab.on { display: flex; }
  .card { background: var(--panel); border: 1px solid var(--rule); border-radius: 12px; padding: 14px 16px; }
  .card h3 { margin: 0 0 6px; font-size: 15px; }
  .card h4 { margin: 0 0 8px; font-size: 12px; font-weight: 600; color: var(--faint);
             text-transform: uppercase; letter-spacing: 0.08em; }
  .card p { margin: 4px 0; line-height: 1.55; }
  .card ul { margin: 6px 0; padding-left: 18px; line-height: 1.6; }
  .quiet { color: var(--muted); font-size: 14px; }
  .settled { color: var(--faint); font-size: 13px; }
  .row-actions { display: flex; gap: 8px; margin-top: 10px; }
  .act-btn { border-radius: 8px; padding: 8px 14px; font-size: 13px; font-weight: 600; cursor: pointer;
             border: 1px solid var(--rule); background: none; color: var(--ink); }
  .act-btn.primary { background: var(--accent-bg); border-color: var(--accent-bg); color: #fff; }
  .item { background: var(--field); border: 1px solid var(--rule); border-radius: 8px; padding: 10px 12px; margin-top: 8px; }
  .item .i-kind { font-size: 11px; color: var(--accent); text-transform: uppercase; letter-spacing: 0.06em; font-weight: 700; }
  .item .i-name { font-size: 14px; font-weight: 600; margin-top: 1px; }
  .item .i-desc { font-size: 13px; color: var(--muted); line-height: 1.45; margin-top: 2px; }
  #messages { display: flex; flex-direction: column; gap: 10px; }
  .msg { border-radius: 12px; padding: 10px 13px; font-size: 15px; line-height: 1.55;
         white-space: pre-wrap; word-wrap: break-word; }
  .msg.user { background: var(--user-bubble); align-self: flex-end; max-width: 88%; }
  .msg.assistant { background: var(--panel); border: 1px solid var(--rule); align-self: flex-start; max-width: 94%; }
  .msg.step { background: transparent; color: var(--muted); font-size: 12px;
              font-family: ui-monospace, Menlo, monospace; align-self: flex-start; padding: 0 4px; }
  .msg.error { background: var(--err-bg); color: var(--err-ink); align-self: flex-start; }
  .empty { color: var(--faint); text-align: center; margin-top: 8vh; line-height: 1.6; }
  #composer { display: flex; gap: 8px; margin-top: 12px; }
  #composer-input { flex: 1; resize: none; border: 1px solid var(--rule); border-radius: 12px;
                    padding: 11px; font-size: 15px; font-family: inherit; min-height: 44px;
                    background: var(--field); color: var(--ink); }
  #send-btn { background: var(--ink); color: var(--bg); border: 0; border-radius: 10px;
              padding: 11px 16px; font-size: 14px; font-weight: 600; cursor: pointer; }
  #send-btn:disabled { opacity: 0.45; cursor: default; }
  nav { position: fixed; left: 0; right: 0; bottom: 0; display: flex; background: var(--panel);
        border-top: 1px solid var(--rule); padding-bottom: env(safe-area-inset-bottom); }
  .tab-btn { flex: 1; background: none; border: 0; color: var(--muted); padding: 9px 2px 7px;
             font-size: 11px; font-weight: 600; cursor: pointer; display: flex; flex-direction: column;
             align-items: center; gap: 3px; position: relative; }
  .tab-btn svg { width: 20px; height: 20px; }
  .tab-btn[aria-selected="true"] { color: var(--accent); }
  .tab-badge { position: absolute; top: 4px; right: calc(50% - 20px); background: var(--accent-bg);
               color: #fff; border-radius: 999px; font-size: 10px; padding: 0 6px; font-weight: 700; display: none; }
  .tab-badge.on { display: block; }
  @media (prefers-reduced-motion: reduce) {
    * { scroll-behavior: auto !important; transition: none !important; }
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
  :root {
    color-scheme: dark;
    --bg: #171512; --panel: #201d18; --sidebar: #100f0d; --ink: #f0ede6;
    --muted: #a8a294; --faint: #8a8474; --rule: #35312a;
    --accent: #e08a67; --accent-bg: #b1572f; --accent-bg-hover: #9d4c27;
    --user-bubble: #2b2926; --warn-bg: #2e2410; --warn-ink: #eec27f; --warn-rule: #57431c;
    --err-bg: #331a1a; --err-ink: #f0a3a3; --field: #1b1915;
  }
  [data-theme="light"] {
    color-scheme: light;
    --bg: #f7f6f2; --panel: #ffffff; --sidebar: #edeae3; --ink: #201d18;
    --muted: #6d675c; --faint: #6e6759; --rule: #ddd8ce;
    --accent: #a34a28; --accent-bg: #a34a28; --accent-bg-hover: #8d3f21;
    --user-bubble: #e9e5db; --warn-bg: #fdf3e3; --warn-ink: #7a4b12; --warn-rule: #ecd9ae;
    --err-bg: #fdecec; --err-ink: #8c1d1d; --field: #ffffff;
  }
  * { box-sizing: border-box; }
  html, body { height: 100%; margin: 0; }
  body { font-family: -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
         display: flex; background: var(--bg); color: var(--ink); font-size: 15px; }
  ::selection { background: var(--accent-bg); color: #fff; }
  :focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  button { font: inherit; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden;
             clip: rect(0 0 0 0); white-space: nowrap; }

  #sidebar { width: 264px; flex: 0 0 264px; background: var(--sidebar); color: var(--ink);
             display: flex; flex-direction: column; padding: 16px 12px; gap: 12px;
             border-right: 1px solid var(--rule); min-height: 0; }
  #sidebar h1 { font-size: 17px; margin: 0 8px; font-weight: 600; letter-spacing: -0.01em; }
  #sidebar .tagline { font-size: 12px; color: var(--muted); margin: 0 8px; line-height: 1.5; }
  .side-label { font-size: 12px; color: var(--faint); margin: 0 8px 4px; }
  #person-select { width: 100%; background: var(--field); color: var(--ink);
                   border: 1px solid var(--rule); border-radius: 8px; padding: 8px;
                   font-size: 14px; }
  #new-chat-btn { display: flex; align-items: center; justify-content: center; gap: 8px;
                  background: var(--accent-bg); color: #fff; border: 0; border-radius: 8px;
                  padding: 10px 12px; font-size: 14px; font-weight: 600; cursor: pointer; }
  #new-chat-btn:hover { background: var(--accent-bg-hover); }
  #new-chat-btn svg { width: 14px; height: 14px; flex: none; }
  #thread-list { list-style: none; margin: 0; padding: 0; overflow-y: auto; flex: 1; min-height: 48px; }
  #thread-list li { margin: 0; }
  .thread-btn { display: block; width: 100%; text-align: left; background: none; border: 0;
                padding: 8px 10px; border-radius: 8px; font-size: 14px; cursor: pointer;
                color: var(--muted); }
  .thread-btn .t-title { display: block; white-space: nowrap; overflow: hidden;
                         text-overflow: ellipsis; }
  .thread-btn .t-sub { display: block; font-size: 12px; color: var(--faint); margin-top: 2px; }
  .thread-btn.active { background: var(--user-bubble); color: var(--ink); }
  .thread-btn:hover { color: var(--ink); }
  #side-error { display: none; color: var(--err-ink); font-size: 12px; margin: 0 8px; }

  #settings { border-top: 1px solid var(--rule); padding-top: 12px; display: flex;
              flex-direction: column; gap: 12px; }
  #settings h2 { font-size: 12px; font-weight: 600; color: var(--faint); margin: 0 8px;
                 text-transform: uppercase; letter-spacing: 0.08em; }
  .set-row { display: flex; align-items: center; gap: 8px; padding: 0 8px; }
  .set-row .set-name { font-size: 13px; color: var(--muted); flex: 1; }
  .theme-btn { background: none; border: 1px solid var(--rule); color: var(--muted);
               border-radius: 999px; padding: 4px 12px; font-size: 13px; cursor: pointer; }
  .theme-btn[aria-pressed="true"] { background: var(--ink); color: var(--bg); border-color: var(--ink); }
  .side-btn { background: none; border: 1px solid var(--rule); color: var(--ink);
              border-radius: 8px; padding: 8px 12px; font-size: 13px; cursor: pointer;
              margin: 0 8px; text-align: left; }
  .side-btn:hover { border-color: var(--muted); }
  #memory-panel { display: none; margin: 0 8px; background: var(--field);
                  border: 1px solid var(--rule); border-radius: 8px; padding: 8px 10px;
                  font-size: 12px; line-height: 1.5; max-height: 320px; overflow-y: auto;
                  white-space: pre-wrap; word-wrap: break-word; color: var(--muted); }
  #memory-panel.open { display: block; }
  #person-form { display: none; flex-direction: column; gap: 8px; padding: 0 8px; }
  #person-form.open { display: flex; }
  #person-form input { background: var(--field); color: var(--ink); border: 1px solid var(--rule);
                       border-radius: 8px; padding: 8px; font-size: 13px; width: 100%; }
  #person-form button { background: var(--ink); color: var(--bg); border: 0; border-radius: 8px;
                        padding: 8px; font-size: 13px; font-weight: 600; cursor: pointer; }

  main { flex: 1; display: flex; flex-direction: column; min-width: 0; }

  /* faces — three views over the same core */
  #face-bar { display: flex; gap: 4px; padding: 10px 20px 0; max-width: 960px; margin: 0 auto; width: 100%; }
  .face-btn { display: inline-flex; align-items: center; gap: 7px; background: none; border: 0;
              border-bottom: 2px solid transparent; color: var(--muted); padding: 8px 10px;
              font-size: 14px; font-weight: 600; cursor: pointer; }
  .face-btn svg { width: 15px; height: 15px; flex: none; }
  .face-btn[aria-selected="true"] { color: var(--ink); border-bottom-color: var(--accent); }
  .face-badge { background: var(--accent-bg); color: #fff; border-radius: 999px; font-size: 11px;
                padding: 1px 7px; font-weight: 700; display: none; }
  .face-badge.on { display: inline-block; }
  .face { display: none; flex: 1; flex-direction: column; min-height: 0; }
  .face.on { display: flex; }
  #face-graph, #face-today { overflow-y: auto; padding: 20px; }
  .face-inner { max-width: 960px; margin: 0 auto; width: 100%; display: flex; flex-direction: column; gap: 20px; }
  .card { background: var(--panel); border: 1px solid var(--rule); border-radius: 12px; padding: 16px 20px; }
  .card h3 { margin: 0 0 6px; font-size: 15px; }
  .card h4 { margin: 0 0 8px; font-size: 12px; font-weight: 600; color: var(--faint);
             text-transform: uppercase; letter-spacing: 0.08em; }
  .card p { margin: 4px 0; line-height: 1.55; }
  .card ul { margin: 6px 0; padding-left: 18px; line-height: 1.6; }
  .quiet { color: var(--muted); font-size: 14px; }
  .row-actions { display: flex; gap: 8px; margin-top: 10px; }
  .act-btn { border-radius: 8px; padding: 7px 14px; font-size: 13px; font-weight: 600; cursor: pointer;
             border: 1px solid var(--rule); background: none; color: var(--ink); }
  .act-btn.primary { background: var(--accent-bg); border-color: var(--accent-bg); color: #fff; }
  .act-btn.primary:hover { background: var(--accent-bg-hover); }
  .act-btn:hover { border-color: var(--muted); }
  .settled { color: var(--faint); font-size: 13px; }
  #graph-svg { width: 100%; height: auto; display: block; }
  .g-node rect { fill: var(--panel); stroke: var(--rule); }
  .g-node text { fill: var(--ink); font-size: 12px; }
  .g-node .g-kind { fill: var(--faint); font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; }
  .g-edge { stroke: var(--rule); stroke-width: 1.2; }
  .stream-item { border-left: 2px solid var(--rule); padding: 2px 0 14px 14px; margin-left: 4px; }
  .stream-item .s-kind { font-size: 11px; color: var(--accent); text-transform: uppercase; letter-spacing: 0.07em; font-weight: 700; }
  .stream-item .s-title { font-size: 14px; margin: 2px 0; }
  .stream-item .s-detail { font-size: 13px; color: var(--muted); line-height: 1.5; white-space: pre-wrap; }

  /* suite panel */
  #suite-panel { display: none; margin: 0 8px; flex-direction: column; gap: 10px; }
  #suite-panel.open { display: flex; }
  .suite-item { background: var(--field); border: 1px solid var(--rule); border-radius: 8px; padding: 8px 10px; }
  .suite-item .si-name { font-size: 13px; font-weight: 600; }
  .suite-item .si-kind { font-size: 11px; color: var(--accent); text-transform: uppercase; letter-spacing: 0.06em; font-weight: 700; }
  .suite-item .si-desc { font-size: 12px; color: var(--muted); line-height: 1.45; margin-top: 2px; }
  .suite-item .row-actions { margin-top: 8px; }
  .suite-item .act-btn { padding: 5px 10px; font-size: 12px; }
  .suite-item input { background: var(--bg); color: var(--ink); border: 1px solid var(--rule);
                      border-radius: 6px; padding: 6px 8px; font-size: 12px; width: 100%; margin-top: 6px; }
  #no-key-banner { display: none; background: var(--warn-bg); color: var(--warn-ink);
                   border-bottom: 1px solid var(--warn-rule); padding: 10px 20px; font-size: 14px; }
  #thread { flex: 1; overflow-y: auto; padding: 24px 20px 12px; }
  #messages { max-width: 760px; margin: 0 auto; display: flex; flex-direction: column; gap: 16px; }
  .msg { border-radius: 12px; padding: 12px 16px; font-size: 15px; line-height: 1.55;
         white-space: pre-wrap; word-wrap: break-word; }
  .msg.user { background: var(--user-bubble); align-self: flex-end; max-width: 85%; }
  .msg.assistant { background: var(--panel); border: 1px solid var(--rule);
                   align-self: flex-start; max-width: 92%; }
  .msg.step { background: transparent; color: var(--muted); font-size: 13px;
              font-family: ui-monospace, "SF Mono", Menlo, monospace;
              align-self: flex-start; padding: 0 16px; }
  .msg.error { background: var(--err-bg); color: var(--err-ink); align-self: flex-start; }
  .empty { color: var(--faint); text-align: center; margin-top: 12vh; font-size: 15px; line-height: 1.6; }
  #composer { border-top: 1px solid var(--rule); background: var(--bg); padding: 12px 20px 16px; }
  #composer-inner { max-width: 760px; margin: 0 auto; display: flex; gap: 8px; align-items: flex-end; }
  #composer-input { flex: 1; resize: none; border: 1px solid var(--rule); border-radius: 12px;
                    padding: 12px; font-size: 15px; font-family: inherit; min-height: 44px;
                    max-height: 160px; background: var(--field); color: var(--ink); }
  #send-btn { background: var(--ink); color: var(--bg); border: 0; border-radius: 10px;
              padding: 12px 20px; font-size: 14px; font-weight: 600; cursor: pointer; }
  #send-btn:disabled { opacity: 0.45; cursor: default; }

  @media (max-width: 760px) {
    body { flex-direction: column; }
    #sidebar { width: 100%; flex: none; max-height: 44vh; overflow-y: auto; border-right: 0;
               border-bottom: 1px solid var(--rule); }
  }
  @media (prefers-reduced-motion: reduce) {
    * { scroll-behavior: auto !important; transition: none !important; }
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
        <p id="person-error" role="alert" style="display:none; color: var(--err-ink); font-size: 12px; margin: 0;"></p>
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
          <svg id="graph-svg" role="img" aria-label="Graph of people, their finished work, the files it produced and the suite items they carry"></svg>
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
    await loadPeople();
    await loadThreads();
    await loadStatus();
  });

  document.getElementById('memory-btn').addEventListener('click', async (ev) => {
    const panel = document.getElementById('memory-panel');
    if (panel.classList.contains('open')) {
      panel.classList.remove('open');
      ev.currentTarget.setAttribute('aria-expanded', 'false');
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
      ev.currentTarget.setAttribute('aria-expanded', 'true');
      return;
    }
    const content = (data.content || '').trim();
    /* The scaffold alone ("# Memory", or About with nothing learned
       yet) reads as a raw file dump; say plainly that there is
       nothing learned yet. Otherwise the actual file is shown. */
    const learned = content.split('\n').some((line) => {
      const t = line.trim();
      return t.startsWith('- ') && !t.startsWith('- Name:') && !t.startsWith('- Role:');
    });
    panel.textContent = learned
      ? data.content
      : 'Nothing learned yet. Preferences and facts your agent picks up as it works will appear here.';
    panel.classList.add('open');
    ev.currentTarget.setAttribute('aria-expanded', 'true');
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
    summary.textContent = workCount
      ? nodes.filter((n) => n.kind === 'person').length + ' people · ' + workCount + ' finished jobs · ' + nodes.filter((n) => n.kind === 'file').length + ' files produced · ' + nodes.filter((n) => n.kind === 'item').length + ' suite items in play'
      : 'People, the work their agents finish, the files it produces, and the suite each person carries.';
    empty.style.display = workCount ? 'none' : 'block';
    /* Column layout by kind: person → suite item → work → file. */
    const cols = { person: 0, item: 1, work: 2, file: 3 };
    const colX = [70, 300, 530, 760];
    const perCol = { person: 0, item: 0, work: 0, file: 0 };
    const pos = {};
    for (const n of nodes) {
      const c = cols[n.kind] != null ? cols[n.kind] : 2;
      const row = perCol[n.kind] != null ? perCol[n.kind]++ : 0;
      pos[n.id] = { x: colX[c], y: 34 + row * 52, node: n };
    }
    const height = Math.max(120, 34 + Math.max(1, ...Object.values(perCol)) * 52);
    const NS = 'http://www.w3.org/2000/svg';
    svg.setAttribute('viewBox', '0 0 920 ' + height);
    svg.innerHTML = '';
    for (const e of edges) {
      const a = pos[e.from], b = pos[e.to];
      if (!a || !b) continue;
      const line = document.createElementNS(NS, 'line');
      line.setAttribute('x1', a.x + 62); line.setAttribute('y1', a.y);
      line.setAttribute('x2', b.x - 62); line.setAttribute('y2', b.y);
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
      html += '<p class="quiet">Suite: ' + (b.suite_size || 0) + ' items · Company library: ' + (b.library_size || 0) + ' items · ' +
        (b.pending_decisions || []).length + ' decisions and ' + (b.pending_offers || []).length + ' offers waiting below.</p>';
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
    const panel = document.getElementById('suite-panel');
    if (panel.classList.contains('open')) {
      panel.classList.remove('open');
      ev.currentTarget.setAttribute('aria-expanded', 'false');
      return;
    }
    await loadSuite();
    panel.classList.add('open');
    ev.currentTarget.setAttribute('aria-expanded', 'true');
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
                  '<input id="appfield-' + esc(item.id) + '-' + esc(f) + '" data-app-field="' + esc(f) + '" type="text" placeholder="' + esc(f) + '">';
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
