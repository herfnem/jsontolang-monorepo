-- ponytail: control-char detection is ASCII 0x00-0x1F + DEL only, not full
-- Unicode C1 range (0x80-0x9F) like Rust's char::is_control(); upgrade if a
-- plugin ever needs to key on non-ASCII control characters.
local function is_control_byte(b)
  return b < 0x20 or b == 0x7f
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

local function contains_any_byte(str, targets)
  for i = 1, #str do
    local b = str:byte(i)
    for _, target in ipairs(targets) do
      if b == target then
        return true
      end
    end
  end
  return false
end

local function has_control_char(str)
  for i = 1, #str do
    if is_control_byte(str:byte(i)) then
      return true
    end
  end
  return false
end

local function needs_custom_json_key(name)
  return name == "-" or name:find(",", 1, true) ~= nil or has_control_char(name)
end

local function needs_interpreted_tag_literal(name)
  return contains_any_byte(name, { 0x60, 0x5c, 0x22 }) or has_control_char(name)
end

local function render_json_tag(name)
  local tag = "json:" .. json_escape(name)
  if needs_interpreted_tag_literal(name) then
    return json_escape(tag)
  end
  return "`" .. tag .. "`"
end

local function render_struct_tag(name)
  if needs_custom_json_key(name) then
    return '`json:"-"`'
  end
  return render_json_tag(name)
end

local function is_go_ident_start(ch)
  return ch == "_" or ch:match("%a") ~= nil
end

local function is_go_ident_char(ch)
  return ch == "_" or ch:match("%w") ~= nil
end

local function render_type_name(name)
  local out = {}
  for i = 1, #name do
    local ch = name:sub(i, i)
    -- NOTE: not `(i == 1) and is_go_ident_start(ch) or is_go_ident_char(ch)`
    -- -- that classic Lua ternary idiom silently breaks when
    -- is_go_ident_start(ch) is false, falling through to
    -- is_go_ident_char(ch) instead of staying false.
    local valid
    if i == 1 then
      valid = is_go_ident_start(ch)
    else
      valid = is_go_ident_char(ch)
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

local function render_field_name(name)
  local out = {}
  local part = {}
  local function flush()
    if #part == 0 then
      return
    end
    local first = part[1]
    if #out == 0 and first:match("%d") then
      out[#out + 1] = "X"
    end
    out[#out + 1] = first:upper()
    for i = 2, #part do
      out[#out + 1] = part[i]:lower()
    end
    part = {}
  end
  for i = 1, #name do
    local ch = name:sub(i, i)
    if ch:match("%w") then
      part[#part + 1] = ch
    else
      flush()
    end
  end
  flush()
  local result = table.concat(out)
  return result == "" and "Field" or result
end

local function render_field_names(fields)
  local used = {}
  local names = {}
  for _, field in ipairs(fields) do
    local base = render_field_name(field.name)
    used[base] = (used[base] or 0) + 1
    local n = used[base]
    names[#names + 1] = (n == 1) and base or (base .. tostring(n))
  end
  return names
end

local function render_type(ty)
  local kind = ty.kind
  if kind == "any" then
    return "interface{}"
  elseif kind == "bool" then
    return "bool"
  elseif kind == "integer" then
    return "int64"
  elseif kind == "unsigned_integer" then
    return "uint64"
  elseif kind == "float" then
    return "float64"
  elseif kind == "string" then
    return "string"
  elseif kind == "named" then
    return render_type_name(ty.name)
  elseif kind == "array" then
    return "[]" .. render_type(ty.item)
  end
  error("unknown type kind: " .. tostring(kind))
end

local function render_field_type(field)
  local ty = render_type(field.ty)
  if field.optional then
    return "*" .. ty
  end
  return ty
end

local function named_type_needs_custom_json(named)
  for _, field in ipairs(named.fields) do
    if needs_custom_json_key(field.name) then
      return true
    end
  end
  return false
end

local function render_custom_json_methods(named, field_names, out)
  local type_name = render_type_name(named.name)

  out[#out + 1] = "func (value *" .. type_name .. ") UnmarshalJSON(data []byte) error {\n"
  out[#out + 1] = "\ttype plain " .. type_name .. "\n"
  out[#out + 1] = "\traw := map[string]json.RawMessage{}\n"
  out[#out + 1] = "\tif err := json.Unmarshal(data, &raw); err != nil {\n"
  out[#out + 1] = "\t\treturn err\n"
  out[#out + 1] = "\t}\n"

  for i, field in ipairs(named.fields) do
    if needs_custom_json_key(field.name) then
      local field_name = field_names[i]
      out[#out + 1] = "\tif field, ok := raw[" .. json_escape(field.name) .. "]; ok {\n"
      out[#out + 1] = "\t\tif err := json.Unmarshal(field, &value." .. field_name .. "); err != nil {\n"
      out[#out + 1] = "\t\t\treturn err\n"
      out[#out + 1] = "\t\t}\n"
      out[#out + 1] = "\t}\n"
      out[#out + 1] = "\tdelete(raw, " .. json_escape(field.name) .. ")\n"
    end
  end

  out[#out + 1] = "\trest, err := json.Marshal(raw)\n"
  out[#out + 1] = "\tif err != nil {\n"
  out[#out + 1] = "\t\treturn err\n"
  out[#out + 1] = "\t}\n"
  out[#out + 1] = "\treturn json.Unmarshal(rest, (*plain)(value))\n"
  out[#out + 1] = "}\n\n"

  out[#out + 1] = "func (value " .. type_name .. ") MarshalJSON() ([]byte, error) {\n"
  out[#out + 1] = "\ttype plain " .. type_name .. "\n"
  out[#out + 1] = "\tencoded, err := json.Marshal(plain(value))\n"
  out[#out + 1] = "\tif err != nil {\n"
  out[#out + 1] = "\t\treturn nil, err\n"
  out[#out + 1] = "\t}\n"
  out[#out + 1] = "\traw := map[string]json.RawMessage{}\n"
  out[#out + 1] = "\tif err := json.Unmarshal(encoded, &raw); err != nil {\n"
  out[#out + 1] = "\t\treturn nil, err\n"
  out[#out + 1] = "\t}\n"
  out[#out + 1] = "\tvar payload []byte\n"

  for i, field in ipairs(named.fields) do
    if needs_custom_json_key(field.name) then
      local field_name = field_names[i]
      out[#out + 1] = "\tpayload, err = json.Marshal(value." .. field_name .. ")\n"
      out[#out + 1] = "\tif err != nil {\n"
      out[#out + 1] = "\t\treturn nil, err\n"
      out[#out + 1] = "\t}\n"
      out[#out + 1] = "\traw[" .. json_escape(field.name) .. "] = payload\n"
    end
  end

  out[#out + 1] = "\treturn json.Marshal(raw)\n"
  out[#out + 1] = "}\n\n"
end

local function render_named_type(named, out)
  local field_names = render_field_names(named.fields)

  out[#out + 1] = "type " .. render_type_name(named.name) .. " struct {\n"
  for i, field in ipairs(named.fields) do
    out[#out + 1] = "\t"
      .. field_names[i]
      .. " "
      .. render_field_type(field)
      .. " "
      .. render_struct_tag(field.name)
      .. "\n"
  end
  out[#out + 1] = "}\n\n"

  if named_type_needs_custom_json(named) then
    render_custom_json_methods(named, field_names, out)
  end
end

return {
  key = "go",
  render = function(document)
    local needs_json_import = false
    for _, named in ipairs(document.types) do
      if named_type_needs_custom_json(named) then
        needs_json_import = true
        break
      end
    end

    local out = { "package models\n\n" }
    if needs_json_import then
      out[#out + 1] = 'import "encoding/json"\n\n'
    end

    for _, named in ipairs(document.types) do
      render_named_type(named, out)
    end

    if document.root.kind ~= "named" then
      out[#out + 1] = "type " .. render_type_name(document.root_name) .. " = " .. render_type(document.root) .. "\n"
    end

    return table.concat(out)
  end,
}
