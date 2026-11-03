package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = 300
print("starters at max level (L30) — win% per route; caps are {c1,c2,c3}; N="..N)
print("config                         " .. "e1  e2  e3  b1 | w2e1 w2e2 w2e3 b2 | w3e1 w3e2 w3e3 b3 | <50%  <90%")
local function run(caps, bp)
  B.set(caps, bp)
  local r = B.row(B.STARTERS, 30, N)
  local lo, mid = 0, 0
  for _, x in ipairs(r) do if x < 0.5 then lo = lo + 1 end; if x < 0.9 then mid = mid + 1 end end
  print(string.format("caps {%d,%d,%d} bp %3d   %s | %2d   %2d", caps[1], caps[2], caps[3], bp, B.fmt(r), lo, mid))
end
for _, bp in ipairs({600, 500, 400, 300}) do
  for _, c3 in ipairs({15, 10, 9, 8, 7}) do run({6, 10, c3, 21, 30}, bp) end
  for _, c in ipairs({{6,8,8},{6,8,7},{6,7,7},{6,6,6}}) do run({c[1],c[2],c[3],21,30}, bp) end
end
