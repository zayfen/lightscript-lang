//! Networking helper functions inspired by common JavaScript usage.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register networking functions.
    pub fn register_net_functions(&mut self) {
        self.register(BuiltinFunction {
            name: "fetch".to_string(),
            params: vec![BuiltinParam {
                name: "url".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "net".to_string(),
            description: "Perform HTTP GET and return body".to_string(),
        });

        self.register(BuiltinFunction {
            name: "httpGet".to_string(),
            params: vec![BuiltinParam {
                name: "url".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "net".to_string(),
            description: "HTTP GET request".to_string(),
        });

        self.register(BuiltinFunction {
            name: "httpPost".to_string(),
            params: vec![
                BuiltinParam {
                    name: "url".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "body".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "net".to_string(),
            description: "HTTP POST request".to_string(),
        });

        self.register(BuiltinFunction {
            name: "httpPut".to_string(),
            params: vec![
                BuiltinParam {
                    name: "url".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "body".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "net".to_string(),
            description: "HTTP PUT request".to_string(),
        });

        self.register(BuiltinFunction {
            name: "httpDelete".to_string(),
            params: vec![BuiltinParam {
                name: "url".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "net".to_string(),
            description: "HTTP DELETE request".to_string(),
        });

        self.register(BuiltinFunction {
            name: "download".to_string(),
            params: vec![
                BuiltinParam {
                    name: "url".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "path".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "net".to_string(),
            description: "Download URL content to file".to_string(),
        });

        self.register(BuiltinFunction {
            name: "upload".to_string(),
            params: vec![
                BuiltinParam {
                    name: "url".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "path".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "net".to_string(),
            description: "Upload file to URL".to_string(),
        });

        self.register(BuiltinFunction {
            name: "websocketConnect".to_string(),
            params: vec![BuiltinParam {
                name: "url".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "net".to_string(),
            description: "Connect websocket endpoint".to_string(),
        });

        self.register(BuiltinFunction {
            name: "dnsLookup".to_string(),
            params: vec![BuiltinParam {
                name: "host".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "net".to_string(),
            description: "Resolve host to IP".to_string(),
        });

        self.register(BuiltinFunction {
            name: "ping".to_string(),
            params: vec![BuiltinParam {
                name: "host".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "net".to_string(),
            description: "Check host reachability".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_functions_registered() {
        let stdlib = Stdlib::new();
        for name in [
            "fetch",
            "httpGet",
            "httpPost",
            "httpPut",
            "httpDelete",
            "download",
            "upload",
            "websocketConnect",
            "dnsLookup",
            "ping",
        ] {
            assert!(stdlib.is_builtin(name), "missing builtin: {name}");
        }
    }

    #[test]
    fn test_net_function_signatures() {
        let stdlib = Stdlib::new();

        let fetch = stdlib.get("fetch").expect("fetch must exist");
        assert_eq!(fetch.return_type.as_deref(), Some("string"));
        assert_eq!(fetch.params.len(), 1);

        let post = stdlib.get("httpPost").expect("httpPost must exist");
        assert_eq!(post.return_type.as_deref(), Some("string"));
        assert_eq!(post.params.len(), 2);

        let download = stdlib.get("download").expect("download must exist");
        assert_eq!(download.return_type.as_deref(), Some("bool"));
        assert_eq!(download.params.len(), 2);

        let ping = stdlib.get("ping").expect("ping must exist");
        assert_eq!(ping.return_type.as_deref(), Some("bool"));
        assert_eq!(ping.params.len(), 1);
    }
}
