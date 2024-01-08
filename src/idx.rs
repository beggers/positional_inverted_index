use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng
};
use serde::{
    Serialize,
    Deserialize
};
use std::{
    collections::HashMap,
    mem
};

#[derive(Serialize, Deserialize)]
pub enum TokenOrdering {
    TokenOrder,
    AscendingFrequencyOrder,
}

#[derive(Serialize, Deserialize)]
pub struct PositionalInvertedIndex {
    index: HashMap<String, HashMap<usize, Vec<usize>>>,
    term_frequencies: HashMap<String, usize>,
    ordering: TokenOrdering,
}

impl PositionalInvertedIndex {
    pub fn new() -> Self {
        PositionalInvertedIndex {
            index: HashMap::new(),
            term_frequencies: HashMap::new(),
            ordering: TokenOrdering::TokenOrder,
        }
    }

    pub fn with_ordering(ordering: TokenOrdering) -> Self {
        PositionalInvertedIndex {
            index: HashMap::new(),
            term_frequencies: HashMap::new(),
            ordering: ordering,
        }
    }

    pub fn index_document(&mut self, doc_id: usize, content: &str) {
        let tokens = Self::get_tokens(content);
        for (pos, token) in tokens.iter().enumerate() {
            self.index
                .entry(token.clone())
                .or_default()
                .entry(doc_id)
                .or_default()
                .push(pos);
            *self.term_frequencies.entry(token.clone()).or_insert(0) += 1;
        }
    }

    pub fn search(&self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            return vec![];
        }

        let tokens = Self::get_tokens(query);
        let tokens = self.order_tokens(&tokens);
        if tokens.is_empty() {
            return vec![];
        }

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

    fn get_tokens(content: &str) -> Vec<String> {
        content.split_whitespace()
            .map(|s| s.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    }

    fn order_tokens(&self, tokens: &Vec<String>) -> Vec<String> {
        match self.ordering {
            TokenOrdering::TokenOrder => tokens.clone(),
            TokenOrdering::AscendingFrequencyOrder => {
                let mut token_freq_pairs: Vec<(&String, &usize)> = tokens.iter()
                    .map(|t| (t, self.term_frequencies.get(t).unwrap_or(&0)))
                    .collect();
                
                token_freq_pairs.sort_by_key(|&(_, freq)| freq);
                token_freq_pairs.into_iter().map(|(token, _)| token.clone()).collect()
            },
        }
    }

    pub fn get_random_terms(&self, n: usize) -> HashMap<String, usize> {
        let mut random_terms = HashMap::new();
    
        if self.term_frequencies.is_empty() || n == 0 {
            return random_terms;
        }
    
        let mut rng = thread_rng();
        let terms: Vec<&String> = self.term_frequencies.keys().collect();
        let weights: Vec<&usize> = self.term_frequencies.values().collect();
    
        let dist = WeightedIndex::new(weights).unwrap();
    
        while random_terms.len() < n && random_terms.len() < self.term_frequencies.len() {
            let term = terms[dist.sample(&mut rng)].clone();
            *random_terms.entry(term.to_string()).or_insert(0) = self.term_frequencies[&term];
        }

        random_terms
    }

    pub fn approximate_term_list_size_in_bytes(&self) -> usize {
        // Average English word is length 4.
        let term_list_size = std::mem::size_of_val(&self.index) + &self.index.len() * (mem::size_of::<String>()+4);
        let term_frequency_list_size = std::mem::size_of_val(&self.term_frequencies) + &self.term_frequencies.len() * (mem::size_of::<String>()+mem::size_of::<usize>());
        return term_list_size + term_frequency_list_size;
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

    pub fn approximate_posting_list_sizes_in_bytes_by_term(&self) -> HashMap<String, usize> {
        let mut sizes = HashMap::new();
        for (term, posting_list) in &self.index {
            let mut size = 0;
            for (_doc_id, positions) in posting_list {
                // Add 1 to account for the doc ID.
                size += (positions.len() + 1) * mem::size_of::<usize>();
            }
            sizes.insert(term.clone(), size);
        }
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

    #[test]
    fn test_get_random_terms_count() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "apple orange banana");
        index.index_document(2, "apple banana");

        let random_terms = index.get_random_terms(2);
        assert_eq!(random_terms.len(), 2);
    }

    #[test]
    fn test_get_random_terms_distribution() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "apple apple apple orange banana");
        index.index_document(2, "banana apple");

        let mut apple_count = 0;
        let total_count = 1000;
        for _ in 0..total_count {
            let random_terms = index.get_random_terms(1);
            if random_terms.contains_key(&"apple".to_string()) {
                apple_count += 1;
            }
        }

        assert!(apple_count > total_count / 3);
    }

    #[test]
    fn test_get_random_terms_correct_weights() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "apple apple apple orange banana");
        index.index_document(2, "banana apple");

        let random_terms = index.get_random_terms(10);

        assert!(random_terms["apple"] == 4);
        assert!(random_terms["orange"] == 1);
        assert!(random_terms["banana"] == 2);
    }

    #[test]
    fn test_get_random_terms_with_empty_index() {
        let index = PositionalInvertedIndex::new();
        let random_terms = index.get_random_terms(2);
        assert!(random_terms.is_empty());
    }

    #[test]
    fn test_get_random_terms_more_than_unique_terms() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "apple orange");

        let random_terms = index.get_random_terms(5);
        assert_eq!(random_terms.len(), 2);
    }

    #[test]
    fn test_posting_list_sizes_by_term_empty_index() {
        let index = PositionalInvertedIndex::new();
        assert!(index.approximate_posting_list_sizes_in_bytes_by_term().is_empty());
    }

    #[test]
    fn test_posting_list_sizes_by_term_single_term_index() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "term1");
        let sizes = index.approximate_posting_list_sizes_in_bytes_by_term();
        assert!(sizes.get("term1").unwrap() > &(0 as usize));
    }

    #[test]
    fn test_posting_list_sizes_by_term_multiple_terms() {
        let mut index = PositionalInvertedIndex::new();
        index.index_document(1, "apple orange");
        index.index_document(1, "apple orange banana");
        let sizes = index.approximate_posting_list_sizes_in_bytes_by_term();
        assert_eq!(sizes.get("apple").unwrap(), sizes.get("orange").unwrap());
        assert!(sizes.get("apple").unwrap() > sizes.get("banana").unwrap());
    }

    #[test]
    fn test_get_tokens_with_regular_text() {
        let content = "Hello world";
        let tokens = PositionalInvertedIndex::get_tokens(content);
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_get_tokens_with_special_characters() {
        let content = "Hello, world!";
        let tokens = PositionalInvertedIndex::get_tokens(content);
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_get_tokens_with_numbers() {
        let content = "2024 is the year";
        let tokens = PositionalInvertedIndex::get_tokens(content);
        assert_eq!(tokens, vec!["2024", "is", "the", "year"]);
    }

    #[test]
    fn test_get_tokens_with_mixed_characters() {
        let content = "Email@example.com is an,,, e-mail address!";
        let tokens = PositionalInvertedIndex::get_tokens(content);
        assert_eq!(tokens, vec!["emailexamplecom", "is", "an", "email", "address"]);
    }

    #[test]
    fn test_get_tokens_with_empty_string() {
        let content = "";
        let tokens = PositionalInvertedIndex::get_tokens(content);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_get_tokens_with_whitespace_only() {
        let content = "   ";
        let tokens = PositionalInvertedIndex::get_tokens(content);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_order_tokens_token_order() {
        let index = PositionalInvertedIndex::with_ordering(TokenOrdering::TokenOrder);
        let tokens = vec!["apple".to_string(), "banana".to_string(), "apple".to_string()];
        let ordered_tokens = index.order_tokens(&tokens);
        assert_eq!(ordered_tokens, tokens);
    }

    #[test]
    fn test_order_tokens_ascending_frequency_order() {
        let mut index = PositionalInvertedIndex::with_ordering(TokenOrdering::AscendingFrequencyOrder);

        // Index some documents to create frequencies
        index.index_document(1, "apple apple apple apple apple cherry");
        index.index_document(2, "banana cherry cherry");

        let tokens = vec!["apple".to_string(), "cherry".to_string(), "banana".to_string()];
        let ordered_tokens = index.order_tokens(&tokens);
        assert_eq!(ordered_tokens, vec!["banana".to_string(), "cherry".to_string(), "apple".to_string()]);
    }
}