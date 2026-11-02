"""Independent recomputation of the v2 contrast floors (checker wrote its own; this is a second implementation)."""
import json
t=json.load(open('tokens.json'))
def hx(s): s=s.lstrip('#'); return [int(s[i:i+2],16) for i in (0,2,4)]
def rgba(s):
    p=s[s.index('(')+1:s.index(')')].split(','); return [float(x) for x in p[:3]], (float(p[3]) if len(p)>3 else 1.0)
def over(bg, fg, a): return [round(fg[i]*a + bg[i]*(1-a)) for i in range(3)]
def lin(c):
    c/=255; return c/12.92 if c<=0.04045 else ((c+0.055)/1.055)**2.4
def lum(rgb): return 0.2126*lin(rgb[0])+0.7152*lin(rgb[1])+0.0722*lin(rgb[2])
def con(a,b):
    la,lb=lum(a),lum(b); return (max(la,lb)+0.05)/(min(la,lb)+0.05)
c=t['color']
def planes(cool):
    base=over(over(hx(c['base']),*rgba(c['ambientAccent'])), rgba(c['ambientCool'])[0], cool)
    p1=over(base,*rgba(c['surface1'])); p2=over(p1,*rgba(c['surface2'])); p3=over(p2,*rgba(c['surface3']))
    return base,p1,p2,p3
def row(name, val, claim):
    print(f"{name:34s} {val:7.3f}   claim {claim}")
print("== checker model: base + ambientAccent only (their code) ==")
_,p1,p2,p3=planes(0)
row("ink on p3", con(hx(c['ink']),p3), "10.03")
row("muted on p3", con(hx(c['muted']),p3), "6.13")
row("subtle on p1", con(hx(c['subtle']),p1), "6.64")
row("accent on p3", con(hx(c['accent']),p3), "5.46")
row("ok on washOk/p3", con(hx(c['ok']),over(p3,*rgba(c['washOk']))), "5.08")
row("warn on washWarn/p3", con(hx(c['warn']),over(p3,*rgba(c['washWarn']))), "5.37")
row("danger on washDanger/p3", con(hx(c['danger']),over(p3,*rgba(c['washDanger']))), "5.11")
row("focus accent on p2", con(hx(c['accent']),p2), "7.05")
row("border lineBright on p2", con(over(p2,*rgba(c['lineBright'])),p2), "3.08")
print("\n== actual page model: base + BOTH ambient washes (ambientCool ignored by their checker) ==")
_,q1,q2,q3=planes(0.035)
row("ink on p3", con(hx(c['ink']),q3), "10.03")
row("muted on p3", con(hx(c['muted']),q3), "6.13")
row("subtle on p1", con(hx(c['subtle']),q1), "6.64")
row("accent on p3", con(hx(c['accent']),q3), "5.46")
row("ok on washOk/p3", con(hx(c['ok']),over(q3,*rgba(c['washOk']))), "5.08")
row("warn on washWarn/p3", con(hx(c['warn']),over(q3,*rgba(c['washWarn']))), "5.37")
row("danger on washDanger/p3", con(hx(c['danger']),over(q3,*rgba(c['washDanger']))), "5.11")
row("focus accent on p2", con(hx(c['accent']),q2), "7.05")
row("border lineBright on p2", con(over(q2,*rgba(c['lineBright'])),q2), "3.08  <-- FLOOR 3.0")
print("\n== boundaries the checker does not assert ==")
row("input border --line on p1", con(over(q1,*rgba(c['line'])),q1), "pack fieldLine floor 3.0")
row("input border --line on p2", con(over(q2,*rgba(c['line'])),q2), "pack fieldLine floor 3.0")
row("lineStrong on p2", con(over(q2,*rgba(c['lineStrong'])),q2), "n/a")
print("\n== light1 plane (defined, unused) if text were placed on it ==")
pl=over(q3,*rgba(c['light1']))
row("ink on light1", con(hx(c['ink']),pl), "7.0")
row("muted on light1", con(hx(c['muted']),pl), "4.5")
row("accent on light1", con(hx(c['accent']),pl), "4.5")
print("\n== pill.accent (defined, unused): accent on washAccent ==")
row("accent on washAccent/p3 (cool)", con(hx(c['accent']),over(q3,*rgba(c['washAccent']))), "4.5")
