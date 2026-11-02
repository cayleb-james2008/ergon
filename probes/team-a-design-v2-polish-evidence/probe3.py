#!/usr/bin/env python3
"""Real-element HC probe: cache-disabled reload, data-contrast path, then CDP forced-colors emulation path."""
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

def evaluate(ws, i, expr):
    m = send(ws, i, 'Runtime.evaluate', {'expression': expr, 'returnByValue': True, 'awaitPromise': True})
    return m['result']['result'].get('value')

if __name__ == '__main__':
    port = int(sys.argv[1]); needle = sys.argv[2]; expr = sys.stdin.read()
    t = target(port, needle)
    ws = websocket.create_connection(t['webSocketDebuggerUrl'], timeout=25, suppress_origin=True)
    try:
        send(ws, 1, 'Network.enable')
        send(ws, 2, 'Network.setCacheDisabled', {'cacheDisabled': True})
        send(ws, 3, 'Page.reload', {'ignoreCache': True})
        time.sleep(3.5)
        ws.settimeout(0.2)
        try:
            while True: ws.recv()
        except Exception:
            pass
        ws.settimeout(25)
        normal = evaluate(ws, 10, expr)
        send(ws, 11, 'Emulation.setEmulatedMedia', {'media': 'screen', 'features': [{'name': 'forced-colors', 'value': 'active'}]})
        time.sleep(0.3)
        forced = evaluate(ws, 12, expr)
        send(ws, 13, 'Emulation.setEmulatedMedia', {'media': '', 'features': []})
        print(json.dumps({'target': {'url': t.get('url')}, 'dataContrastPath': json.loads(normal) if normal else None, 'forcedColorsPath': json.loads(forced) if forced else None}, indent=1))
    finally:
        ws.close()
