local ROOT = "/home/cayleb/Work/tmp/card-balance-wt/src/shared/"
local realdofile, cfg = dofile, nil
dofile = function(p) if p:match("Config%.luau$") then cfg = cfg or realdofile(p); return cfg end; return realdofile(p) end  -- one shared Config
local Resolver = dofile(ROOT.."combat/Resolver.luau")
local Cards    = dofile(ROOT.."content/Cards.luau")
local Battles  = dofile(ROOT.."progression/Battles.luau")
local Config   = dofile(ROOT.."progression/Config.luau")
local by = {} for _, e in ipairs(Cards.EDITIONS) do by[e.id] = e end
local S = { "NR-NARUTO-01", "VS-THORFINN-01", "BC-ASTA-01" }
local ROUTES = { {"enc-1",1,"BL-ICHIGO-01"},{"enc-2",2,"VS-THORFINN-01"},{"enc-3",3,"BC-ASTA-01"},{"boss-1",3,"NR-KAKASHI-01"},
  {"w2-enc-1",1,"VG-MUSASHI-01"},{"w2-enc-2",2,"MHA-DEKU-01"},{"w2-enc-3",3,"BC-SEA-01"},{"boss-2",3,"DB-PRINCE-01"},
  {"w3-enc-1",1,"BK-GUTS-01"},{"w3-enc-2",2,"VG-RIVAL-01"},{"w3-enc-3",3,"MHA-BLAST-01"},{"boss-3",3,"MHA-FROSTBURN-01"} }
local function wins(ids, level, route, n)
  local lv = {} for _, id in ipairs(ids) do lv[id] = level end
  local w = 0
  for s = 1, n do
    local mine = assert(Battles.buildSnapshot(ids, by, lv, route[2], { atk = 1, hp = 1 }))
    local foe  = assert(Battles.buildEnemySnapshot({ route[3] }, by, Config.ROUTE_ENEMY_LEVEL and Config.ROUTE_ENEMY_LEVEL[route[1]] or nil))
    if assert(Resolver.resolveBattle(mine, foe, { rulesVersion = Cards.RULES_VERSION or "provisional-1", seed = s * 7919 })).snapshot.winner == "A" then w = w + 1 end
  end
  return w / n
end
local first = { S[1], S[2], S[3], "VG-HEAD-01" }
for _, deck in ipairs({ { "3 starters", S }, { "first deck", first } }) do
  for _, level in ipairs({ 6, 10, 30 }) do
    local row = {} for i, r in ipairs(ROUTES) do row[i] = string.format("%d", math.floor(wins(deck[2], level, r, 1000) * 100 + 0.5)) end
    print(deck[1] .. " L" .. level .. ": " .. table.concat(row, " "))
  end
end
for id, e in pairs(by) do   -- §8.6: starters + one card, w3-enc-1
  if id ~= S[1] and id ~= S[2] and id ~= S[3] then
    print(e.rarity, id, string.format("%.1f%%", wins({ S[1], S[2], S[3], id }, 30, ROUTES[9], 1000) * 100))
  end
end
