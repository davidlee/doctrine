import re, sys, collections
ROUTES = ["review","demonstrate","probe","control","owner-fix","none"]
def load(path):
  rows = {}
  for line in open(path):
    c = [x.strip().strip('`').strip('*') for x in line.split('|')]
    if len(c) < 7 or not re.fullmatch(r'RV-\d+', c[1]) or not re.fullmatch(r'F-\d+', c[2]): continue
    if c[3].lower() in ROUTES and c[4].lower() not in ROUTES:  # severity column omitted: shift left
      c = c[:3] + [""] + c[3:]
    route = c[4].lower()
    second = c[5].lower() if c[5].lower() in ("duplicate","stale","neither") else ""
    rows[f"{c[1]}/{c[2]}"] = (route, second)
  return rows
def kappa(a, b, keys):
  n = len(keys); po = sum(a[k][0]==b[k][0] for k in keys)/n
  ca = collections.Counter(a[k][0] for k in keys); cb = collections.Counter(b[k][0] for k in keys)
  pe = sum(ca[r]*cb[r] for r in set(ca)|set(cb))/n/n
  return po, (po-pe)/(1-pe)
def disc(r, keys):
  c = collections.Counter(r[k][0] for k in keys); used = sum(1 for v in c.values() if v and True) - (1 if c.get("none") else 0)
  modal, m = c.most_common(1)[0]
  return c, used, modal, m/len(keys), used >= 3 and m/len(keys) <= 0.5
x, y = load(sys.argv[1]), load(sys.argv[2])
keys = sorted(set(x) & set(y))
print(f"rows: {len(x)} / {len(y)}; joined {len(keys)}; only-x {sorted(set(x)-set(y))} only-y {sorted(set(y)-set(x))}")
bad = [k for k in keys if x[k][0] not in ROUTES or y[k][0] not in ROUTES]
print("unparsed routes:", [(k, x[k][0], y[k][0]) for k in bad])
po, k = kappa(x, y, keys); print(f"R1 agreement {sum(x[k][0]==y[k][0] for k in keys)}/{len(keys)} = {po:.1%}, kappa {k:.2f}")
for name, r in (("X", x), ("Y", y)):
  c, used, modal, share, ok = disc(r, keys)
  print(f"R2 {name}: {dict(c)} routes-used(excl none)={used} modal={modal} {share:.1%} discriminating={ok}")
  sl = collections.Counter(r[k][1] for k in keys if r[k][0] in ("owner-fix","none"))
  print(f"   second labels on owner-fix/none: {dict(sl)}")
jo = [k for k in keys if x[k][0]=="owner-fix" and y[k][0]=="owner-fix"]
sa = [k for k in jo if x[k][1]==y[k][1] and x[k][1]]
js = [k for k in jo if x[k][1]=="stale" and y[k][1]=="stale"]
print(f"R4 jointly owner-fix {len(jo)}; second-label agreement {len(sa)}/{len(jo)} = {len(sa)/max(1,len(jo)):.1%}; jointly stale {len(js)} ({len(js)/max(1,len(jo)):.1%})")
print("   jo rows:", [(k, x[k][1], y[k][1]) for k in jo])
print("R5 none X:", [k for k in keys if x[k][0]=="none"]); print("R5 none Y:", [k for k in keys if y[k][0]=="none"])
conf = collections.Counter((x[k][0], y[k][0]) for k in keys if x[k][0]!=y[k][0])
print("disagreement pairs (X,Y):", conf.most_common())
