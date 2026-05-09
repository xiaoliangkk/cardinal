use search_cache::{SearchCache, SearchOptions};
use search_cancel::CancellationToken;
use std::fs;
use tempdir::TempDir;

#[test]
fn content_filter_rejects_empty_needle() {
    let temp_dir = TempDir::new("content_empty_needle").unwrap();
    fs::write(temp_dir.path().join("file.txt"), b"content").unwrap();

    let mut cache = SearchCache::walk_fs(temp_dir.path());
    let result = cache.search_with_options(
        r#"content:"""#,
        SearchOptions {
            case_insensitive: false,
        },
        CancellationToken::noop(),
    );

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("content: requires a value")
    );
}

#[test]
fn content_filter_rejects_unsupported_spotlight_characters() {
    let temp_dir = TempDir::new("content_unsupported_chars").unwrap();
    fs::write(temp_dir.path().join("file.txt"), b"content").unwrap();

    for query in [
        "content:foo*bar",
        r#"content:"foo'bar""#,
        r#"content:"foo\bar""#,
    ] {
        let mut cache = SearchCache::walk_fs(temp_dir.path());
        let result = cache.search_with_options(
            query,
            SearchOptions {
                case_insensitive: false,
            },
            CancellationToken::noop(),
        );

        assert!(result.is_err(), "{query} should fail");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("content filter contains unsupported character"),
            "{query} should report unsupported Spotlight syntax"
        );
    }
}
