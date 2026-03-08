//! Encoding/decoding helper functions.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register encoding helpers.
    pub fn register_encoding_functions(&mut self) {
        self.register(BuiltinFunction {
            name: "base64Encode".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Encode text with Base64".to_string(),
        });

        self.register(BuiltinFunction {
            name: "base64Decode".to_string(),
            params: vec![BuiltinParam {
                name: "base64".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Decode Base64 text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "hexEncode".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Encode text as hex".to_string(),
        });

        self.register(BuiltinFunction {
            name: "hexDecode".to_string(),
            params: vec![BuiltinParam {
                name: "hex".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Decode hex text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "urlEncode".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Percent-encode URL component".to_string(),
        });

        self.register(BuiltinFunction {
            name: "urlDecode".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Decode percent-encoded text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "utf8Encode".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("array".to_string()),
            category: "encoding".to_string(),
            description: "Encode string to UTF-8 byte array".to_string(),
        });

        self.register(BuiltinFunction {
            name: "utf8Decode".to_string(),
            params: vec![BuiltinParam {
                name: "bytes".to_string(),
                ty: "array".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Decode UTF-8 byte array to string".to_string(),
        });

        self.register(BuiltinFunction {
            name: "csvEncode".to_string(),
            params: vec![BuiltinParam {
                name: "rows".to_string(),
                ty: "array".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Encode rows to CSV text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "csvDecode".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("array".to_string()),
            category: "encoding".to_string(),
            description: "Decode CSV text to rows".to_string(),
        });

        self.register(BuiltinFunction {
            name: "queryStringify".to_string(),
            params: vec![BuiltinParam {
                name: "obj".to_string(),
                ty: "any".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "encoding".to_string(),
            description: "Serialize object as URL query string".to_string(),
        });

        self.register(BuiltinFunction {
            name: "queryParse".to_string(),
            params: vec![BuiltinParam {
                name: "query".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("any".to_string()),
            category: "encoding".to_string(),
            description: "Parse URL query string".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_functions_registered() {
        let stdlib = Stdlib::new();
        for name in [
            "base64Encode",
            "base64Decode",
            "hexEncode",
            "hexDecode",
            "urlEncode",
            "urlDecode",
            "utf8Encode",
            "utf8Decode",
            "csvEncode",
            "csvDecode",
            "queryStringify",
            "queryParse",
        ] {
            assert!(stdlib.is_builtin(name), "missing builtin: {name}");
        }
    }

    #[test]
    fn test_encoding_function_signatures() {
        let stdlib = Stdlib::new();

        let base64_encode = stdlib.get("base64Encode").expect("base64Encode must exist");
        assert_eq!(base64_encode.return_type.as_deref(), Some("string"));
        assert_eq!(base64_encode.params.len(), 1);

        let utf8_encode = stdlib.get("utf8Encode").expect("utf8Encode must exist");
        assert_eq!(utf8_encode.return_type.as_deref(), Some("array"));

        let csv_decode = stdlib.get("csvDecode").expect("csvDecode must exist");
        assert_eq!(csv_decode.return_type.as_deref(), Some("array"));

        let query_parse = stdlib.get("queryParse").expect("queryParse must exist");
        assert_eq!(query_parse.return_type.as_deref(), Some("any"));
    }
}
