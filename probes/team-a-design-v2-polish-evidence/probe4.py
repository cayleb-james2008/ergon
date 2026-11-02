#!/usr/bin/env python3
"""Clean real-element HC probe with settle time (the theme transitions are 120ms; immediate reads catch the start value)."""
import json, sys, time, urllib.request
import websocket

def target(port, needle):
    with urllib.request.urlopen(f'http://127.0.0.1:{port}/json/list', timeout=5) as r:
        for t in json.load(r):
            if needle in t.get('url', ''):
                return t
    raise SystemExit('no matching target')

def send(ws, i, method, params=None):
    ws.send(json.dumps({'id': i, 'method': method, 'params': params or {}}))
    while True:
        m = json.loads(ws.recv())
        if m.get('id') == i:
            return m

def ev(ws, i, expr):
    m = send(ws, i, 'Runtime.evaluate', {'expression': expr, 'returnByValue': True, 'awaitPromise': True})
    v = m['result']['result'].get('value')
    return json.loads(v) if v else None

if __name__ == '__main__':
    port = int(sys.argv[1]); needle = sys.argv[2]; expr = sys.stdin.read()
    t = target(port, needle)
    ws = websocket.create_connection(t['webSocketDebuggerUrl'], timeout=30, suppress_origin=True)
    try:
        send(ws, 0, 'Page.bringToFront')
        send(ws, 1, 'Network.enable'); send(ws, 2, 'Network.setCacheDisabled', {'cacheDisabled': True})
        send(ws, 3, 'Page.reload', {'ignoreCache': True}); time.sleep(2.5)
        # wait for a usable execution context / complete document
        ready = False
        for attempt in range(20):
            try:
                m = send(ws, 100 + attempt, 'Runtime.evaluate', {'expression': 'document.readyState', 'returnByValue': True})
                rs = m.get('result', {}).get('result', {}).get('value')
                if rs == 'complete':
                    ready = True
                    break
            except Exception:
                pass
            time.sleep(0.3)
        if not ready:
            time.sleep(2)
        ws.settimeout(0.2)
        try:
            while True: ws.recv()
        except Exception: pass
        ws.settimeout(30)
        normal = ev(ws, 10, expr)
        send(ws, 11, 'Runtime.evaluate', {'expression': "document.documentElement.setAttribute('data-contrast','high')"})
        time.sleep(0.6)  # settle: transitions are 120ms
        dc = ev(ws, 12, expr)
        send(ws, 13, 'Runtime.evaluate', {'expression': "document.documentElement.removeAttribute('data-contrast')"})
        time.sleep(0.6)
        send(ws, 14, 'Emulation.setEmulatedMedia', {'media': 'screen', 'features': [{'name': 'forced-colors', 'value': 'active'}]})
        time.sleep(0.6)
        fc = ev(ws, 15, expr)
        send(ws, 16, 'Emulation.setEmulatedMedia', {'media': '', 'features': []})
        print(json.dumps({'target': {'url': t.get('url')}, 'normal': normal, 'dataContrastPath_settled': dc, 'forcedColorsPath_settled': fc}, indent=1))
    finally:
        ws.close()
