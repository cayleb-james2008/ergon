-- balance.lua — fresh simulation with the game's REAL resolver. Read-only: loads src/shared from SRC (argv[1] or default).
-- Config knobs (CHAPTER_CAPS, CARD_TRAINED_BP_PER_LEVEL) are overridden at run time through ONE shared Config table, so
-- the same code that ships is what is measured; nothing is copied or re-implemented.
local SRC = os.getenv("BAL_SRC") or "/tmp/claude-1000/-home-cayleb-Work/ae3da1fa-e00b-44d9-8428-93a167393f60/scratchpad/cg-candidate/src/shared/"
local realdofile, cfg = dofile, nil
dofile = function(p)
  if p:match("Config%.luau$") then cfg = cfg or realdofile(p); return cfg end
  return realdofile(p)
end
local Resolver = dofile(SRC.."combat/Resolver.luau")
local Cards    = dofile(SRC.."content/Cards.luau")
local Battles  = dofile(SRC.."progression/Battles.luau")
local Config   = dofile(SRC.."progression/Config.luau")
local by = {} for _, e in ipairs(Cards.EDITIONS) do by[e.id] = e end

local M = { Config = Config, by = by }
M.STARTERS = { "NR-NARUTO-01", "VS-THORFINN-01", "BC-ASTA-01" }   -- STARTER_IDS order, player = side A
M.ROUTES = {
  {"enc-1",1,"BL-ICHIGO-01"},{"enc-2",2,"VS-THORFINN-01"},{"enc-3",3,"BC-ASTA-01"},{"boss-1",3,"NR-KAKASHI-01"},
  {"w2-enc-1",1,"VG-MUSASHI-01"},{"w2-enc-2",2,"MHA-DEKU-01"},{"w2-enc-3",3,"BC-SEA-01"},{"boss-2",3,"DB-PRINCE-01"},
  {"w3-enc-1",1,"BK-GUTS-01"},{"w3-enc-2",2,"VG-RIVAL-01"},{"w3-enc-3",3,"MHA-BLAST-01"},{"boss-3",3,"MHA-FROSTBURN-01"} }

M.enemyLevel = nil   -- optional: function(routeIndex) -> enemy level (exploration model; the shipped change is verified on real code)
M.routeIndex = nil
function M.fight(ids, level, chapter, enemy, seed, routeId)
  local lv = {} for _, id in ipairs(ids) do lv[id] = level end
  local mine = assert(Battles.buildSnapshot(ids, by, lv, chapter, { atk = 1, hp = 1 }))
  -- the SHIPPED route table (Config.ROUTE_ENEMY_LEVEL) when the source has one; nil/absent = level 1 = raw stats (pre-pass code)
  local shipped = Config.ROUTE_ENEMY_LEVEL and Config.ROUTE_ENEMY_LEVEL[routeId] or nil
  local foe  = assert(Battles.buildEnemySnapshot({ enemy }, by, shipped))
  if M.enemyLevel and M.routeIndex then
    local L = M.enemyLevel(M.routeIndex)
    local bp = Config.CARD_TRAINED_BP_PER_LEVEL / 10000
    for _, f in ipairs(foe) do
      f.hp = math.max(1, math.floor(f.hp * (1 + bp * (L - 1)))); f.maxHp = f.hp
      f.atk = math.max(1, math.floor(f.atk * (1 + bp * (L - 1))))
    end
  end
  return assert(Resolver.resolveBattle(mine, foe, { rulesVersion = Cards.RULES_VERSION or "provisional-1", seed = seed }))
end

-- wins out of n for one deck/level/route
function M.wins(ids, level, route, n)
  local w = 0
  for i, r in ipairs(M.ROUTES) do if r == route then M.routeIndex = i - 1 end end
  for s = 1, n do
    if M.fight(ids, level, route[2], route[3], s * 7919, route[1]).snapshot.winner == "A" then w = w + 1 end
  end
  return w
end

-- all 12 routes -> array of win fractions
function M.row(ids, level, n)
  local r = {}
  for i, route in ipairs(M.ROUTES) do r[i] = M.wins(ids, level, route, n) / n end
  return r
end

function M.fmt(r)
  local t = {}
  for i, x in ipairs(r) do t[i] = string.format("%3d", math.floor(x * 100 + 0.5)) end
  return table.concat(t, " ")
end

function M.set(caps, bp)
  if caps then Config.CHAPTER_CAPS = caps end
  if bp then Config.CARD_TRAINED_BP_PER_LEVEL = bp end
end

return M
