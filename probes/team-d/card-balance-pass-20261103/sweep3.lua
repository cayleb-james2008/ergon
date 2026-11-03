package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = 200
local S = B.STARTERS
local function cat(a, b) local d = { table.unpack(a) }; for _, x in ipairs(b) do d[#d+1] = x end; return d end
local decks = {
  {"S+Dojo(first)", cat(S, {"VG-HEAD-01"})},
  {"S+Rare Bear", cat(S, {"VS-BEAR-01"})},
  {"S+Epic Guts", cat(S, {"BK-GUTS-01"})},
  {"4 Epics", {"BK-GUTS-01","MHA-BLAST-01","NR-SHADOW-01","VG-RIVAL-01"}},
  {"2Leg+2Epic", {"NR-KAKASHI-01","MHA-FROSTBURN-01","BK-GUTS-01","MHA-BLAST-01"}},
}
local names = {"e1","e2","e3","b1","w2e1","w2e2","w2e3","b2","w3e1","w3e2","w3e3","b3"}
local function summary(r)
  local fails = {}
  for i, x in ipairs(r) do if x < 0.5 then fails[#fails+1] = names[i]..string.format("(%d)", math.floor(x*100+.5)) end end
  return #fails, table.concat(fails, " ")
end
local function run(caps, bp)
  B.set(caps, bp)
  local out = {}
  for _, d in ipairs(decks) do
    local n, f = summary(B.row(d[2], 30, N))
    out[#out+1] = string.format("%s:%d", d[1], n)
  end
  local _, f1 = summary(B.row(decks[1][2], 30, N))
  print(string.format("caps {%d,%2d,%2d} bp %3d | fails<50%%  %s | first deck fails: %s", caps[1], caps[2], caps[3], bp, table.concat(out, "  "), f1))
end
for _, bp in ipairs({600, 450, 350}) do
  for _, c2 in ipairs({10, 8, 6}) do
    for _, c3 in ipairs({15, 10, 8, 6}) do
      if c3 >= c2 or (c2 == 10 and c3 == 15) or true then run({6, c2, c3, 21, 30}, bp) end
    end
  end
end
