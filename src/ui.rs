//! Embedded web UI — plain HTML/CSS/JS, no framework, no build step.
//! Familiar chat shell: left sidebar with threads, one scrolling thread,
//! one composer row pinned at the bottom. Light + dark themes (dark is
//! the default), the switch in Settings; one agent per person, switched
//! in the sidebar; what the agent has learned is viewable under Memory.

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
    await loadThreads();
    await loadStatus();
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

  loadStatus();
  loadPeople();
  loadThreads();
</script>
</body>
</html>
"##;
