local RUST_KEYWORDS = {}
for _, kw in ipairs({
  "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if",
  "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
  "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
  "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
  "unsized", "virtual", "yield", "try",
}) do
  RUST_KEYWORDS[kw] = true
end

local function is_rust_keyword(name)
  return RUST_KEYWORDS[name] == true
end

local function escape_rust_keyword(name)
  if is_rust_keyword(name) then
    return "r#" .. name
  end
  return name
end

local function is_reserved_rust_type_name(name)
  return name == "Self"
end

local function disambiguate_rust_type_name(name)
  if is_reserved_rust_type_name(name) then
    return name .. "Type"
  end
  return name
end

local function is_rust_ident_start(ch)
  return ch == "_" or ch:match("%a") ~= nil
end

local function is_rust_ident_char(ch)
  return ch == "_" or ch:match("%w") ~= nil
end

local function sanitize_type_name(name)
  local out = {}
  for i = 1, #name do
    local ch = name:sub(i, i)
    -- NOTE: not `(i == 1) and is_rust_ident_start(ch) or is_rust_ident_char(ch)`
    -- -- that classic Lua ternary idiom silently breaks when
    -- is_rust_ident_start(ch) is false, falling through to
    -- is_rust_ident_char(ch) instead of staying false.
    local valid
    if i == 1 then
      valid = is_rust_ident_start(ch)
    else
      valid = is_rust_ident_char(ch)
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
  if result == "" then
    result = "Root"
  end
  return disambiguate_rust_type_name(result)
end

local function rust_debug_escape(str)
  local out = { '"' }
  for i = 1, #str do
    local b = str:byte(i)
    local ch = str:sub(i, i)
    if ch == '"' then
      out[#out + 1] = '\\"'
    elseif ch == '\\' then
      out[#out + 1] = '\\\\'
    elseif b == 9 then
      out[#out + 1] = '\\t'
    elseif b == 10 then
      out[#out + 1] = '\\n'
    elseif b == 13 then
      out[#out + 1] = '\\r'
    elseif b < 32 or b == 127 then
      out[#out + 1] = string.format('\\u{%x}', b)
    else
      out[#out + 1] = ch
    end
  end
  out[#out + 1] = '"'
  return table.concat(out)
end

local function render_field_name(name)
  local out = {}
  local needs_separator = false
  for i = 1, #name do
    local ch = name:sub(i, i)
    if ch:match("%w") then
      if #out == 0 and ch:match("%d") then
        out[#out + 1] = "_"
      end
      out[#out + 1] = ch:lower()
      needs_separator = false
    elseif #out > 0 then
      needs_separator = true
    end
    if needs_separator and out[#out] ~= "_" then
      out[#out + 1] = "_"
    end
  end
  while out[#out] == "_" do
    out[#out] = nil
  end
  local result = table.concat(out)
  return result == "" and "field" or result
end

local function render_field_names(fields)
  local used = {}
  local names = {}
  for _, field in ipairs(fields) do
    local base = render_field_name(field.name)
    used[base] = (used[base] or 0) + 1
    local n = used[base]
    local name = (n == 1) and base or (base .. "_" .. tostring(n))
    names[#names + 1] = escape_rust_keyword(name)
  end
  return names
end

local function allocate_type_names(document)
  local allocated = {}
  local used = {}

  local function allocate(raw_name)
    local base = sanitize_type_name(raw_name)
    used[base] = (used[base] or 0) + 1
    local n = used[base]
    allocated[raw_name] = (n == 1) and base or (base .. tostring(n))
  end

  for _, named in ipairs(document.types) do
    allocate(named.name)
  end

  if document.root.kind ~= "named" then
    allocate(document.root_name)
  end

  return allocated
end

local function render_named_type_name(name, type_names)
  return type_names[name] or sanitize_type_name(name)
end

local function render_type(ty, type_names)
  local kind = ty.kind
  if kind == "any" then
    return "serde_json::Value"
  elseif kind == "bool" then
    return "bool"
  elseif kind == "integer" then
    return "i64"
  elseif kind == "unsigned_integer" then
    return "u64"
  elseif kind == "float" then
    return "f64"
  elseif kind == "string" then
    return "String"
  elseif kind == "named" then
    return render_named_type_name(ty.name, type_names)
  elseif kind == "array" then
    return "Vec<" .. render_type(ty.item, type_names) .. ">"
  end
  error("unknown type kind: " .. tostring(kind))
end

local function render_named_type(named, type_names, out)
  out[#out + 1] = "#[derive(Debug, Clone, Serialize, Deserialize)]\n"
  out[#out + 1] = "pub struct " .. render_named_type_name(named.name, type_names) .. " {\n"

  local field_names = render_field_names(named.fields)

  for i, field in ipairs(named.fields) do
    local field_name = field_names[i]
    if field_name ~= field.name then
      out[#out + 1] = "    #[serde(rename = " .. rust_debug_escape(field.name) .. ")]\n"
    end

    local ty = render_type(field.ty, type_names)
    if field.optional then
      ty = "Option<" .. ty .. ">"
    end

    out[#out + 1] = "    pub " .. field_name .. ": " .. ty .. ",\n"
  end

  out[#out + 1] = "}\n\n"
end

return {
  key = "rust",
  render = function(document)
    local type_names = allocate_type_names(document)
    local out = { "use serde::{Deserialize, Serialize};\n\n" }

    for _, named in ipairs(document.types) do
      render_named_type(named, type_names, out)
    end

    if document.root.kind ~= "named" then
      out[#out + 1] = "pub type "
        .. render_named_type_name(document.root_name, type_names)
        .. " = "
        .. render_type(document.root, type_names)
        .. ";\n"
    end

    return table.concat(out)
  end,
}
