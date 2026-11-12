# path-trie

A Rust library that stores a set of filesystem paths in a trie for efficient prefix queries and deduplication.

## Usage

```rust
use path_trie::PathTrie;

fn main() {
    let mut trie = PathTrie::new();
    trie.insert("/usr/local/bin");
    trie.insert("/usr/local/lib");
    trie.insert("/usr/bin");

    assert!(trie.contains("/usr/local/bin"));
    assert!(!trie.contains("/usr/local"));

    let results = trie.prefix_search("/usr");
    assert_eq!(results, vec!["usr/bin", "usr/local/bin", "usr/local/lib"]);
}
```

## Why this library exists

When working with many filesystem paths, repeated string comparisons for prefix checks and duplicate detection become expensive. A trie splits each path into components (directories and the final file name) and stores them as a tree. This makes prefix queries proportional to the length of the prefix rather than the number of stored paths, and it naturally deduplicates identical paths.

The trade-off is memory: each path component is stored as an owned string, so the trie uses more memory than a flat `HashSet<String>`. In exchange, prefix queries are cheap and the structure makes it easy to enumerate all paths under a directory.

## Edge cases

Paths are normalized on insertion and lookup: empty components are ignored, so `//usr//local//bin` and `usr/local/bin` refer to the same entry. A prefix query for `usr` matches `usr/bin` and `usr/local/bin` but not `usrbin`. Querying the empty prefix returns every stored path. The returned paths never include a leading slash unless it was present in the original path, because leading empty components are discarded during normalization.
