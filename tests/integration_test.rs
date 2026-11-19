use sigmaos_tools_omafind::{SearchIndex, SearchHit};
use std::path::PathBuf;

#[test]
fn test_multidoc_lexical_search_aaa() {
    // Arrange
    let mut index = SearchIndex::new();
    index.index_doc("/docs/arch.md", "Architecture", "Microkernel and capability IPC design", vec![0.1, 0.2]);
    index.index_doc("/docs/gui.md", "Desktop GUI", "Wayland compositor and Catppuccin theme", vec![0.3, 0.4]);

    // Act
    let results = index.search_lexical("capability IPC");

    // Assert
    assert!(!results.is_empty());
    assert_eq!(results[0].path, PathBuf::from("/docs/arch.md"));
}

#[test]
fn test_semantic_vector_ranking_aaa() {
    // Arrange
    let mut index = SearchIndex::new();
    index.index_doc("/a.rs", "Math A", "Vector math", vec![1.0, 0.0, 0.0]);
    index.index_doc("/b.rs", "Math B", "Tensor math", vec![0.0, 1.0, 0.0]);

    // Act
    let hits = index.search_semantic(&[0.9, 0.1, 0.0], 2);

    // Assert
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].path, PathBuf::from("/a.rs"));
    assert!(hits[0].score > hits[1].score);
}
