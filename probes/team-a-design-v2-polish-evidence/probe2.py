#!/usr/bin/env python3
"""Like probe.py but forces a cache-disabled reload first (so a re-frozen CSS/JS is actually used)."""
import json, sys, time, urllib.request
import websocket

def target(port, needle):
    with urllib.request.urlopen(f'http://127.0.0.1:{port}/json/list', timeout=5) as r:
        for t in json.load(r):
            if needle in t.get('url', ''):
                return t
    raise SystemExit('no matching target')

if __name__ == '__main__':
    port = int(sys.argv[1]); needle = sys.argv[2]; expr = sys.stdin.read()
    t = target(port, needle)
    ws = websocket.create_connection(t['webSocketDebuggerUrl'], timeout=20, suppress_origin=True)
    try:
        ws.send(json.dumps({'id': 1, 'method': 'Network.enable'}))
        ws.send(json.dumps({'id': 2, 'method': 'Network.setCacheDisabled', 'params': {'cacheDisabled': True}}))
        ws.send(json.dumps({'id': 3, 'method': 'Page.reload', 'params': {'ignoreCache': True}}))
        time.sleep(3)
        # drain
        ws.settimeout(0.2)
        try:
            while True: ws.recv()
        except Exception:
            pass
        ws.settimeout(20)
        ws.send(json.dumps({'id': 4, 'method': 'Runtime.evaluate', 'params': {'expression': expr, 'returnByValue': True, 'awaitPromise': True}}))
        while True:
            msg = json.loads(ws.recv())
            if msg.get('id') == 4:
                print(json.dumps({'target': {'title': t.get('title'), 'url': t.get('url')}, 'result': msg}, indent=1))
                break
    finally:
        ws.close()
