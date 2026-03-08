//! Crypto helper functions inspired by common JavaScript usage.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register crypto functions.
    pub fn register_crypto_functions(&mut self) {
        self.register(BuiltinFunction {
            name: "md5".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Compute MD5 hash".to_string(),
        });

        self.register(BuiltinFunction {
            name: "sha1".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Compute SHA-1 hash".to_string(),
        });

        self.register(BuiltinFunction {
            name: "sha256".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Compute SHA-256 hash".to_string(),
        });

        self.register(BuiltinFunction {
            name: "sha512".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Compute SHA-512 hash".to_string(),
        });

        self.register(BuiltinFunction {
            name: "hmacSha256".to_string(),
            params: vec![
                BuiltinParam {
                    name: "text".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "key".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Compute HMAC-SHA256".to_string(),
        });

        self.register(BuiltinFunction {
            name: "pbkdf2".to_string(),
            params: vec![
                BuiltinParam {
                    name: "password".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "salt".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "iterations".to_string(),
                    ty: "i64".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Derive key using PBKDF2".to_string(),
        });

        self.register(BuiltinFunction {
            name: "encryptAES".to_string(),
            params: vec![
                BuiltinParam {
                    name: "plaintext".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "key".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Encrypt text with AES".to_string(),
        });

        self.register(BuiltinFunction {
            name: "decryptAES".to_string(),
            params: vec![
                BuiltinParam {
                    name: "ciphertext".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "key".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Decrypt AES cipher text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "sign".to_string(),
            params: vec![
                BuiltinParam {
                    name: "message".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "privateKey".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Sign message".to_string(),
        });

        self.register(BuiltinFunction {
            name: "verify".to_string(),
            params: vec![
                BuiltinParam {
                    name: "message".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "signature".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "publicKey".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "crypto".to_string(),
            description: "Verify signature".to_string(),
        });

        self.register(BuiltinFunction {
            name: "randomBytes".to_string(),
            params: vec![BuiltinParam {
                name: "length".to_string(),
                ty: "i64".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Generate random bytes (hex/base64 text)".to_string(),
        });

        self.register(BuiltinFunction {
            name: "randomUUID".to_string(),
            params: vec![],
            return_type: Some("string".to_string()),
            category: "crypto".to_string(),
            description: "Generate random UUID".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_functions_registered() {
        let stdlib = Stdlib::new();
        for name in [
            "md5",
            "sha1",
            "sha256",
            "sha512",
            "hmacSha256",
            "pbkdf2",
            "encryptAES",
            "decryptAES",
            "sign",
            "verify",
            "randomBytes",
            "randomUUID",
        ] {
            assert!(stdlib.is_builtin(name), "missing builtin: {name}");
        }
    }

    #[test]
    fn test_crypto_function_signatures() {
        let stdlib = Stdlib::new();

        let sha256 = stdlib.get("sha256").expect("sha256 must exist");
        assert_eq!(sha256.return_type.as_deref(), Some("string"));
        assert_eq!(sha256.params.len(), 1);

        let verify = stdlib.get("verify").expect("verify must exist");
        assert_eq!(verify.return_type.as_deref(), Some("bool"));
        assert_eq!(verify.params.len(), 3);

        let random_uuid = stdlib.get("randomUUID").expect("randomUUID must exist");
        assert_eq!(random_uuid.return_type.as_deref(), Some("string"));
        assert!(random_uuid.params.is_empty());
    }
}
