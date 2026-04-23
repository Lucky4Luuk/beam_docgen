-- The doc generator turns anything you throw into it into documentation, as far as that is possible.
-- The final JSON output relies on the idea of tagged enums, so we can keep track of
-- what is what. A lua table is clearly different to a table containing function data, but there
-- is no easy way to tell these apart, so we use externally tagged enums (see https://serde.rs/enum-representations.html)
--
-- The final representation (before getting turned into JSON) should look something like this:
-- { "Table": { "__docs_name": "debug", "getinfo": ... } }
-- { "Function": { "__docs_name": "getinfo", "source": "[C]", ... } }
--
-- Full (pretty printed) representation just for clarity:
--  {
--      "Table": {
--          "__docs_name": "debug",
--          "getinfo": {
--              "Function": {
--                  "__docs_name": "getinfo",
--                  "source": "[C]",
--                  ...
--              }
--          }
--      }
--  }

local M = {}

local function arrContains(t, s)
    for i, v in ipairs(t) do
        if v == s then
            return true
        end
    end
    return false
end

-- Takes a function, figures out as much info about it as possible and turns
-- it into its docs representation (a map of info)
M.function_to_docs = function(r, name, f)
    local info = debug.getinfo(f)

    r["Function"] = { source = info.source, linedefined = info.linedefined, lastlinedefined = info.lastlinedefined }
    r["Function"].__docs_name = name
end

-- Generates the right structure to denote a reference cycle for the docs
M.ref_cycle_to_docs = function(name)
    local t = {}
    t["Cycle"] = { name = name }
    t["Cycle"].__docs_name = name
    return t
end

-- Turns a table into its docs representation
M.table_to_docs = function(r, name, t, seen, next_wave)
    r["Table"] = {}
    r["Table"].__docs_name = name
    table.insert(seen, tostring(t))

    for k, v in pairs(t) do
        -- We want to skip numeric indices for now, as they usually are only
        -- used for HUGE amounts of data in the api (think vehicle nodes)
        -- TODO: Handle numbered indices separately (we might still want to inspect the data inside of them)
        if type(k) == "string" then
            local kname = tostring(k)
            if arrContains(seen, tostring(v)) then -- we want to skip anything we have already seen to avoid looping references
                r["Table"][k] = M.ref_cycle_to_docs(kname)
            else
                -- r["Table"][k] = M.to_docs(kname, v, seen)
                -- We need to allocate a table in r to work in for the next wave
                r["Table"][k] = { alloc = true }
                table.insert(next_wave, { kname = kname, v = v, ref = r["Table"][k] })
            end
        end
    end

    return r
end

M.to_docs_wave = function(r, name, v, seen, next_wave)
    local kind = type(v)
    if kind == "table" then
        M.table_to_docs(r, name, v, seen, next_wave)
    elseif kind == "function" then
        M.function_to_docs(r, name, v)
    elseif kind == "string" or kind == "number" or kind == "boolean" or kind == "nil" then -- All of these can be directly passed through
        r["Value"] = { v = tostring(v) }
        r["Value"].__docs_name = name
    elseif kind == "userdata" then
        -- Mostlikely this is something like the `obj` table in VE lua, where you can just call getmetatable on it
        local t = getmetatable(v)
        if t ~= nil and t[1] ~= nil then
            -- print("Userdata " .. name .. " has a valid metatable. Adding it...")
            M.table_to_docs(r, name, t[1], seen, next_wave)
        else
            r["Other"] = { kind = "<" .. kind .. ">" }
            r["Other"].__docs_name = name
        end
    else
        r["Other"] = { kind = "<" .. kind .. ">" }
        r["Other"].__docs_name = name
    end
end

-- Turns any lua value into its docs representation (returns a table)
M.to_docs = function(name, v, seen)
    local r = {}
    local next_wave = {}
    M.to_docs_wave(r, name, v, seen, next_wave)

    while #next_wave > 0 do
        local current_wave = next_wave
        next_wave = {}

        for _, item in ipairs(current_wave) do
            item.ref.alloc = nil
            M.to_docs_wave(item.ref, item.kname, item.v, seen, next_wave)
        end
    end

    return r
end

-- Generates documentation as JSON data
M.gen_docs = function(name, item, filename)
    local file = io.open(filename, "w")
    if file then
        -- We take this root approach in case the input is not a table, so it still works to do gen_docs(5) and it'll output { "root": 5 }
        local t = {}
        t["root"] = M.to_docs(name, item, {})
        print("to_docs completed! Encoding now...")
        local json = jsonEncode(t)
        file:write(json)
        file:close()
        print("Wrote json data to file!")
    else
        print("Failed to open file!")
    end
end

return M
