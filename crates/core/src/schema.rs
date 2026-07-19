use anyhow::Result;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeExpr {
    Any,
    Bool,
    Integer,
    UnsignedInteger,
    Float,
    String,
    Array { item: Box<TypeExpr> },
    Named { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpr,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NamedType {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Document {
    pub root_name: String,
    pub root: TypeExpr,
    pub types: Vec<NamedType>,
}

pub fn to_type_name(name: &str) -> String {
    let mut result = String::new();

    for part in name.split(|ch: char| !ch.is_ascii_alphanumeric()) {
        if part.is_empty() {
            continue;
        }

        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.push_str(chars.as_str());
        }
    }

    if result.is_empty() {
        "Root".into()
    } else {
        result
    }
}

pub fn child_type_name(parent_name: &str, field_name: &str) -> String {
    if parent_name.is_empty() {
        to_type_name(field_name)
    } else {
        format!("{}{}", to_type_name(parent_name), to_type_name(field_name))
    }
}

pub fn array_item_type_name(parent_name: &str, field_name: &str) -> String {
    if field_name.is_empty() {
        format!("{}Item", to_type_name(parent_name))
    } else {
        format!("{}Item", child_type_name(parent_name, field_name))
    }
}

pub fn infer_document(root_name: &str, value: &Value) -> Result<Document> {
    let mut registry = TypeRegistry::new();
    let root_type_name = to_type_name(root_name);
    let root = infer_root_type(root_name, &root_type_name, value, &mut registry);
    let types = registry.finish(&root);

    Ok(Document {
        root_name: root_type_name,
        root,
        types,
    })
}

fn infer_root_type(
    root_path: &str,
    root_name: &str,
    value: &Value,
    registry: &mut TypeRegistry,
) -> TypeExpr {
    match value {
        Value::Object(map) => {
            registry.merge_named_object(root_path, root_name.to_string(), map);
            TypeExpr::Named {
                name: root_name.to_string(),
            }
        }
        Value::Array(items) => infer_array_type(root_path, root_name, "", items, registry),
        _ => infer_scalar_type(value),
    }
}

fn infer_value_type(
    parent_path: &str,
    parent_name: &str,
    field_name: &str,
    value: &Value,
    registry: &mut TypeRegistry,
) -> TypeExpr {
    match value {
        Value::Object(map) => {
            let path = child_path_key(parent_path, field_name);
            let name = registry.type_name_for_path(&path, child_type_name(parent_name, field_name));
            registry.merge_named_object(&path, name.clone(), map);
            TypeExpr::Named { name }
        }
        Value::Array(items) => {
            infer_array_type(parent_path, parent_name, field_name, items, registry)
        }
        _ => infer_scalar_type(value),
    }
}

fn infer_array_type(
    parent_path: &str,
    parent_name: &str,
    field_name: &str,
    items: &[Value],
    registry: &mut TypeRegistry,
) -> TypeExpr {
    if items.is_empty() {
        return TypeExpr::Array {
            item: Box::new(TypeExpr::Any),
        };
    }

    if items.iter().all(Value::is_object) {
        let item_path = array_item_path_key(parent_path, field_name);
        let item_name =
            registry.type_name_for_path(&item_path, array_item_type_name(parent_name, field_name));
        for item in items {
            let Value::Object(map) = item else {
                unreachable!();
            };
            registry.merge_named_object(&item_path, item_name.clone(), map);
        }

        return TypeExpr::Array {
            item: Box::new(TypeExpr::Named { name: item_name }),
        };
    }

    let item_path = array_item_path_key(parent_path, field_name);
    let item_name = array_item_type_name(parent_name, field_name);
    let mut inferred = None;
    for item in items {
        let item_type = match item {
            Value::Array(nested_items) => {
                infer_array_type(&item_path, &item_name, "", nested_items, registry)
            }
            Value::Object(_) => {
                return TypeExpr::Array {
                    item: Box::new(TypeExpr::Any),
                };
            }
            _ => infer_scalar_type(item),
        };

        inferred = Some(match inferred {
            None => item_type,
            Some(existing) => merge_type_expr(&existing, &item_type),
        });
    }

    TypeExpr::Array {
        item: Box::new(inferred.unwrap_or(TypeExpr::Any)),
    }
}

fn infer_scalar_type(value: &Value) -> TypeExpr {
    match value {
        Value::Null => TypeExpr::Any,
        Value::Bool(_) => TypeExpr::Bool,
        Value::Number(number) => {
            if number.is_i64() {
                TypeExpr::Integer
            } else if number.is_u64() {
                TypeExpr::UnsignedInteger
            } else {
                TypeExpr::Float
            }
        }
        Value::String(_) => TypeExpr::String,
        Value::Array(_) | Value::Object(_) => TypeExpr::Any,
    }
}

fn child_path_key(parent_path: &str, field_name: &str) -> String {
    if parent_path.is_empty() {
        field_name.to_string()
    } else {
        format!("{parent_path}\x1f{field_name}")
    }
}

fn array_item_path_key(parent_path: &str, field_name: &str) -> String {
    let base = if field_name.is_empty() {
        parent_path.to_string()
    } else {
        child_path_key(parent_path, field_name)
    };

    format!("{base}\x1f[]")
}

#[derive(Debug, Default)]
struct TypeRegistry {
    builders: BTreeMap<String, NamedTypeBuilder>,
    path_names: BTreeMap<String, String>,
    used_names: BTreeMap<String, usize>,
}

impl TypeRegistry {
    fn new() -> Self {
        Self::default()
    }

    fn finish(self, root: &TypeExpr) -> Vec<NamedType> {
        let reachable = self.reachable_names(root);

        self.builders
            .into_iter()
            .filter(|(name, _)| reachable.contains(name))
            .map(|(_, builder)| builder)
            .map(|builder| builder.finish())
            .collect()
    }

    fn reachable_names(&self, root: &TypeExpr) -> BTreeSet<String> {
        let mut reachable = BTreeSet::new();
        let mut pending = Vec::new();
        collect_named_refs(root, &mut pending);

        while let Some(name) = pending.pop() {
            if !reachable.insert(name.clone()) {
                continue;
            }

            let Some(builder) = self.builders.get(&name) else {
                continue;
            };

            for state in builder.fields.values() {
                collect_named_refs(&state.ty, &mut pending);
            }
        }

        reachable
    }

    fn type_name_for_path(&mut self, path: &str, suggested_name: String) -> String {
        if let Some(name) = self.path_names.get(path) {
            return name.clone();
        }

        let next = self.used_names.entry(suggested_name.clone()).or_insert(0);
        *next += 1;

        let name = if *next == 1 {
            suggested_name
        } else {
            format!("{}{next}", suggested_name)
        };

        self.path_names.insert(path.to_string(), name.clone());
        name
    }

    fn merge_named_object(&mut self, path: &str, suggested_name: String, map: &Map<String, Value>) {
        let name = self.type_name_for_path(path, suggested_name);
        let mut fields = Vec::new();
        for (field_name, value) in map {
            let ty = infer_value_type(path, &name, field_name, value, self);
            fields.push((field_name.clone(), ty));
        }

        let builder = self
            .builders
            .entry(name.clone())
            .or_insert_with(|| NamedTypeBuilder::new(&name));
        builder.merge(fields);
    }
}

#[derive(Debug, Clone)]
struct NamedTypeBuilder {
    name: String,
    instances: usize,
    fields: BTreeMap<String, FieldState>,
}

impl NamedTypeBuilder {
    fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            instances: 0,
            fields: BTreeMap::new(),
        }
    }

    fn merge(&mut self, fields: Vec<(String, TypeExpr)>) {
        self.instances += 1;

        for (name, ty) in fields {
            let state = self.fields.entry(name).or_insert(FieldState {
                ty: ty.clone(),
                present: 0,
            });
            state.ty = merge_type_expr(&state.ty, &ty);
            state.present += 1;
        }
    }

    fn finish(self) -> NamedType {
        NamedType {
            name: self.name,
            fields: self
                .fields
                .into_iter()
                .map(|(name, state)| Field {
                    name,
                    ty: state.ty,
                    optional: state.present < self.instances,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
struct FieldState {
    ty: TypeExpr,
    present: usize,
}

fn merge_type_expr(left: &TypeExpr, right: &TypeExpr) -> TypeExpr {
    if left == right {
        return left.clone();
    }

    match (left, right) {
        (TypeExpr::Integer, TypeExpr::Float) | (TypeExpr::Float, TypeExpr::Integer) => {
            TypeExpr::Float
        }
        (TypeExpr::UnsignedInteger, TypeExpr::Float)
        | (TypeExpr::Float, TypeExpr::UnsignedInteger) => TypeExpr::Float,
        (TypeExpr::Integer, TypeExpr::UnsignedInteger)
        | (TypeExpr::UnsignedInteger, TypeExpr::Integer) => TypeExpr::Any,
        (TypeExpr::Array { item: left_item }, TypeExpr::Array { item: right_item }) => {
            TypeExpr::Array {
                item: Box::new(merge_type_expr(left_item, right_item)),
            }
        }
        _ => TypeExpr::Any,
    }
}

fn collect_named_refs(ty: &TypeExpr, pending: &mut Vec<String>) {
    match ty {
        TypeExpr::Named { name } => pending.push(name.clone()),
        TypeExpr::Array { item } => collect_named_refs(item, pending),
        TypeExpr::Any
        | TypeExpr::Bool
        | TypeExpr::Integer
        | TypeExpr::UnsignedInteger
        | TypeExpr::Float
        | TypeExpr::String => {}
    }
}
