//! Embedded web UI — plain HTML/CSS/JS, no framework, no build step.
//! Familiar chat shell: left sidebar with threads, one scrolling thread,
//! one composer row pinned at the bottom.

pub const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ergon — your agent that finishes the work</title>
<style>
  * { box-sizing: border-box; }
  html, body { height: 100%; margin: 0; }
  body { font-family: -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
         display: flex; background: #f7f7f5; color: #1f1e1c; }
  #sidebar { width: 260px; flex: 0 0 260px; background: #171614; color: #eceae6;
             display: flex; flex-direction: column; padding: 14px 10px; }
  #sidebar h1 { font-size: 17px; margin: 4px 8px 2px; font-weight: 650; }
  #sidebar .tagline { font-size: 12px; color: #a8a49c; margin: 0 8px 14px; line-height: 1.4; }
  #new-chat-btn { background: #d97757; color: #fff; border: 0; border-radius: 8px;
                  padding: 9px 12px; font-size: 14px; cursor: pointer; margin: 0 2px 12px; }
  #new-chat-btn:hover { background: #c86647; }
  #thread-list { list-style: none; margin: 0; padding: 0; overflow-y: auto; flex: 1; }
  #thread-list li { padding: 8px 10px; border-radius: 8px; font-size: 13.5px; cursor: pointer;
                    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: #cfccc5; }
  #thread-list li.active { background: #2b2926; color: #fff; }
  #thread-list li:hover { background: #232120; }
  #sidebar .foot { font-size: 11.5px; color: #8a867e; padding: 8px; line-height: 1.5; }
  main { flex: 1; display: flex; flex-direction: column; min-width: 0; }
  #no-key-banner { display: none; background: #fdf3e3; color: #7a4b12; border-bottom: 1px solid #ecd9ae;
                   padding: 10px 18px; font-size: 13.5px; }
  #thread { flex: 1; overflow-y: auto; padding: 26px 18px 12px; }
  #messages { max-width: 760px; margin: 0 auto; display: flex; flex-direction: column; gap: 14px; }
  .msg { border-radius: 12px; padding: 11px 14px; font-size: 14.5px; line-height: 1.55;
         white-space: pre-wrap; word-wrap: break-word; }
  .msg.user { background: #eceae6; align-self: flex-end; max-width: 85%; }
  .msg.assistant { background: #fff; border: 1px solid #e5e2dc; align-self: flex-start; max-width: 92%; }
  .msg.step { background: transparent; color: #6d6a63; font-size: 13px; align-self: flex-start;
              padding: 2px 14px; }
  .msg.error { background: #fdecec; color: #8c1d1d; align-self: flex-start; }
  .empty { color: #8a867e; text-align: center; margin-top: 12vh; font-size: 15px; line-height: 1.6; }
  #composer { border-top: 1px solid #e5e2dc; background: #f7f7f5; padding: 12px 18px 16px; }
  #composer-inner { max-width: 760px; margin: 0 auto; display: flex; gap: 8px; align-items: flex-end; }
  #composer-input { flex: 1; resize: none; border: 1px solid #d8d4cc; border-radius: 12px;
                    padding: 11px 13px; font-size: 14.5px; font-family: inherit; min-height: 44px;
                    max-height: 160px; background: #fff; }
  #send-btn { background: #1f1e1c; color: #fff; border: 0; border-radius: 10px;
              padding: 11px 18px; font-size: 14px; cursor: pointer; }
  #send-btn:disabled { opacity: .45; cursor: default; }
</style>
</head>
<body>
  <aside id="sidebar">
    <h1>Ergon</h1>
    <p class="tagline">An agent for every employee. Describe the work — it finishes it.</p>
    <button id="new-chat-btn" type="button">+ New chat</button>
    <ul id="thread-list"></ul>
    <div class="foot">Your agent learns how you work and keeps a personal memory of it.</div>
  </aside>
  <main>
    <div id="no-key-banner">No model key configured — set ERGON_MODEL_API_KEY to let the agent work. Nothing here is simulated: without a key, Ergon will not invent answers.</div>
    <div id="thread">
      <div id="messages">
        <div class="empty">What should your agent finish for you?<br>Give it a task in plain words — it works step by step, right here, until the job is done.</div>
      </div>
    </div>
    <form id="composer">
      <div id="composer-inner">
        <textarea id="composer-input" rows="1" placeholder="Describe a task — e.g. organise my notes into a clean summary document"></textarea>
        <button id="send-btn" type="submit">Send</button>
      </div>
    </form>
  </main>
<script>
  let currentThread = null;
  const messagesEl = document.getElementById('messages');
  const listEl = document.getElementById('thread-list');
  const inputEl = document.getElementById('composer-input');
  const sendBtn = document.getElementById('send-btn');

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

  async function loadStatus() {
    try {
      const r = await fetch('/api/status');
      const s = await r.json();
      if (!s.model_configured) document.getElementById('no-key-banner').style.display = 'block';
    } catch (e) { /* status is best-effort */ }
  }

  async function loadThreads() {
    const r = await fetch('/api/threads');
    const data = await r.json();
    listEl.innerHTML = '';
    for (const t of data.threads) {
      const li = document.createElement('li');
      li.textContent = t.title || 'Untitled task';
      if (currentThread && t.id === currentThread) li.className = 'active';
      li.onclick = () => openThread(t.id);
      listEl.appendChild(li);
    }
  }

  async function newThread() {
    const r = await fetch('/api/threads', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: '{}' });
    const t = await r.json();
    currentThread = t.id;
    messagesEl.innerHTML = '<div class="empty">New task — describe the work and your agent will finish it.</div>';
    await loadThreads();
    inputEl.focus();
  }

  async function openThread(id) {
    currentThread = id;
    const r = await fetch('/api/threads/' + encodeURIComponent(id));
    const t = await r.json();
    messagesEl.innerHTML = '';
    for (const m of (t.messages || [])) addMsg(m.role, m.content);
    if (!messagesEl.children.length) messagesEl.innerHTML = '<div class="empty">Empty thread.</div>';
    await loadThreads();
  }

  document.getElementById('new-chat-btn').onclick = newThread;

  document.getElementById('composer').addEventListener('submit', async (ev) => {
    ev.preventDefault();
    const text = inputEl.value.trim();
    if (!text) return;
    if (!currentThread) await newThread();
    inputEl.value = '';
    addMsg('user', text);
    sendBtn.disabled = true;
    const working = addMsg('step', 'Agent is working…');
    try {
      const r = await fetch('/api/chat', {
        method: 'POST', headers: {'Content-Type': 'application/json'},
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
    sendBtn.disabled = false;
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
  loadThreads();
</script>
</body>
</html>
"#;
