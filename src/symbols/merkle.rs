use sha2::{Digest, Sha256};

use super::SymbolNode;

/// Compute content hash from the raw source text of a symbol.
/// Normalizes whitespace to make hashing resilient to formatting changes.
pub fn content_hash(source: &str) -> [u8; 32] {
    let normalized = normalize_source(source);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    hasher.finalize().into()
}

/// Compute the Merkle hash for a symbol node.
/// Combines the node's own content hash with all children's Merkle hashes.
/// This must be called bottom-up (children first).
pub fn compute_merkle_hash(node: &mut SymbolNode) {
    // First, recursively compute children's merkle hashes.
    for child in node.children.iter_mut() {
        compute_merkle_hash(child);
    }

    let mut hasher = Sha256::new();
    hasher.update(node.content_hash);
    for child in &node.children {
        hasher.update(child.merkle_hash);
    }
    node.merkle_hash = hasher.finalize().into();
}

/// Normalize source code for hashing: collapse runs of whitespace into single spaces,
/// trim leading/trailing whitespace. This makes the hash resilient to formatting changes
/// while still detecting meaningful code changes.
fn normalize_source(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut prev_was_space = false;

    for ch in source.chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }

    result.trim().to_string()
}

/// Estimate the number of tokens a source string would consume.
/// Rough approximation: ~3.5 characters per token for code.
pub fn estimate_tokens(source: &str) -> usize {
    (source.len() as f64 / 3.5).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_collapses_whitespace() {
        let input = "fn  foo(\n    x: i32,\n    y: i32\n) -> i32 {\n    x + y\n}";
        let normalized = normalize_source(input);
        assert_eq!(normalized, "fn foo( x: i32, y: i32 ) -> i32 { x + y }");
    }

    #[test]
    fn test_content_hash_deterministic() {
        let h1 = content_hash("fn foo() {}");
        let h2 = content_hash("fn foo() {}");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_content_hash_whitespace_insensitive() {
        let h1 = content_hash("fn foo() { }");
        let h2 = content_hash("fn  foo()  {  }");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_content_hash_detects_changes() {
        let h1 = content_hash("fn foo() {}");
        let h2 = content_hash("fn bar() {}");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_estimate_tokens() {
        // "fn foo() {}" is 11 chars → ceil(11/3.5) = 4
        assert_eq!(estimate_tokens("fn foo() {}"), 4);
    }
}
