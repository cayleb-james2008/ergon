import json, re, pathlib, html.parser, collections
ROOT = pathlib.Path('/tmp/checker-design-v2/set')
FILES = sorted(p for p in ROOT.rglob('*') if p.is_file())
TEXT = {str(p.relative_to(ROOT)): p.read_text() for p in FILES}
out = {"files": {}, "checks": {}}

# ---------- inventory ----------
out["files"] = {"count": len(FILES), "paths": [str(p.relative_to(ROOT)) for p in FILES],
                "total_lines": sum(len(t.splitlines()) for t in TEXT.values())}

# ---------- external references / local resolution ----------
EXT = re.compile(r'https?://|//[a-z0-9.-]+\.[a-z]{2,}/|@import|<img\b|@font-face|fonts\.googleapis|fonts\.gstatic', re.I)
ext_hits = {}
for f, t in TEXT.items():
    for m in EXT.finditer(t):
        ext_hits.setdefault(f, []).append(t[max(0,m.start()-30):m.end()+30].replace('\n',' '))
out["checks"]["external_refs"] = ext_hits
refs = []
for f, t in TEXT.items():
    if not f.endswith('.html'): continue
    for m in re.finditer(r'(?:href|src)="([^"]+)"', t):
        refs.append((f, m.group(1)))
missing = []
for f, r in refs:
    if re.match(r'^[a-z]+:', r): continue
    target = (ROOT/f).parent / r
    if not target.exists(): missing.append((f, r))
out["checks"]["local_refs"] = {"count": len(refs), "missing": missing,
                               "unique": sorted(set(r for _, r in refs))}

# ---------- colour literals outside token files ----------
HEX = re.compile(r'#(?:[0-9a-fA-F]{3,8})\b')
FUNC = re.compile(r'\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color-mix)\s*\(')
NAMED = re.compile(r'(?<![\w-])(?:white|black|red|green|blue|gray|grey|silver|maroon|olive|lime|aqua|teal|navy|fuchsia|purple|orange|yellow|transparent|currentColor)(?![\w-])')
colour_hits = {}
for f, t in TEXT.items():
    if f in ('tokens.json', 'tokens.css'): continue
    hits = [m.group(0) for m in HEX.finditer(t)] + [m.group(0) for m in FUNC.finditer(t)] + [m.group(0) for m in NAMED.finditer(t)]
    if hits: colour_hits[f] = collections.Counter(hits).most_common(10)
out["checks"]["colour_literals"] = colour_hits

# ---------- custom property resolution ----------
defined = set(re.findall(r'(--[a-z0-9-]+)\s*:', TEXT['tokens.css']))
local = set()
used = {}
for f, t in TEXT.items():
    if f.endswith(('.css', '.html', '.js')):
        local |= set(re.findall(r'(--[a-z0-9-]+)\s*:', t))
        for n in re.findall(r'var\(\s*(--[a-z0-9-]+)', t):
            used.setdefault(n, set()).add(f)
all_defined = defined | local
missing_vars = {k: sorted(v) for k, v in used.items() if k not in all_defined}
out["checks"]["custom_props"] = {"defined": len(defined), "used": len(used), "missing": missing_vars,
                                 "unused": sorted(all_defined - set(used))}

# ---------- structure / labels ----------
class P(html.parser.HTMLParser):
    def __init__(self): super().__init__(); self.stack=[]; self.errors=[]; self.ids=[]; self.aria=[]; self.buttons=[]; self.tag_counts=collections.Counter()
    def handle_starttag(self, tag, attrs):
        a=dict(attrs); self.tag_counts[tag]+=1
        if tag in ('br','img','input','meta','link','hr','source'): return
        self.stack.append((tag,a))
        if 'id' in a: self.ids.append(a['id'])
        if 'aria-controls' in a: self.aria.append(a['aria-controls'])
    def handle_endtag(self, tag):
        if tag in ('br','img','input','meta','link','hr','source'): return
        if self.stack and self.stack[-1][0]==tag: self.stack.pop()
        else: self.errors.append(f"close </{tag}> while stack top {self.stack[-1][0] if self.stack else None}")
pages = [f for f in TEXT if f.endswith('.html')]
structure = {}
for f in pages:
    p=P(); p.feed(TEXT[f])
    structure[f]={"unclosed": [t for t,_ in p.stack], "errors": p.errors[:5], "ids_dup": [i for i,c in collections.Counter(p.ids).items() if c>1],
                  "aria_controls_missing": [x for x in p.aria if x not in p.ids], "label": "DESIGN COMP — NOT A BUILT SURFACE" in TEXT[f],
                  "buttons": p.tag_counts['button'], "inputs": p.tag_counts['input']}
out["checks"]["structure"] = structure
out["checks"]["labels"] = {f: structure[f]["label"] for f in pages}

# ---------- targets ----------
target_hits = []
for f,t in TEXT.items():
    for m in re.finditer(r'min-height:\s*([^;"}]+)', t):
        v=m.group(1).strip()
        if 'var(--target' in v: continue
        target_hits.append((f, v, t[:m.start()].count('\n')+1))
out["checks"]["target_literals"] = target_hits
out["checks"]["employee_class_used"] = {f: t.count('employee') for f,t in TEXT.items() if 'employee' in t}

# ---------- motion ----------
durations = collections.Counter()
for f,t in TEXT.items():
    if not f.endswith(('.css','.html','.js')): continue
    for m in re.finditer(r'(\d+(?:\.\d+)?)(ms|s)\b', t):
        val=m.group(1)+m.group(2)
        durations[(f,val)]+=1
motion = {"duration_literals": {f"{f}:{v}":c for (f,v),c in sorted(durations.items())},
          "ease_values": sorted(set(re.findall(r'cubic-bezier\([^)]*\)', ' '.join(TEXT.values())))),
          "transition_all": [f for f,t in TEXT.items() if re.search(r'transition-all', t, re.I)],
          "infinite": [f for f,t in TEXT.items() if re.search(r'infinite', t, re.I)],
          "keyframes": sorted(set(re.findall(r'@keyframes\s+([\w-]+)', ' '.join(TEXT.values())))),
          "animation_props": sorted(set(re.findall(r'animation(?:-name)?:\s*([^;}]+)', ' '.join(TEXT.values()))))[:10]}
out["checks"]["motion"] = motion

# ---------- unslop ----------
unslop = {}
unslop["gradients"] = {f: sorted(set(re.findall(r'(?:linear|radial|conic)-gradient\(', t))) for f,t in TEXT.items() if 'gradient(' in t}
unslop["text_shadow"] = [f for f,t in TEXT.items() if 'text-shadow' in t]
unslop["drop_shadow"] = [f for f,t in TEXT.items() if 'drop-shadow' in t]
unslop["uppercase"] = [f for f,t in TEXT.items() if re.search(r'text-transform\s*:\s*uppercase', t, re.I)]
unslop["weights"] = sorted(set(re.findall(r'font-weight:\s*(\d+)', ' '.join(TEXT.values()))))
emoji = re.compile('[\U0001F000-\U0001FAFF\u2600-\u27BF]')
unslop["emoji"] = {f: sorted(set(emoji.findall(t))) for f,t in TEXT.items() if emoji.search(t)}
unslop["sparkle_or_new"] = {f: sorted(set(re.findall(r'sparkle|✨|"New"|badge', t, re.I))) for f,t in TEXT.items() if re.search(r'sparkle|✨|"New"|badge', t, re.I)}
unslop["hype"] = {f: sorted(set(re.findall(r'supercharge|seamless|revolutionary|unlock|magic|AI-powered|game-chang', t, re.I))) for f,t in TEXT.items() if re.search(r'supercharge|seamless|revolutionary|unlock|magic|AI-powered|game-chang', t, re.I)}
unslop["transition_all"] = motion["transition_all"]
out["checks"]["unslop"] = unslop

# ---------- states ----------
states = {}
for f,t in TEXT.items():
    if not f.endswith('.html'): continue
    states[f] = {s: bool(re.search(s, t, re.I)) for s in ('hover','focus','active|pressed','disabled','busy|loading','error','empty')}
out["checks"]["states"] = states

json.dump(out, open('/tmp/checker-design-v2/audit.json','w'), indent=2)
print(json.dumps({k:(v if k!='files' else {'count':v['count'],'total_lines':v['total_lines']}) for k,v in out['checks'].items()}, indent=2)[:6000])
