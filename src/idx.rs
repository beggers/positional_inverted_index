use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct PositionalInvertedIndex {
    index: HashMap<String, HashMap<usize, Vec<usize>>>,
}

impl PositionalInvertedIndex {
    pub fn new() -> Self {
        PositionalInvertedIndex {
            index: HashMap::new(),
        }
    }

    pub fn index_document(&mut self, doc_id: usize, content: &str) {
        let tokens = content.split_whitespace().map(|s| s.to_lowercase()).collect::<Vec<_>>();
        for (pos, token) in tokens.iter().enumerate() {
            self.index
                .entry(token.clone())
                .or_default()
                .entry(doc_id)
                .or_default()
                .push(pos);
        }
    }

    pub fn search(&self, query: &str) -> Vec<usize> {
        let tokens = query.split_whitespace().map(|s| s.to_lowercase()).collect::<Vec<_>>();

        // Doc ID -> possible start positions for the query. We'll populate this
        // with posting lists for the first token then iterate over posting
        // lists and filter.
        // 
        // We could traverse posting lists together so we
        // never actually have to access an entire posting list at once, but this
        // has worse memory access characteristics and doesn't save us anything:
        // we need to load all the required posting lists into memory (from S3
        // in the production case) anyway.
        //
        // TODO annotate posting lists with size and go smallest to largest.
        let mut possibles: HashMap<usize, Vec<usize>> = HashMap::new();
        if let Some(docs) = self.index.get(&tokens[0]) {
            for (&doc_id, positions) in docs {
                possibles.insert(doc_id, positions.clone());
            }
        } else {
            return vec![];
        }

        for (i, token) in tokens.iter().enumerate() {
            if let Some(posting_list) = self.index.get(token) {
                let mut new_possibles = HashMap::new();
                for (&candidate_doc_id, candidate_phrase_starts) in possibles.iter() {
                    if let Some(current_token_positions) = posting_list.get(&candidate_doc_id) {
                        let mut new_starts = vec![];
                        for &candidate_phrase_start in candidate_phrase_starts {
                            match current_token_positions.binary_search(&(candidate_phrase_start + i)) {
                                Ok(pos) => new_starts.push(candidate_phrase_start),
                                Err(pos) => {},
                            }
                        }
                        if !new_starts.is_empty() {
                            new_possibles.insert(candidate_doc_id, new_starts);
                        }
                    }
                }
                possibles = new_possibles;
            } else {
                return vec![];
            }
        }

        let mut results = vec![];
        for (doc_id, positions) in possibles {
            results.push(doc_id);
        }
        results.sort();
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let index = PositionalInvertedIndex::new();
        assert!(index.index.is_empty());
    }

    #[test]
    fn test_index_single_document() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "hello world");
        assert_eq!(index.index["hello"][&1], vec![0]);
        assert_eq!(index.index["world"][&1], vec![1]);
    }

    #[test]
    fn test_index_multiple_documents() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "hello world");
        index.index_document(2, "world of rust");
        assert_eq!(index.index["world"][&1], vec![1]);
        assert_eq!(index.index["world"][&2], vec![0]);
        assert_eq!(index.index["rust"][&2], vec![2]);
    }

    #[test]
    fn test_search_nonpresent_token() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "hello world");
        index.index_document(2, "world of rust");
        let results = index.search("foo");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_single_token() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "hello world");
        index.index_document(2, "world of rust");
        let results = index.search("world");
        assert_eq!(results, vec![1, 2]);
    }

    #[test]
    fn test_search_multi_token_single_result() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "hello world");
        index.index_document(2, "world of rust");
        let results = index.search("hello world");
        assert_eq!(results, vec![1]);
    }

    #[test]
    fn test_search_multi_token_multi_result_simple() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "hello world hello rust");
        index.index_document(2, "world of hell rust hello");
        index.index_document(3, "hello rust");
        let results1 = index.search("hello rust");
        assert_eq!(results1, vec![1, 3]);
        let results2 = index.search("hell");
        assert_eq!(results2, vec![2]);
    }

    fn test_search_multi_token_multi_result_complex() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "This is a longer string with more tokens than any other test case");
        index.index_document(2, "This is another long string with many more tokens so many tokens Look how many");
        index.index_document(3, "And finally we have a third document with a few tokens but still many tokens relatively");
        let results1 = index.search("many tokens");
        assert_eq!(results1, vec![2, 3]);
    }
}