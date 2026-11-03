package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = 1000
local S = B.STARTERS
local function cat(a, b) local d = { table.unpack(a) }; for _, x in ipairs(b) do d[#d+1] = x end; return d end
local decks = { {"S only (3 starters)", S}, {"S+Dojo (first deck)", cat(S, {"VG-HEAD-01"})}, {"S+Goku (Unc)", cat(S, {"DB-GOKU-01"})}, {"S+Musashi (Unc)", cat(S, {"VG-MUSASHI-01"})}, {"S+Bear (Rare)", cat(S, {"VS-BEAR-01"})}, {"S+Sea (Rare)", cat(S, {"BC-SEA-01"})}, {"S+Namek (Rare)", cat(S, {"DB-NAMEK-01"})}, {"S+Ichigo (Rare)", cat(S, {"BL-ICHIGO-01"})}, {"S+Guts (Epic)", cat(S, {"BK-GUTS-01"})}, {"S+Kakashi (Legend)", cat(S, {"NR-KAKASHI-01"})} }
local route = B.ROUTES[9]
io.write(string.format("%-22s", "w3-enc-1 enemy level:"))
local Ls = {1, 4, 5, 6, 7, 8, 9}
for _, L in ipairs(Ls) do io.write(string.format("%6d", L)) end
print("   (win% at player max level = effective 6 at chapter 1; N=" .. N .. ")")
for _, d in ipairs(decks) do
  io.write(string.format("%-22s", d[1]))
  for _, L in ipairs(Ls) do
    B.enemyLevel = function(i) return (i == 8) and L or 1 end
    io.write(string.format("%6d", math.floor(B.wins(d[2], 30, route, N) / N * 100 + 0.5)))
  end
  print()
end
