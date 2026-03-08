//! Filesystem helper functions inspired by common JavaScript usage.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register filesystem helpers.
    pub fn register_filesystem_functions(&mut self) {
        self.register(BuiltinFunction {
            name: "readFile".to_string(),
            params: vec![
                BuiltinParam {
                    name: "path".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "encoding".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "filesystem".to_string(),
            description: "Read text file content".to_string(),
        });

        self.register(BuiltinFunction {
            name: "writeFile".to_string(),
            params: vec![
                BuiltinParam {
                    name: "path".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "content".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "filesystem".to_string(),
            description: "Write text to file".to_string(),
        });

        self.register(BuiltinFunction {
            name: "appendFile".to_string(),
            params: vec![
                BuiltinParam {
                    name: "path".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "content".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "filesystem".to_string(),
            description: "Append text to file".to_string(),
        });

        self.register(BuiltinFunction {
            name: "exists".to_string(),
            params: vec![BuiltinParam {
                name: "path".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "filesystem".to_string(),
            description: "Check whether path exists".to_string(),
        });

        self.register(BuiltinFunction {
            name: "mkdir".to_string(),
            params: vec![BuiltinParam {
                name: "path".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "filesystem".to_string(),
            description: "Create directory".to_string(),
        });

        self.register(BuiltinFunction {
            name: "readDir".to_string(),
            params: vec![BuiltinParam {
                name: "path".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("array".to_string()),
            category: "filesystem".to_string(),
            description: "List directory entries".to_string(),
        });

        self.register(BuiltinFunction {
            name: "removeFile".to_string(),
            params: vec![BuiltinParam {
                name: "path".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "filesystem".to_string(),
            description: "Remove file".to_string(),
        });

        self.register(BuiltinFunction {
            name: "removeDir".to_string(),
            params: vec![BuiltinParam {
                name: "path".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "filesystem".to_string(),
            description: "Remove directory".to_string(),
        });

        self.register(BuiltinFunction {
            name: "rename".to_string(),
            params: vec![
                BuiltinParam {
                    name: "from".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "to".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "filesystem".to_string(),
            description: "Rename file or directory".to_string(),
        });

        self.register(BuiltinFunction {
            name: "copyFile".to_string(),
            params: vec![
                BuiltinParam {
                    name: "src".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "dst".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "filesystem".to_string(),
            description: "Copy file".to_string(),
        });

        self.register(BuiltinFunction {
            name: "fileSize".to_string(),
            params: vec![BuiltinParam {
                name: "path".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "filesystem".to_string(),
            description: "Return file size in bytes".to_string(),
        });

        self.register(BuiltinFunction {
            name: "cwd".to_string(),
            params: vec![],
            return_type: Some("string".to_string()),
            category: "filesystem".to_string(),
            description: "Return current working directory".to_string(),
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
    fn test_filesystem_functions_registered() {
        let stdlib = Stdlib::new();
        for name in [
            "readFile",
            "writeFile",
            "appendFile",
            "exists",
            "mkdir",
            "readDir",
            "removeFile",
            "removeDir",
            "rename",
            "copyFile",
            "fileSize",
            "cwd",
        ] {
            assert!(stdlib.is_builtin(name), "missing builtin: {name}");
        }
    }

    #[test]
    fn test_filesystem_function_signatures() {
        let stdlib = Stdlib::new();

        let read_file = stdlib.get("readFile").expect("readFile must exist");
        assert_eq!(read_file.return_type.as_deref(), Some("string"));
        assert_eq!(read_file.params.len(), 2);
        assert_param(read_file, 0, "path", "string");
        assert_param(read_file, 1, "encoding", "string");

        let write_file = stdlib.get("writeFile").expect("writeFile must exist");
        assert_eq!(write_file.return_type.as_deref(), Some("bool"));
        assert_eq!(write_file.params.len(), 2);

        let read_dir = stdlib.get("readDir").expect("readDir must exist");
        assert_eq!(read_dir.return_type.as_deref(), Some("array"));

        let file_size = stdlib.get("fileSize").expect("fileSize must exist");
        assert_eq!(file_size.return_type.as_deref(), Some("i64"));

        let cwd = stdlib.get("cwd").expect("cwd must exist");
        assert_eq!(cwd.return_type.as_deref(), Some("string"));
        assert!(cwd.params.is_empty());
    }
}
