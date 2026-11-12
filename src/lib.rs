//! A trie for storing filesystem paths.
//!
//! Paths are split on `/` and inserted as a sequence of components. The trie
//! supports checking whether an exact path is present and querying all paths
//! that share a given prefix. The structure is intentionally simple and
//! stores owned strings for each component.

use std::collections::BTreeMap;

/// A trie of filesystem paths.
///
/// Each node represents a path component. The root node represents the empty
/// path. A node marked as terminal corresponds to a complete path stored in the
/// trie.
#[derive(Debug, Default, Clone)]
pub struct PathTrie {
    children: BTreeMap<String, PathTrie>,
    terminal: bool,
}

impl PathTrie {
    /// Creates an empty trie.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts `path` into the trie.
    ///
    /// The path is split on `/`. Empty components are ignored, so leading,
    /// trailing, and repeated slashes do not create distinct entries. Inserting
    /// the same path twice has no additional effect.
    pub fn insert(&mut self, path: &str) {
        let mut node = self;
        for component in path.split('/').filter(|s| !s.is_empty()) {
            node = node.children.entry(component.to_string()).or_default();
        }
        node.terminal = true;
    }

    /// Returns `true` if `path` was previously inserted.
    ///
    /// The same normalization as [`insert`](Self::insert) applies: empty
    /// components are ignored.
    pub fn contains(&self, path: &str) -> bool {
        let mut node = self;
        for component in path.split('/').filter(|s| !s.is_empty()) {
            match node.children.get(component) {
                Some(child) => node = child,
                None => return false,
            }
        }
        node.terminal
    }

    /// Returns all stored paths that have `prefix` as a path prefix.
    ///
    /// A path `p` is included if `p` starts with `prefix` followed by a `/` or
    /// if `p` equals `prefix`. The returned paths use `/` as the separator and
    /// do not include a leading slash unless the original path had one.
    ///
    /// The result is sorted lexicographically by component, which makes the
    /// output deterministic and independent of insertion order.
    pub fn prefix_search(&self, prefix: &str) -> Vec<String> {
        let components: Vec<&str> = prefix.split('/').filter(|s| !s.is_empty()).collect();
        let mut node = self;
        for component in &components {
            match node.children.get(*component) {
                Some(child) => node = child,
                None => return Vec::new(),
            }
        }

        let mut results = Vec::new();
        let mut stack: Vec<(&PathTrie, Vec<String>)> = vec![(node, components.iter().map(|s| s.to_string()).collect())];

        while let Some((current, path_components)) = stack.pop() {
            if current.terminal {
                results.push(path_components.join("/"));
            }
            // Push in reverse order so that the lexicographically smallest
            // component is processed first, yielding sorted output.
            for (component, child) in current.children.iter().rev() {
                let mut child_path = path_components.clone();
                child_path.push(component.clone());
                stack.push((child, child_path));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_trie_contains_nothing() {
        let trie = PathTrie::new();
        assert!(!trie.contains("/"));
        assert!(!trie.contains(""));
        assert!(!trie.contains("a/b"));
    }

    #[test]
    fn insert_and_contains_exact_path() {
        let mut trie = PathTrie::new();
        trie.insert("/usr/local/bin");
        assert!(trie.contains("/usr/local/bin"));
        assert!(!trie.contains("/usr/local"));
        assert!(!trie.contains("/usr/local/bin/extra"));
    }

    #[test]
    fn normalization_ignores_empty_components() {
        let mut trie = PathTrie::new();
        trie.insert("//usr//local//bin//");
        assert!(trie.contains("/usr/local/bin"));
        assert!(trie.contains("usr/local/bin"));
        assert!(trie.contains("//usr//local//bin"));
    }

    #[test]
    fn duplicate_insert_is_idempotent() {
        let mut trie = PathTrie::new();
        trie.insert("a/b");
        trie.insert("a/b");
        assert_eq!(trie.prefix_search("a"), vec!["a/b".to_string()]);
    }

    #[test]
    fn prefix_search_returns_all_descendants_sorted() {
        let mut trie = PathTrie::new();
        trie.insert("usr/local/bin");
        trie.insert("usr/local/lib");
        trie.insert("usr/bin");
        trie.insert("usr/share/doc");
        trie.insert("etc/passwd");

        let results = trie.prefix_search("usr");
        assert_eq!(
            results,
            vec![
                "usr/bin".to_string(),
                "usr/local/bin".to_string(),
                "usr/local/lib".to_string(),
                "usr/share/doc".to_string(),
            ]
        );
    }

    #[test]
    fn prefix_search_with_exact_prefix_includes_itself() {
        let mut trie = PathTrie::new();
        trie.insert("a/b");
        trie.insert("a/b/c");
        let results = trie.prefix_search("a/b");
        assert_eq!(results, vec!["a/b".to_string(), "a/b/c".to_string()]);
    }

    #[test]
    fn prefix_search_returns_empty_when_no_match() {
        let mut trie = PathTrie::new();
        trie.insert("foo/bar");
        assert!(trie.prefix_search("baz").is_empty());
        assert!(trie.prefix_search("foo/baz").is_empty());
        assert!(trie.prefix_search("foo/bar/baz").is_empty());
    }

    #[test]
    fn prefix_search_normalizes_prefix() {
        let mut trie = PathTrie::new();
        trie.insert("a/b/c");
        trie.insert("a/b/d");
        let results = trie.prefix_search("//a//b//");
        assert_eq!(results, vec!["a/b/c".to_string(), "a/b/d".to_string()]);
    }

    #[test]
    fn root_prefix_search_returns_all_paths() {
        let mut trie = PathTrie::new();
        trie.insert("x");
        trie.insert("y/z");
        let results = trie.prefix_search("");
        assert_eq!(results, vec!["x".to_string(), "y/z".to_string()]);
    }

    #[test]
    fn root_insert_and_contains() {
        let mut trie = PathTrie::new();
        trie.insert("");
        assert!(trie.contains(""));
        assert!(trie.contains("/"));
        assert!(!trie.contains("a"));
    }

    #[test]
    fn prefix_search_does_not_include_partial_component_matches() {
        let mut trie = PathTrie::new();
        trie.insert("foo/bar");
        trie.insert("foobar/baz");
        let results = trie.prefix_search("foo");
        assert_eq!(results, vec!["foo/bar".to_string()]);
    }
}
