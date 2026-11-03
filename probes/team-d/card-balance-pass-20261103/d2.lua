package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = 1000
local S = B.STARTERS
local function cat(a, b) local d = { table.unpack(a) }; for _, x in ipairs(b) do d[#d+1] = x end; return d end
local decks = { {"3 starters", S}, {"first deck", cat(S, {"VG-HEAD-01"})}, {"S+Gale Mage (Unc)", cat(S, {"BC-WIND-01"})}, {"S+Goku (Unc)", cat(S, {"DB-GOKU-01"})}, {"S+Deku (Rare)", cat(S, {"MHA-DEKU-01"})} }
local route = B.ROUTES[9]
for _, mult in ipairs({ {1, 1}, {1.06, 1.06} }) do
  B.skillMult = { atk = mult[1], hp = mult[2] }
  print(string.format("\nskill multiplier atk %.2f hp %.2f — win%% at w3-enc-1, cards at ceiling", mult[1], mult[2]))
  io.write(string.format("%-20s", "enemy level:")); for _, L in ipairs({8, 9, 10, 11}) do io.write(string.format("%6d", L)) end print()
  for _, d in ipairs(decks) do
    io.write(string.format("%-20s", d[1]))
    for _, L in ipairs({8, 9, 10, 11}) do
      B.enemyLevel = function(i) return (i == 8) and L or 1 end
      io.write(string.format("%6d", math.floor(B.wins(d[2], 30, route, N) / N * 100 + 0.5)))
    end
    print()
  end
end
