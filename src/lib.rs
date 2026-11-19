#![forbid(unsafe_code)]
//! SigmaOS Tools Omafind - Local semantic and lexical file search engine.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Indexed document record.
#[derive(Debug, Clone)]
pub struct SearchDocument {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

/// Search match result.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub path: PathBuf,
    pub score: f32,
    pub snippet: String,
}

/// Omafind search index.
#[derive(Debug, Clone, Default)]
pub struct SearchIndex {
    docs: Vec<SearchDocument>,
    word_index: HashMap<String, Vec<usize>>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            docs: Vec::new(),
            word_index: HashMap::new(),
        }
    }

    pub fn index_doc<P: AsRef<Path>>(
        &mut self,
        path: P,
        title: &str,
        content: &str,
        embedding: Vec<f32>,
    ) {
        let idx = self.docs.len();
        let doc = SearchDocument {
            path: path.as_ref().to_path_buf(),
            title: title.to_string(),
            content: content.to_string(),
            embedding,
        };

        for word in content.split_whitespace().chain(title.split_whitespace()) {
            let clean = word
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();
            if !clean.is_empty() {
                self.word_index.entry(clean).or_default().push(idx);
            }
        }

        self.docs.push(doc);
    }

    /// Lexical keyword search.
    pub fn search_lexical(&self, query: &str) -> Vec<SearchHit> {
        let terms: Vec<String> = query
            .split_whitespace()
            .map(|s| {
                s.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect()
            })
            .filter(|s: &String| !s.is_empty())
            .collect();

        if terms.is_empty() {
            return Vec::new();
        }

        let mut doc_scores: HashMap<usize, f32> = HashMap::new();
        for term in &terms {
            if let Some(hits) = self.word_index.get(term) {
                for &doc_idx in hits {
                    *doc_scores.entry(doc_idx).or_insert(0.0) += 1.0;
                }
            }
        }

        let mut results: Vec<SearchHit> = doc_scores
            .into_iter()
            .map(|(idx, score)| {
                let doc = &self.docs[idx];
                SearchHit {
                    path: doc.path.clone(),
                    score,
                    snippet: doc.content.chars().take(80).collect(),
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Cosine similarity vector search.
    pub fn search_semantic(&self, query_emb: &[f32], top_k: usize) -> Vec<SearchHit> {
        if query_emb.is_empty() {
            return Vec::new();
        }

        let mut hits: Vec<SearchHit> = self
            .docs
            .iter()
            .map(|doc| {
                let score = cosine_similarity(query_emb, &doc.embedding);
                SearchHit {
                    path: doc.path.clone(),
                    score,
                    snippet: doc.content.chars().take(80).collect(),
                }
            })
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        hits
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_ranking_aaa() {
        // Arrange
        let mut idx = SearchIndex::new();
        idx.index_doc(
            "/home/user/beach.png",
            "Beach Vacation",
            "A happy dog playing on the sunny ocean beach sand.",
            vec![0.9, 0.1, 0.0],
        );
        idx.index_doc(
            "/home/user/lease.pdf",
            "Apartment Lease Contract",
            "Pet policy and dog rules for apartment residents.",
            vec![0.1, 0.9, 0.0],
        );

        // Act - Lexical search
        let hits = idx.search_lexical("dog beach");

        // Assert
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].path, PathBuf::from("/home/user/beach.png"));
        assert!(hits[0].score >= hits[1].score);

        // Act - Semantic search
        let sem_hits = idx.search_semantic(&[0.85, 0.15, 0.0], 1);
        assert_eq!(sem_hits[0].path, PathBuf::from("/home/user/beach.png"));
        assert!(sem_hits[0].score > 0.9);
    }
}


#[cfg(test)]
mod oma_find_extra_tests {
    use super::*;

    #[test]
    fn test_empty_query_search_aaa() {
        // Arrange
        let mut idx = SearchIndex::new();
        idx.index_doc("/a.txt", "Title", "Some content", vec![0.5]);
        // Act
        let hits_lex = idx.search_lexical("");
        let hits_sem = idx.search_semantic(&[], 5);
        // Assert
        assert!(hits_lex.is_empty());
        assert!(hits_sem.is_empty());
    }

    #[test]
    fn test_zero_vector_similarity_aaa() {
        // Arrange & Act
        let score = cosine_similarity(&[0.0, 0.0], &[0.0, 0.0]);
        // Assert
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_dimension_mismatch_similarity_aaa() {
        // Arrange & Act
        let score = cosine_similarity(&[1.0, 0.0], &[1.0, 0.0, 0.0]);
        // Assert
        assert_eq!(score, 0.0);
    }
}
