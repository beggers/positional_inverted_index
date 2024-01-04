use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::fs;
use std::mem;
use regex::Regex;

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
                                Ok(_pos) => new_starts.push(candidate_phrase_start),
                                Err(_pos) => {},
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
        for (doc_id, _positions) in possibles {
            results.push(doc_id);
        }
        results.sort();
        results
    }

    pub fn index_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let contents = fs::read_to_string(file_path)?;
        let re = Regex::new(r"\n\s*\n")?;
        let paragraphs = re.split(&contents);
        let mut hasher = DefaultHasher::new();
        for paragraph in paragraphs {
            paragraph.hash(&mut hasher);
            let hash = hasher.finish();
            self.index_document(hash as usize, paragraph);
        }
        Ok(())
    }

    pub fn approximate_term_list_size_in_bytes(&self) -> usize {
        // Average English word is length 4.
        return std::mem::size_of_val(&self.index) + &self.index.len() * (mem::size_of::<String>()+4);
    }

    pub fn approximate_posting_list_sizes_in_bytes(&self) -> Vec<usize> {
        let mut sizes = vec![];
        for (_term, posting_list) in &self.index {
            let mut size = 0;
            for (_doc_id, positions) in posting_list {
                // Add 1 to account for the doc ID.
                size += (positions.len() + 1) * mem::size_of::<usize>();
            }
            sizes.push(size);
        }
        sizes.sort();
        sizes
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

    #[test]
    fn test_search_multi_token_multi_result_complex() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "This is a longer string with more tokens than any other test case");
        index.index_document(2, "This is another long string with many more tokens so many tokens Look how many");
        index.index_document(3, "And finally we have a third document with a few tokens but still many tokens relatively");
        let results1 = index.search("many tokens");
        assert_eq!(results1, vec![2, 3]);
    }

    #[test]
    fn test_index_file() {
        let mut index = PositionalInvertedIndex::new();
        let test_file_path = "test_data/3_paragraphs.txt"; // Replace with the actual path to your test file
        assert!(index.index_file(test_file_path).is_ok());
        for (_term, posting_list) in &index.index {
            assert!(posting_list.len() <= 3);
        }
    }

    #[test]
    fn test_empty_index_term_list_size() {
        let index = PositionalInvertedIndex::new();
        assert!(index.approximate_term_list_size_in_bytes() > 0);
        assert!(index.approximate_term_list_size_in_bytes() < 100);
    }

    #[test]
    fn test_increasing_size_increases_term_list_size() {
        let mut index = PositionalInvertedIndex::new();
        let initial_size = index.approximate_term_list_size_in_bytes();

        index.index_document(1, "test document one");
        let first_size = index.approximate_term_list_size_in_bytes();
        assert!(first_size > initial_size);

        index.index_document(2, "another test document");
        let second_size = index.approximate_term_list_size_in_bytes();
        assert!(second_size > first_size);
    }

    #[test]
    fn test_term_list_size_is_reasonable_for_large_index() {
        let mut index = PositionalInvertedIndex::new();
        for i in 1..=1000 {
            index.index_document(i, "some repetitive test document content");
        }

        let size = index.approximate_term_list_size_in_bytes();
        assert!(size < 1000000);
    }

    #[test]
    fn test_empty_index_posting_list_sizes() {
        let index = PositionalInvertedIndex::new();
        assert!(index.approximate_posting_list_sizes_in_bytes().is_empty());
    }

    #[test]
    fn test_single_term_posting_list_size() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "test");
        let sizes = index.approximate_posting_list_sizes_in_bytes();
        assert_eq!(sizes.len(), 1);
        assert!(sizes[0] > 0);
    }

    #[test]
    fn test_multiple_terms_correct_number_of_posting_list_sizes() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "test document");
        index.index_document(2, "another test document");
        let sizes = index.approximate_posting_list_sizes_in_bytes();
        assert_eq!(sizes.len(), 3);
    }

    #[test]
    fn test_multiple_documents_multiple_terms_correct_number_of_posting_list_sizes() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "test document");
        index.index_document(2, "another test document");

        let sizes = index.approximate_posting_list_sizes_in_bytes();
        assert_eq!(sizes.len(), 3);
    }

    #[test]
    fn test_posting_list_sizes_sorted() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "test document");
        index.index_document(2, "another test document");

        let sizes = index.approximate_posting_list_sizes_in_bytes();
        assert!(sizes[0] <= sizes[1]);
        assert!(sizes[1] <= sizes[2]);
    }

    #[test]
    fn test_increasing_size_increases_posting_list_sizes() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "a document");
        index.index_document(2, "a bit longer document");

        let initial_sizes = index.approximate_posting_list_sizes_in_bytes();
        assert!(initial_sizes[0] <= initial_sizes[1]);

        index.index_document(3, "a bit longer document");
        index.index_document(4, "a bit longer document");

        let final_sizes = index.approximate_posting_list_sizes_in_bytes();

        for i in 0..3 {
            assert!(initial_sizes[i] < final_sizes[i]);
        }
    }
}