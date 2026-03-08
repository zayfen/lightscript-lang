//! IO functions for Ziv standard library.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register IO functions.
    pub fn register_io_functions(&mut self) {
        self.register(BuiltinFunction {
            name: "print".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: None,
            category: "io".to_string(),
            description: "Print a value to stdout without newline".to_string(),
        });

        self.register(BuiltinFunction {
            name: "println".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: None,
            category: "io".to_string(),
            description: "Print a value to stdout with newline".to_string(),
        });

        self.register(BuiltinFunction {
            name: "read".to_string(),
            params: vec![],
            return_type: Some("string".to_string()),
            category: "io".to_string(),
            description: "Read a line from stdin".to_string(),
        });

        self.register(BuiltinFunction {
            name: "eprint".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: None,
            category: "io".to_string(),
            description: "Print a value to stderr without newline".to_string(),
        });

        self.register(BuiltinFunction {
            name: "eprintln".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: None,
            category: "io".to_string(),
            description: "Print a value to stderr with newline".to_string(),
        });

        self.register(BuiltinFunction {
            name: "input".to_string(),
            params: vec![BuiltinParam {
                name: "prompt".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "io".to_string(),
            description: "Read one line from stdin with prompt".to_string(),
        });

        self.register(BuiltinFunction {
            name: "readAll".to_string(),
            params: vec![],
            return_type: Some("string".to_string()),
            category: "io".to_string(),
            description: "Read all remaining stdin content".to_string(),
        });

        self.register(BuiltinFunction {
            name: "printf".to_string(),
            params: vec![
                BuiltinParam {
                    name: "format".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "value".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: None,
            category: "io".to_string(),
            description: "Formatted output to stdout".to_string(),
        });

        self.register(BuiltinFunction {
            name: "flush".to_string(),
            params: vec![],
            return_type: None,
            category: "io".to_string(),
            description: "Flush stdout buffer".to_string(),
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
    fn test_io_functions_registered() {
        let stdlib = Stdlib::new();
        assert!(stdlib.is_builtin("print"));
        assert!(stdlib.is_builtin("println"));
        assert!(stdlib.is_builtin("read"));
        assert!(stdlib.is_builtin("eprint"));
        assert!(stdlib.is_builtin("eprintln"));
        assert!(stdlib.is_builtin("input"));
        assert!(stdlib.is_builtin("readAll"));
        assert!(stdlib.is_builtin("printf"));
        assert!(stdlib.is_builtin("flush"));
    }

    #[test]
    fn test_io_function_signatures_and_metadata() {
        let stdlib = Stdlib::new();

        let print = stdlib.get("print").expect("print builtin must exist");
        assert_eq!(print.return_type, None);
        assert_eq!(print.params.len(), 1);
        assert_param(print, 0, "value", "any");
        assert!(print.description.contains("stdout"));

        let println = stdlib.get("println").expect("println builtin must exist");
        assert_eq!(println.return_type, None);
        assert_eq!(println.params.len(), 1);
        assert_param(println, 0, "value", "any");
        assert!(println.description.contains("newline"));

        let read = stdlib.get("read").expect("read builtin must exist");
        assert_eq!(read.return_type.as_deref(), Some("string"));
        assert!(read.params.is_empty());

        let eprint = stdlib.get("eprint").expect("eprint builtin must exist");
        assert_eq!(eprint.return_type, None);
        assert_eq!(eprint.params.len(), 1);
        assert_param(eprint, 0, "value", "any");
        assert!(eprint.description.contains("stderr"));

        let eprintln = stdlib.get("eprintln").expect("eprintln builtin must exist");
        assert_eq!(eprintln.return_type, None);
        assert_eq!(eprintln.params.len(), 1);
        assert_param(eprintln, 0, "value", "any");
        assert!(eprintln.description.contains("stderr"));

        let input = stdlib.get("input").expect("input builtin must exist");
        assert_eq!(input.return_type.as_deref(), Some("string"));
        assert_eq!(input.params.len(), 1);
        assert_param(input, 0, "prompt", "string");

        let read_all = stdlib.get("readAll").expect("readAll builtin must exist");
        assert_eq!(read_all.return_type.as_deref(), Some("string"));
        assert!(read_all.params.is_empty());

        let printf = stdlib.get("printf").expect("printf builtin must exist");
        assert_eq!(printf.return_type, None);
        assert_eq!(printf.params.len(), 2);
        assert_param(printf, 0, "format", "string");
        assert_param(printf, 1, "value", "any");

        let flush = stdlib.get("flush").expect("flush builtin must exist");
        assert_eq!(flush.return_type, None);
        assert!(flush.params.is_empty());
    }
}
