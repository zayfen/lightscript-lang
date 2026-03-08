//! Math functions for Ziv standard library.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register math functions.
    pub fn register_math_functions(&mut self) {
        self.register(BuiltinFunction {
            name: "abs".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("number".to_string()),
            category: "math".to_string(),
            description: "Return the absolute value of a number".to_string(),
        });

        self.register(BuiltinFunction {
            name: "min".to_string(),
            params: vec![
                BuiltinParam {
                    name: "a".to_string(),
                    ty: "number".to_string(),
                },
                BuiltinParam {
                    name: "b".to_string(),
                    ty: "number".to_string(),
                },
            ],
            return_type: Some("number".to_string()),
            category: "math".to_string(),
            description: "Return the minimum of two numbers".to_string(),
        });

        self.register(BuiltinFunction {
            name: "max".to_string(),
            params: vec![
                BuiltinParam {
                    name: "a".to_string(),
                    ty: "number".to_string(),
                },
                BuiltinParam {
                    name: "b".to_string(),
                    ty: "number".to_string(),
                },
            ],
            return_type: Some("number".to_string()),
            category: "math".to_string(),
            description: "Return the maximum of two numbers".to_string(),
        });

        self.register(BuiltinFunction {
            name: "sqrt".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("f64".to_string()),
            category: "math".to_string(),
            description: "Return the square root of a number".to_string(),
        });

        self.register(BuiltinFunction {
            name: "pow".to_string(),
            params: vec![
                BuiltinParam {
                    name: "base".to_string(),
                    ty: "number".to_string(),
                },
                BuiltinParam {
                    name: "exp".to_string(),
                    ty: "number".to_string(),
                },
            ],
            return_type: Some("f64".to_string()),
            category: "math".to_string(),
            description: "Return base raised to the power of exp".to_string(),
        });

        self.register(BuiltinFunction {
            name: "floor".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "math".to_string(),
            description: "Return the largest integer less than or equal to x".to_string(),
        });

        self.register(BuiltinFunction {
            name: "ceil".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "math".to_string(),
            description: "Return the smallest integer greater than or equal to x".to_string(),
        });

        self.register(BuiltinFunction {
            name: "round".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "math".to_string(),
            description: "Round x to the nearest integer".to_string(),
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
    fn test_math_functions_registered() {
        let stdlib = Stdlib::new();
        assert!(stdlib.is_builtin("abs"));
        assert!(stdlib.is_builtin("min"));
        assert!(stdlib.is_builtin("max"));
        assert!(stdlib.is_builtin("sqrt"));
        assert!(stdlib.is_builtin("pow"));
        assert!(stdlib.is_builtin("floor"));
        assert!(stdlib.is_builtin("ceil"));
        assert!(stdlib.is_builtin("round"));
    }

    #[test]
    fn test_math_function_signatures_and_return_types() {
        let stdlib = Stdlib::new();

        let abs = stdlib.get("abs").expect("abs builtin must exist");
        assert_eq!(abs.return_type.as_deref(), Some("number"));
        assert_eq!(abs.params.len(), 1);
        assert_param(abs, 0, "x", "number");

        let min = stdlib.get("min").expect("min builtin must exist");
        assert_eq!(min.return_type.as_deref(), Some("number"));
        assert_eq!(min.params.len(), 2);
        assert_param(min, 0, "a", "number");
        assert_param(min, 1, "b", "number");

        let max = stdlib.get("max").expect("max builtin must exist");
        assert_eq!(max.return_type.as_deref(), Some("number"));
        assert_eq!(max.params.len(), 2);
        assert_param(max, 0, "a", "number");
        assert_param(max, 1, "b", "number");

        let sqrt = stdlib.get("sqrt").expect("sqrt builtin must exist");
        assert_eq!(sqrt.return_type.as_deref(), Some("f64"));
        assert_eq!(sqrt.params.len(), 1);
        assert_param(sqrt, 0, "x", "number");

        let pow = stdlib.get("pow").expect("pow builtin must exist");
        assert_eq!(pow.return_type.as_deref(), Some("f64"));
        assert_eq!(pow.params.len(), 2);
        assert_param(pow, 0, "base", "number");
        assert_param(pow, 1, "exp", "number");

        let floor = stdlib.get("floor").expect("floor builtin must exist");
        assert_eq!(floor.return_type.as_deref(), Some("i64"));
        assert_eq!(floor.params.len(), 1);
        assert_param(floor, 0, "x", "number");

        let ceil = stdlib.get("ceil").expect("ceil builtin must exist");
        assert_eq!(ceil.return_type.as_deref(), Some("i64"));
        assert_eq!(ceil.params.len(), 1);
        assert_param(ceil, 0, "x", "number");

        let round = stdlib.get("round").expect("round builtin must exist");
        assert_eq!(round.return_type.as_deref(), Some("i64"));
        assert_eq!(round.params.len(), 1);
        assert_param(round, 0, "x", "number");
    }
}
