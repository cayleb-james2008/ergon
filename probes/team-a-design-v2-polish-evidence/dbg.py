import json, time, urllib.request, websocket, sys
port=19222; needle=sys.argv[1]
with urllib.request.urlopen(f'http://127.0.0.1:{port}/json/list') as r:
    ts=json.load(r)
t=[x for x in ts if needle in x.get('url','')][0]
print('target:', t['url'])
ws=websocket.create_connection(t['webSocketDebuggerUrl'], timeout=20, suppress_origin=True)
def send(i,method,params=None):
    ws.send(json.dumps({'id':i,'method':method,'params':params or {}}))
    while True:
        m=json.loads(ws.recv())
        if m.get('id')==i: return m
print('eval1:', json.dumps(send(1,'Runtime.evaluate',{'expression':'JSON.stringify({url:location.href,ready:document.readyState,row:!!document.querySelector(".sidebar nav button[aria-current]")})','returnByValue':True}))[:400])
time.sleep(1)
print('eval2:', json.dumps(send(2,'Runtime.evaluate',{'expression':'JSON.stringify({matches:document.querySelector(".sidebar nav button[aria-current]")?.matches(\'[data-contrast="high"] .sidebar nav button[aria-current="page"]\')})','returnByValue':True}))[:400])
ws.close()
