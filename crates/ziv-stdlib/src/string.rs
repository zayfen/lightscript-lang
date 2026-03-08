//! String functions for Ziv standard library.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register string functions.
    pub fn register_string_functions(&mut self) {
        self.register(BuiltinFunction {
            name: "strlen".to_string(),
            params: vec![BuiltinParam {
                name: "s".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "string".to_string(),
            description: "Return the length of a string".to_string(),
        });

        self.register(BuiltinFunction {
            name: "concat".to_string(),
            params: vec![
                BuiltinParam {
                    name: "a".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "b".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "string".to_string(),
            description: "Concatenate two strings".to_string(),
        });

        self.register(BuiltinFunction {
            name: "substr".to_string(),
            params: vec![
                BuiltinParam {
                    name: "s".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "start".to_string(),
                    ty: "i64".to_string(),
                },
                BuiltinParam {
                    name: "length".to_string(),
                    ty: "i64".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "string".to_string(),
            description: "Return a substring from start with given length".to_string(),
        });

        self.register(BuiltinFunction {
            name: "char_at".to_string(),
            params: vec![
                BuiltinParam {
                    name: "s".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "index".to_string(),
                    ty: "i64".to_string(),
                },
            ],
            return_type: Some("char".to_string()),
            category: "string".to_string(),
            description: "Get character at specified index".to_string(),
        });

        self.register(BuiltinFunction {
            name: "to_upper".to_string(),
            params: vec![BuiltinParam {
                name: "s".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "string".to_string(),
            description: "Convert string to uppercase".to_string(),
        });

        self.register(BuiltinFunction {
            name: "to_lower".to_string(),
            params: vec![BuiltinParam {
                name: "s".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "string".to_string(),
            description: "Convert string to lowercase".to_string(),
        });

        self.register(BuiltinFunction {
            name: "trim".to_string(),
            params: vec![BuiltinParam {
                name: "s".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "string".to_string(),
            description: "Remove leading and trailing whitespace".to_string(),
        });

        self.register(BuiltinFunction {
            name: "contains".to_string(),
            params: vec![
                BuiltinParam {
                    name: "s".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "substr".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "string".to_string(),
            description: "Check if string contains substring".to_string(),
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
    fn test_string_functions_registered() {
        let stdlib = Stdlib::new();
        assert!(stdlib.is_builtin("strlen"));
        assert!(stdlib.is_builtin("concat"));
        assert!(stdlib.is_builtin("substr"));
        assert!(stdlib.is_builtin("char_at"));
        assert!(stdlib.is_builtin("to_upper"));
        assert!(stdlib.is_builtin("to_lower"));
        assert!(stdlib.is_builtin("trim"));
        assert!(stdlib.is_builtin("contains"));
    }

    #[test]
    fn test_string_function_signatures_and_return_types() {
        let stdlib = Stdlib::new();

        let strlen = stdlib.get("strlen").expect("strlen builtin must exist");
        assert_eq!(strlen.return_type.as_deref(), Some("i64"));
        assert_eq!(strlen.params.len(), 1);
        assert_param(strlen, 0, "s", "string");

        let concat = stdlib.get("concat").expect("concat builtin must exist");
        assert_eq!(concat.return_type.as_deref(), Some("string"));
        assert_eq!(concat.params.len(), 2);
        assert_param(concat, 0, "a", "string");
        assert_param(concat, 1, "b", "string");

        let substr = stdlib.get("substr").expect("substr builtin must exist");
        assert_eq!(substr.return_type.as_deref(), Some("string"));
        assert_eq!(substr.params.len(), 3);
        assert_param(substr, 0, "s", "string");
        assert_param(substr, 1, "start", "i64");
        assert_param(substr, 2, "length", "i64");

        let char_at = stdlib.get("char_at").expect("char_at builtin must exist");
        assert_eq!(char_at.return_type.as_deref(), Some("char"));
        assert_eq!(char_at.params.len(), 2);
        assert_param(char_at, 0, "s", "string");
        assert_param(char_at, 1, "index", "i64");

        let to_upper = stdlib.get("to_upper").expect("to_upper builtin must exist");
        assert_eq!(to_upper.return_type.as_deref(), Some("string"));
        assert_eq!(to_upper.params.len(), 1);
        assert_param(to_upper, 0, "s", "string");

        let to_lower = stdlib.get("to_lower").expect("to_lower builtin must exist");
        assert_eq!(to_lower.return_type.as_deref(), Some("string"));
        assert_eq!(to_lower.params.len(), 1);
        assert_param(to_lower, 0, "s", "string");

        let trim = stdlib.get("trim").expect("trim builtin must exist");
        assert_eq!(trim.return_type.as_deref(), Some("string"));
        assert_eq!(trim.params.len(), 1);
        assert_param(trim, 0, "s", "string");

        let contains = stdlib.get("contains").expect("contains builtin must exist");
        assert_eq!(contains.return_type.as_deref(), Some("bool"));
        assert_eq!(contains.params.len(), 2);
        assert_param(contains, 0, "s", "string");
        assert_param(contains, 1, "substr", "string");
    }
}
