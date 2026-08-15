use anyhow::Result;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeExpr {
    Any,
    Bool,
    Integer,
    UnsignedInteger,
    Float,
    String,
    Array {
        item: Box<TypeExpr>,
    },
    Named {
        name: String,
    },
    /// A value that was JSON `null` in at least one sample, alongside a
    /// concrete type in others.
    Nullable {
        of: Box<TypeExpr>,
    },
    /// An object whose keys are data rather than field names:
    /// all-numeric keys, or enough same-shaped
    /// properties that they read as a lookup table rather than a record.
    Map {
        value: Box<TypeExpr>,
    },
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
    let (root, types) = registry.finish(root);

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
        _ => infer_scalar_type(value).unwrap_or(TypeExpr::Any),
    }
}

/// `None` means `value` was JSON `null` — the caller tracks that as
/// nullability rather than folding it into the merged type directly, so a
/// field that is null in some samples and, say, a string in others ends up
/// `Nullable { of: String }` instead of collapsing straight to `Any`.
fn infer_value_type(
    parent_path: &str,
    parent_name: &str,
    field_name: &str,
    value: &Value,
    registry: &mut TypeRegistry,
) -> Option<TypeExpr> {
    match value {
        Value::Object(map) => {
            let path = child_path_key(parent_path, field_name);
            let name = registry.type_name_for_path(&path, child_type_name(parent_name, field_name));
            registry.merge_named_object(&path, name.clone(), map);
            Some(TypeExpr::Named { name })
        }
        Value::Array(items) => Some(infer_array_type(
            parent_path,
            parent_name,
            field_name,
            items,
            registry,
        )),
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

    let nullable = items.iter().any(Value::is_null);
    let non_null: Vec<&Value> = items.iter().filter(|item| !item.is_null()).collect();

    let item_type = if non_null.is_empty() {
        TypeExpr::Any
    } else if non_null.iter().all(|item| item.is_object()) {
        let item_path = array_item_path_key(parent_path, field_name);
        let item_name =
            registry.type_name_for_path(&item_path, array_item_type_name(parent_name, field_name));
        for item in &non_null {
            let Value::Object(map) = item else {
                unreachable!();
            };
            registry.merge_named_object(&item_path, item_name.clone(), map);
        }

        TypeExpr::Named { name: item_name }
    } else {
        let item_path = array_item_path_key(parent_path, field_name);
        let item_name = array_item_type_name(parent_name, field_name);
        let mut inferred = None;
        for item in &non_null {
            let item_type = match item {
                Value::Array(nested_items) => {
                    infer_array_type(&item_path, &item_name, "", nested_items, registry)
                }
                Value::Object(_) => {
                    return TypeExpr::Array {
                        item: Box::new(TypeExpr::Any),
                    };
                }
                _ => infer_scalar_type(item).expect("null items were filtered out above"),
            };

            inferred = Some(match inferred {
                None => item_type,
                Some(existing) => merge_type_expr(&existing, &item_type),
            });
        }

        inferred.unwrap_or(TypeExpr::Any)
    };

    let item_type = if nullable {
        nullable_type(item_type)
    } else {
        item_type
    };

    TypeExpr::Array {
        item: Box::new(item_type),
    }
}

fn infer_scalar_type(value: &Value) -> Option<TypeExpr> {
    match value {
        Value::Null => None,
        Value::Bool(_) => Some(TypeExpr::Bool),
        Value::Number(number) => Some(if number.is_i64() {
            TypeExpr::Integer
        } else if number.is_u64() {
            TypeExpr::UnsignedInteger
        } else {
            TypeExpr::Float
        }),
        Value::String(_) => Some(TypeExpr::String),
        // Unreachable in practice: callers route Array/Object through
        // infer_value_type/infer_array_type before falling back here.
        Value::Array(_) | Value::Object(_) => Some(TypeExpr::Any),
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

    /// Finishes every builder, applies the map-shape rewrite,
    /// and drops declarations nothing reaches any more.
    fn finish(self, root: TypeExpr) -> (TypeExpr, Vec<NamedType>) {
        let named: BTreeMap<String, NamedType> = self
            .builders
            .into_iter()
            .map(|(name, builder)| (name, builder.finish()))
            .collect();

        let decided = decide_maps(&named);

        let root = substitute(root, &decided);
        let named: BTreeMap<String, NamedType> = named
            .into_iter()
            .map(|(name, named_type)| {
                let fields = named_type
                    .fields
                    .into_iter()
                    .map(|field| Field {
                        ty: substitute(field.ty, &decided),
                        ..field
                    })
                    .collect();
                (
                    name,
                    NamedType {
                        fields,
                        ..named_type
                    },
                )
            })
            .collect();

        let reachable = reachable_names(&root, &named);

        let types = named
            .into_iter()
            .filter(|(name, _)| reachable.contains(name))
            .map(|(_, named_type)| named_type)
            .collect();

        (root, types)
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

    fn merge(&mut self, fields: Vec<(String, Option<TypeExpr>)>) {
        self.instances += 1;

        for (name, ty) in fields {
            let state = self.fields.entry(name).or_insert(FieldState {
                ty: None,
                nullable: false,
                present: 0,
            });
            state.present += 1;

            match ty {
                Some(ty) => {
                    state.ty = Some(match state.ty.take() {
                        None => ty,
                        Some(existing) => merge_type_expr(&existing, &ty),
                    });
                }
                None => state.nullable = true,
            }
        }
    }

    fn finish(self) -> NamedType {
        NamedType {
            name: self.name,
            fields: self
                .fields
                .into_iter()
                .map(|(name, state)| {
                    let ty = match (state.ty, state.nullable) {
                        (Some(ty), true) => nullable_type(ty),
                        (Some(ty), false) => ty,
                        // Every observed instance had this key set to `null`.
                        (None, _) => TypeExpr::Any,
                    };
                    Field {
                        name,
                        ty,
                        optional: state.present < self.instances,
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
struct FieldState {
    /// Merged type from non-null observations only; `None` until the first
    /// one is seen.
    ty: Option<TypeExpr>,
    /// Whether at least one observation was JSON `null`.
    nullable: bool,
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
        (TypeExpr::Map { value: left_value }, TypeExpr::Map { value: right_value }) => {
            TypeExpr::Map {
                value: Box::new(merge_type_expr(left_value, right_value)),
            }
        }
        (TypeExpr::Nullable { of }, other) | (other, TypeExpr::Nullable { of }) => {
            nullable_type(merge_type_expr(of, other))
        }
        _ => TypeExpr::Any,
    }
}

/// `T` and `Nullable(T)` unify to `Nullable(T)`; nullability never nests, and
/// wrapping `Any` is a no-op since it already stands for "could be anything,
/// including null".
fn nullable_type(ty: TypeExpr) -> TypeExpr {
    match ty {
        TypeExpr::Any | TypeExpr::Nullable { .. } => ty,
        other => TypeExpr::Nullable {
            of: Box::new(other),
        },
    }
}

/// an object's keys are data, not field
/// names, when every key is numeric (any size), or there are enough
/// same-shaped properties that they read as a lookup table (20, or 50 when
/// every value looks like string data).
const MAP_SIZE_THRESHOLD: usize = 20;
const STRING_MAP_SIZE_THRESHOLD: usize = 50;

fn decide_maps(named: &BTreeMap<String, NamedType>) -> HashMap<String, TypeExpr> {
    named
        .iter()
        .filter_map(|(name, named_type)| {
            map_value_type(&named_type.fields).map(|value| {
                (
                    name.clone(),
                    TypeExpr::Map {
                        value: Box::new(value),
                    },
                )
            })
        })
        .collect()
}

fn map_value_type(fields: &[Field]) -> Option<TypeExpr> {
    if fields.len() < 2 {
        return None;
    }

    let unify = || {
        fields
            .iter()
            .map(|field| field.ty.clone())
            .reduce(|left, right| merge_type_expr(&left, &right))
            .expect("checked fields.len() >= 2 above")
    };

    let all_numeric_keys = fields
        .iter()
        .all(|field| !field.name.is_empty() && field.name.bytes().all(|b| b.is_ascii_digit()));
    if all_numeric_keys {
        return Some(unify());
    }

    let all_stringy = fields.iter().all(|field| field.ty == TypeExpr::String);
    if all_stringy && fields.len() < STRING_MAP_SIZE_THRESHOLD {
        return None;
    }

    if fields.len() < MAP_SIZE_THRESHOLD {
        return None;
    }

    match unify() {
        // The values had nothing in common, so the keys are field names.
        TypeExpr::Any => None,
        value => Some(value),
    }
}

/// Rewrites every `Named { name }` that [`decide_maps`] turned into a map,
/// recursing into the replacement too so a map whose value type itself
/// referenced another decided name still resolves fully. Named-type
/// references form a strict parent-to-child DAG (each is keyed by the JSON
/// path that produced it), so this recursion always terminates.
fn substitute(ty: TypeExpr, decided: &HashMap<String, TypeExpr>) -> TypeExpr {
    match ty {
        TypeExpr::Named { name } => match decided.get(&name) {
            Some(replacement) => substitute(replacement.clone(), decided),
            None => TypeExpr::Named { name },
        },
        TypeExpr::Array { item } => TypeExpr::Array {
            item: Box::new(substitute(*item, decided)),
        },
        TypeExpr::Nullable { of } => TypeExpr::Nullable {
            of: Box::new(substitute(*of, decided)),
        },
        TypeExpr::Map { value } => TypeExpr::Map {
            value: Box::new(substitute(*value, decided)),
        },
        other => other,
    }
}

fn reachable_names(root: &TypeExpr, named: &BTreeMap<String, NamedType>) -> BTreeSet<String> {
    let mut reachable = BTreeSet::new();
    let mut pending = Vec::new();
    collect_named_refs(root, &mut pending);

    while let Some(name) = pending.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }

        let Some(named_type) = named.get(&name) else {
            continue;
        };

        for field in &named_type.fields {
            collect_named_refs(&field.ty, &mut pending);
        }
    }

    reachable
}

fn collect_named_refs(ty: &TypeExpr, pending: &mut Vec<String>) {
    match ty {
        TypeExpr::Named { name } => pending.push(name.clone()),
        TypeExpr::Array { item } => collect_named_refs(item, pending),
        TypeExpr::Nullable { of } => collect_named_refs(of, pending),
        TypeExpr::Map { value } => collect_named_refs(value, pending),
        TypeExpr::Any
        | TypeExpr::Bool
        | TypeExpr::Integer
        | TypeExpr::UnsignedInteger
        | TypeExpr::Float
        | TypeExpr::String => {}
    }
}
