//! Container functions for Ziv standard library.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register vector/hash-map style container functions.
    pub fn register_container_functions(&mut self) {
        // Vector
        self.register(BuiltinFunction {
            name: "vectorNew".to_string(),
            params: vec![],
            return_type: Some("vector".to_string()),
            category: "container".to_string(),
            description: "Create an empty vector".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorLen".to_string(),
            params: vec![BuiltinParam {
                name: "vec".to_string(),
                ty: "vector".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "container".to_string(),
            description: "Return vector length".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorPush".to_string(),
            params: vec![
                BuiltinParam {
                    name: "vec".to_string(),
                    ty: "vector".to_string(),
                },
                BuiltinParam {
                    name: "value".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("vector".to_string()),
            category: "container".to_string(),
            description: "Append value to vector".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorPop".to_string(),
            params: vec![BuiltinParam {
                name: "vec".to_string(),
                ty: "vector".to_string(),
            }],
            return_type: Some("any".to_string()),
            category: "container".to_string(),
            description: "Remove and return last vector element".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorGet".to_string(),
            params: vec![
                BuiltinParam {
                    name: "vec".to_string(),
                    ty: "vector".to_string(),
                },
                BuiltinParam {
                    name: "index".to_string(),
                    ty: "i64".to_string(),
                },
            ],
            return_type: Some("any".to_string()),
            category: "container".to_string(),
            description: "Get vector element at index".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorSet".to_string(),
            params: vec![
                BuiltinParam {
                    name: "vec".to_string(),
                    ty: "vector".to_string(),
                },
                BuiltinParam {
                    name: "index".to_string(),
                    ty: "i64".to_string(),
                },
                BuiltinParam {
                    name: "value".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("vector".to_string()),
            category: "container".to_string(),
            description: "Set vector element at index".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorInsert".to_string(),
            params: vec![
                BuiltinParam {
                    name: "vec".to_string(),
                    ty: "vector".to_string(),
                },
                BuiltinParam {
                    name: "index".to_string(),
                    ty: "i64".to_string(),
                },
                BuiltinParam {
                    name: "value".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("vector".to_string()),
            category: "container".to_string(),
            description: "Insert vector element at index".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorRemove".to_string(),
            params: vec![
                BuiltinParam {
                    name: "vec".to_string(),
                    ty: "vector".to_string(),
                },
                BuiltinParam {
                    name: "index".to_string(),
                    ty: "i64".to_string(),
                },
            ],
            return_type: Some("any".to_string()),
            category: "container".to_string(),
            description: "Remove vector element at index".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorContains".to_string(),
            params: vec![
                BuiltinParam {
                    name: "vec".to_string(),
                    ty: "vector".to_string(),
                },
                BuiltinParam {
                    name: "value".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "container".to_string(),
            description: "Return whether vector contains value".to_string(),
        });
        self.register(BuiltinFunction {
            name: "vectorClear".to_string(),
            params: vec![BuiltinParam {
                name: "vec".to_string(),
                ty: "vector".to_string(),
            }],
            return_type: Some("vector".to_string()),
            category: "container".to_string(),
            description: "Clear all vector elements".to_string(),
        });

        // HashMap
        self.register(BuiltinFunction {
            name: "hashMapNew".to_string(),
            params: vec![],
            return_type: Some("hashmap".to_string()),
            category: "container".to_string(),
            description: "Create an empty hash map".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapLen".to_string(),
            params: vec![BuiltinParam {
                name: "map".to_string(),
                ty: "hashmap".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "container".to_string(),
            description: "Return hash map size".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapSet".to_string(),
            params: vec![
                BuiltinParam {
                    name: "map".to_string(),
                    ty: "hashmap".to_string(),
                },
                BuiltinParam {
                    name: "key".to_string(),
                    ty: "any".to_string(),
                },
                BuiltinParam {
                    name: "value".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("hashmap".to_string()),
            category: "container".to_string(),
            description: "Set key/value pair in hash map".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapGet".to_string(),
            params: vec![
                BuiltinParam {
                    name: "map".to_string(),
                    ty: "hashmap".to_string(),
                },
                BuiltinParam {
                    name: "key".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("any".to_string()),
            category: "container".to_string(),
            description: "Get value by key from hash map".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapHas".to_string(),
            params: vec![
                BuiltinParam {
                    name: "map".to_string(),
                    ty: "hashmap".to_string(),
                },
                BuiltinParam {
                    name: "key".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "container".to_string(),
            description: "Return whether hash map has key".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapRemove".to_string(),
            params: vec![
                BuiltinParam {
                    name: "map".to_string(),
                    ty: "hashmap".to_string(),
                },
                BuiltinParam {
                    name: "key".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("any".to_string()),
            category: "container".to_string(),
            description: "Remove key from hash map and return old value".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapKeys".to_string(),
            params: vec![BuiltinParam {
                name: "map".to_string(),
                ty: "hashmap".to_string(),
            }],
            return_type: Some("array".to_string()),
            category: "container".to_string(),
            description: "Return all hash map keys".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapValues".to_string(),
            params: vec![BuiltinParam {
                name: "map".to_string(),
                ty: "hashmap".to_string(),
            }],
            return_type: Some("array".to_string()),
            category: "container".to_string(),
            description: "Return all hash map values".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapClear".to_string(),
            params: vec![BuiltinParam {
                name: "map".to_string(),
                ty: "hashmap".to_string(),
            }],
            return_type: Some("hashmap".to_string()),
            category: "container".to_string(),
            description: "Remove all entries from hash map".to_string(),
        });
        self.register(BuiltinFunction {
            name: "hashMapMerge".to_string(),
            params: vec![
                BuiltinParam {
                    name: "target".to_string(),
                    ty: "hashmap".to_string(),
                },
                BuiltinParam {
                    name: "source".to_string(),
                    ty: "hashmap".to_string(),
                },
            ],
            return_type: Some("hashmap".to_string()),
            category: "container".to_string(),
            description: "Merge source hash map into target hash map".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_param(func: &BuiltinFunction, idx: usize, name: &str, ty: &str) {
        assert_eq!(func.params[idx].name, name);
        assert_eq!(func.params[idx].ty, ty);
    }

    #[test]
    fn test_container_functions_registered() {
        let stdlib = Stdlib::new();
        for name in [
            "vectorNew",
            "vectorLen",
            "vectorPush",
            "vectorPop",
            "vectorGet",
            "vectorSet",
            "vectorInsert",
            "vectorRemove",
            "vectorContains",
            "vectorClear",
            "hashMapNew",
            "hashMapLen",
            "hashMapSet",
            "hashMapGet",
            "hashMapHas",
            "hashMapRemove",
            "hashMapKeys",
            "hashMapValues",
            "hashMapClear",
            "hashMapMerge",
        ] {
            assert!(stdlib.is_builtin(name), "missing builtin: {name}");
        }
    }

    #[test]
    fn test_vector_signatures() {
        let stdlib = Stdlib::new();
        let vnew = stdlib
            .get("vectorNew")
            .expect("vectorNew builtin must exist");
        assert!(vnew.params.is_empty());
        assert_eq!(vnew.return_type.as_deref(), Some("vector"));

        let vlen = stdlib
            .get("vectorLen")
            .expect("vectorLen builtin must exist");
        assert_eq!(vlen.params.len(), 1);
        assert_param(vlen, 0, "vec", "vector");
        assert_eq!(vlen.return_type.as_deref(), Some("i64"));

        let vset = stdlib
            .get("vectorSet")
            .expect("vectorSet builtin must exist");
        assert_eq!(vset.params.len(), 3);
        assert_param(vset, 0, "vec", "vector");
        assert_param(vset, 1, "index", "i64");
        assert_param(vset, 2, "value", "any");
        assert_eq!(vset.return_type.as_deref(), Some("vector"));

        let vcontains = stdlib
            .get("vectorContains")
            .expect("vectorContains builtin must exist");
        assert_eq!(vcontains.return_type.as_deref(), Some("bool"));
    }

    #[test]
    fn test_hashmap_signatures() {
        let stdlib = Stdlib::new();
        let mnew = stdlib
            .get("hashMapNew")
            .expect("hashMapNew builtin must exist");
        assert!(mnew.params.is_empty());
        assert_eq!(mnew.return_type.as_deref(), Some("hashmap"));

        let mset = stdlib
            .get("hashMapSet")
            .expect("hashMapSet builtin must exist");
        assert_eq!(mset.params.len(), 3);
        assert_param(mset, 0, "map", "hashmap");
        assert_param(mset, 1, "key", "any");
        assert_param(mset, 2, "value", "any");
        assert_eq!(mset.return_type.as_deref(), Some("hashmap"));

        let mkeys = stdlib
            .get("hashMapKeys")
            .expect("hashMapKeys builtin must exist");
        assert_eq!(mkeys.return_type.as_deref(), Some("array"));

        let mmerge = stdlib
            .get("hashMapMerge")
            .expect("hashMapMerge builtin must exist");
        assert_eq!(mmerge.params.len(), 2);
        assert_param(mmerge, 0, "target", "hashmap");
        assert_param(mmerge, 1, "source", "hashmap");
        assert_eq!(mmerge.return_type.as_deref(), Some("hashmap"));
    }
}
