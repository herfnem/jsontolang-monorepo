local function is_ident_start(ch)
  return ch == "_" or ch == "$" or ch:match("%a") ~= nil
end

local function is_ident_char(ch)
  return ch == "_" or ch == "$" or ch:match("%w") ~= nil
end

local function render_type_name(name)
  local out = {}
  for i = 1, #name do
    local ch = name:sub(i, i)
    -- NOTE: not `(i == 1) and is_ident_start(ch) or is_ident_char(ch)` --
    -- that classic Lua ternary idiom silently breaks when is_ident_start(ch)
    -- is false, falling through to is_ident_char(ch) instead of staying false.
    local valid
    if i == 1 then
      valid = is_ident_start(ch)
    else
      valid = is_ident_char(ch)
    end
    if valid then
      out[#out + 1] = ch
    elseif i == 1 then
      out[#out + 1] = "_"
      if ch:match("%d") then
        out[#out + 1] = ch
      end
    elseif out[#out] ~= "_" then
      out[#out + 1] = "_"
    end
  end
  local result = table.concat(out)
  return result == "" and "Root" or result
end

local function json_escape(str)
  local out = { '"' }
  for i = 1, #str do
    local b = str:byte(i)
    local ch = str:sub(i, i)
    if ch == '"' then
      out[#out + 1] = '\\"'
    elseif ch == '\\' then
      out[#out + 1] = '\\\\'
    elseif b == 8 then
      out[#out + 1] = '\\b'
    elseif b == 9 then
      out[#out + 1] = '\\t'
    elseif b == 10 then
      out[#out + 1] = '\\n'
    elseif b == 12 then
      out[#out + 1] = '\\f'
    elseif b == 13 then
      out[#out + 1] = '\\r'
    elseif b < 32 then
      out[#out + 1] = string.format('\\u%04x', b)
    else
      out[#out + 1] = ch
    end
  end
  out[#out + 1] = '"'
  return table.concat(out)
end

local function is_typescript_identifier(name)
  if #name == 0 then
    return false
  end
  if not is_ident_start(name:sub(1, 1)) then
    return false
  end
  for i = 2, #name do
    if not is_ident_char(name:sub(i, i)) then
      return false
    end
  end
  return true
end

local function render_property_name(name)
  if is_typescript_identifier(name) then
    return name
  end
  return json_escape(name)
end

local function render_type(ty)
  local kind = ty.kind
  if kind == "any" then
    return "any"
  elseif kind == "bool" then
    return "boolean"
  elseif kind == "integer" or kind == "unsigned_integer" or kind == "float" then
    return "number"
  elseif kind == "string" then
    return "string"
  elseif kind == "named" then
    return render_type_name(ty.name)
  elseif kind == "array" then
    return render_type(ty.item) .. "[]"
  end
  error("unknown type kind: " .. tostring(kind))
end

local function render_named_type(named, out)
  out[#out + 1] = "export interface " .. render_type_name(named.name) .. " {\n"
  for _, field in ipairs(named.fields) do
    local optional = field.optional and "?" or ""
    out[#out + 1] = "  "
      .. render_property_name(field.name)
      .. optional
      .. ": "
      .. render_type(field.ty)
      .. ";\n"
  end
  out[#out + 1] = "}\n\n"
end

return {
  key = "typescript",
  render = function(document)
    local out = {}
    for _, named in ipairs(document.types) do
      render_named_type(named, out)
    end
    if document.root.kind ~= "named" then
      out[#out + 1] = "export type "
        .. render_type_name(document.root_name)
        .. " = "
        .. render_type(document.root)
        .. ";\n"
    end
    return table.concat(out)
  end,
}
