use jsontolang_core::schema::{Document, Field, NamedType, TypeExpr, infer_document};
use serde_json::json;

#[test]
fn infers_primitive_fields() {
    let document = infer_document(
        "User",
        &json!({
            "active": true,
            "age": 3,
            "score": 4.5,
            "name": "Neko"
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "User".into(),
            root: TypeExpr::Named {
                name: "User".into()
            },
            types: vec![NamedType {
                name: "User".into(),
                fields: vec![
                    Field {
                        name: "active".into(),
                        ty: TypeExpr::Bool,
                        optional: false,
                    },
                    Field {
                        name: "age".into(),
                        ty: TypeExpr::Integer,
                        optional: false,
                    },
                    Field {
                        name: "name".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    },
                    Field {
                        name: "score".into(),
                        ty: TypeExpr::Float,
                        optional: false,
                    },
                ],
            }],
        }
    );
}

#[test]
fn marks_missing_fields_optional_when_merging_object_arrays() {
    let document = infer_document(
        "User",
        &json!({
            "pets": [
                { "name": "Mochi", "age": 3 },
                { "name": "Tuna" }
            ]
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "User".into(),
            root: TypeExpr::Named {
                name: "User".into()
            },
            types: vec![
                NamedType {
                    name: "User".into(),
                    fields: vec![Field {
                        name: "pets".into(),
                        ty: TypeExpr::Array {
                            item: Box::new(TypeExpr::Named {
                                name: "UserPetsItem".into()
                            }),
                        },
                        optional: false,
                    }],
                },
                NamedType {
                    name: "UserPetsItem".into(),
                    fields: vec![
                        Field {
                            name: "age".into(),
                            ty: TypeExpr::Integer,
                            optional: true,
                        },
                        Field {
                            name: "name".into(),
                            ty: TypeExpr::String,
                            optional: false,
                        },
                    ],
                },
            ],
        }
    );
}

#[test]
fn falls_back_to_any_for_mixed_primitive_arrays() {
    let document = infer_document("Root", &json!({ "values": [1, "two", true] })).unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Root".into(),
            root: TypeExpr::Named {
                name: "Root".into()
            },
            types: vec![NamedType {
                name: "Root".into(),
                fields: vec![Field {
                    name: "values".into(),
                    ty: TypeExpr::Array {
                        item: Box::new(TypeExpr::Any)
                    },
                    optional: false,
                }],
            }],
        }
    );
}

#[test]
fn infers_root_arrays() {
    let document = infer_document(
        "Users",
        &json!([
            { "name": "Mochi" },
            { "name": "Tuna", "age": 2 }
        ]),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Users".into(),
            root: TypeExpr::Array {
                item: Box::new(TypeExpr::Named {
                    name: "UsersItem".into()
                }),
            },
            types: vec![NamedType {
                name: "UsersItem".into(),
                fields: vec![
                    Field {
                        name: "age".into(),
                        ty: TypeExpr::Integer,
                        optional: true,
                    },
                    Field {
                        name: "name".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    },
                ],
            }],
        }
    );
}

#[test]
fn keeps_same_nested_field_names_distinct_across_paths() {
    let document = infer_document(
        "Order",
        &json!({
            "billing": {
                "address": { "street": "A Street" }
            },
            "shipping": {
                "address": { "city": "A City" }
            }
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Order".into(),
            root: TypeExpr::Named {
                name: "Order".into()
            },
            types: vec![
                NamedType {
                    name: "Order".into(),
                    fields: vec![
                        Field {
                            name: "billing".into(),
                            ty: TypeExpr::Named {
                                name: "OrderBilling".into()
                            },
                            optional: false,
                        },
                        Field {
                            name: "shipping".into(),
                            ty: TypeExpr::Named {
                                name: "OrderShipping".into()
                            },
                            optional: false,
                        },
                    ],
                },
                NamedType {
                    name: "OrderBilling".into(),
                    fields: vec![Field {
                        name: "address".into(),
                        ty: TypeExpr::Named {
                            name: "OrderBillingAddress".into()
                        },
                        optional: false,
                    }],
                },
                NamedType {
                    name: "OrderBillingAddress".into(),
                    fields: vec![Field {
                        name: "street".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    }],
                },
                NamedType {
                    name: "OrderShipping".into(),
                    fields: vec![Field {
                        name: "address".into(),
                        ty: TypeExpr::Named {
                            name: "OrderShippingAddress".into()
                        },
                        optional: false,
                    }],
                },
                NamedType {
                    name: "OrderShippingAddress".into(),
                    fields: vec![Field {
                        name: "city".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    }],
                },
            ],
        }
    );
}

#[test]
fn uses_item_suffix_for_array_object_type_names() {
    let document = infer_document(
        "Catalog",
        &json!({
            "addresses": [{ "street": "A Street" }],
            "statuses": [{ "label": "active" }],
            "species": [{ "name": "cat" }]
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Catalog".into(),
            root: TypeExpr::Named {
                name: "Catalog".into()
            },
            types: vec![
                NamedType {
                    name: "Catalog".into(),
                    fields: vec![
                        Field {
                            name: "addresses".into(),
                            ty: TypeExpr::Array {
                                item: Box::new(TypeExpr::Named {
                                    name: "CatalogAddressesItem".into(),
                                }),
                            },
                            optional: false,
                        },
                        Field {
                            name: "species".into(),
                            ty: TypeExpr::Array {
                                item: Box::new(TypeExpr::Named {
                                    name: "CatalogSpeciesItem".into(),
                                }),
                            },
                            optional: false,
                        },
                        Field {
                            name: "statuses".into(),
                            ty: TypeExpr::Array {
                                item: Box::new(TypeExpr::Named {
                                    name: "CatalogStatusesItem".into(),
                                }),
                            },
                            optional: false,
                        },
                    ],
                },
                NamedType {
                    name: "CatalogAddressesItem".into(),
                    fields: vec![Field {
                        name: "street".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    }],
                },
                NamedType {
                    name: "CatalogSpeciesItem".into(),
                    fields: vec![Field {
                        name: "name".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    }],
                },
                NamedType {
                    name: "CatalogStatusesItem".into(),
                    fields: vec![Field {
                        name: "label".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    }],
                },
            ],
        }
    );
}

#[test]
fn disambiguates_distinct_raw_keys_that_normalize_to_same_type_name() {
    let document = infer_document(
        "Profile",
        &json!({
            "foo-bar": { "street": "A Street" },
            "foo_bar": { "city": "A City" }
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Profile".into(),
            root: TypeExpr::Named {
                name: "Profile".into()
            },
            types: vec![
                NamedType {
                    name: "Profile".into(),
                    fields: vec![
                        Field {
                            name: "foo-bar".into(),
                            ty: TypeExpr::Named {
                                name: "ProfileFooBar".into()
                            },
                            optional: false,
                        },
                        Field {
                            name: "foo_bar".into(),
                            ty: TypeExpr::Named {
                                name: "ProfileFooBar2".into()
                            },
                            optional: false,
                        },
                    ],
                },
                NamedType {
                    name: "ProfileFooBar".into(),
                    fields: vec![Field {
                        name: "street".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    }],
                },
                NamedType {
                    name: "ProfileFooBar2".into(),
                    fields: vec![Field {
                        name: "city".into(),
                        ty: TypeExpr::String,
                        optional: false,
                    }],
                },
            ],
        }
    );
}

#[test]
fn preserves_homogeneous_nested_arrays() {
    let document = infer_document(
        "Grid",
        &json!({
            "matrix": [[1, 2], [3, 4]]
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Grid".into(),
            root: TypeExpr::Named {
                name: "Grid".into()
            },
            types: vec![NamedType {
                name: "Grid".into(),
                fields: vec![Field {
                    name: "matrix".into(),
                    ty: TypeExpr::Array {
                        item: Box::new(TypeExpr::Array {
                            item: Box::new(TypeExpr::Integer)
                        }),
                    },
                    optional: false,
                }],
            }],
        }
    );
}

#[test]
fn widens_mixed_numeric_fields_to_float() {
    let document = infer_document(
        "Metrics",
        &json!({
            "samples": [
                { "value": 1 },
                { "value": 2.5 }
            ]
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Metrics".into(),
            root: TypeExpr::Named {
                name: "Metrics".into()
            },
            types: vec![
                NamedType {
                    name: "Metrics".into(),
                    fields: vec![Field {
                        name: "samples".into(),
                        ty: TypeExpr::Array {
                            item: Box::new(TypeExpr::Named {
                                name: "MetricsSamplesItem".into()
                            }),
                        },
                        optional: false,
                    }],
                },
                NamedType {
                    name: "MetricsSamplesItem".into(),
                    fields: vec![Field {
                        name: "value".into(),
                        ty: TypeExpr::Float,
                        optional: false,
                    }],
                },
            ],
        }
    );
}

#[test]
fn falls_back_to_any_for_mixed_signed_and_large_u64_integer_values() {
    let document = infer_document(
        "Metrics",
        &json!({
            "samples": [
                { "value": 1 },
                { "value": 18446744073709551615u64 }
            ]
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Metrics".into(),
            root: TypeExpr::Named {
                name: "Metrics".into()
            },
            types: vec![
                NamedType {
                    name: "Metrics".into(),
                    fields: vec![Field {
                        name: "samples".into(),
                        ty: TypeExpr::Array {
                            item: Box::new(TypeExpr::Named {
                                name: "MetricsSamplesItem".into()
                            }),
                        },
                        optional: false,
                    }],
                },
                NamedType {
                    name: "MetricsSamplesItem".into(),
                    fields: vec![Field {
                        name: "value".into(),
                        ty: TypeExpr::Any,
                        optional: false,
                    }],
                },
            ],
        }
    );
}

#[test]
fn preserves_unsigned_width_for_u64_only_integer_values() {
    let document = infer_document(
        "Metrics",
        &json!({
            "samples": [
                { "value": 9223372036854775808u64 },
                { "value": 18446744073709551615u64 }
            ]
        }),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Metrics".into(),
            root: TypeExpr::Named {
                name: "Metrics".into()
            },
            types: vec![
                NamedType {
                    name: "Metrics".into(),
                    fields: vec![Field {
                        name: "samples".into(),
                        ty: TypeExpr::Array {
                            item: Box::new(TypeExpr::Named {
                                name: "MetricsSamplesItem".into()
                            }),
                        },
                        optional: false,
                    }],
                },
                NamedType {
                    name: "MetricsSamplesItem".into(),
                    fields: vec![Field {
                        name: "value".into(),
                        ty: TypeExpr::UnsignedInteger,
                        optional: false,
                    }],
                },
            ],
        }
    );
}

#[test]
fn excludes_orphan_named_types_after_mixed_shape_field_merge() {
    let document = infer_document(
        "Records",
        &json!([
            { "meta": { "flag": true } },
            { "meta": "plain" }
        ]),
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            root_name: "Records".into(),
            root: TypeExpr::Array {
                item: Box::new(TypeExpr::Named {
                    name: "RecordsItem".into()
                }),
            },
            types: vec![NamedType {
                name: "RecordsItem".into(),
                fields: vec![Field {
                    name: "meta".into(),
                    ty: TypeExpr::Any,
                    optional: false,
                }],
            }],
        }
    );
}
