use rand::{distributions::{Distribution, WeightedIndex}, prelude::SliceRandom, thread_rng, Rng};
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum QueryTokenDistribution {
    Fixed,
    Uniform,
    FromDocument
}

pub fn generate_queries_from_fixed_dictionary(num_queries: usize, max_tokens: usize) -> Vec<String> {
    let dictionary = [
        "The", "quantity", "respectable", "she", "announced"
    ];

    let mut rng = thread_rng();
    let mut queries = Vec::with_capacity(num_queries);

    for _ in 0..num_queries {
        let query_length = rng.gen_range(1..=max_tokens);
        let query = dictionary
            .choose_multiple(&mut rng, query_length)
            .cloned()
            .collect::<Vec<&str>>()
            .join(" ");
        queries.push(query);
    }

    queries
}

pub fn generate_queries_from_distribution(num_queries: usize, max_tokens: usize, terms: &HashMap<String, usize>) -> Vec<String> {
    if terms.is_empty() {
        return vec![];
    }

    let mut rng = rand::thread_rng();
    let mut queries = Vec::with_capacity(num_queries);

    let (terms, weights): (Vec<_>, Vec<_>) = terms.iter().map(|(term, &weight)| (term.as_str(), weight)).unzip();

    let dist = WeightedIndex::new(&weights).unwrap();

    for _ in 0..num_queries {
        let query_length = rng.gen_range(1..=max_tokens);
        let query = (0..query_length)
            .map(|_| terms[dist.sample(&mut rng)])
            .collect::<Vec<&str>>()
            .join(" ");
        queries.push(query);
    }

    queries
}

pub fn pull_query_from_paragraph(paragraph: &str, num_queries: usize, max_tokens: usize) -> Vec<String> {
    if paragraph.is_empty() || num_queries == 0 {
        return vec![];
    }

    let tokens: Vec<&str> = paragraph.split_whitespace().collect();
    let token_count = tokens.len();
    let max_query_length = max_tokens.min(token_count);

    if token_count == 0 {
        return vec![];
    }

    let mut rng = rand::thread_rng();
    let mut queries = Vec::with_capacity(num_queries);

    for _ in 0..num_queries {
        let query_length = rng.gen_range(1..=max_query_length);
        let start_index = rng.gen_range(0..token_count - query_length + 1);
        let selected_tokens = &tokens[start_index..start_index + query_length];
        queries.push(selected_tokens.join(" "));
    }

    queries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_dictionary_generate_correct_number_of_queries() {
        let num_queries = 10;
        let max_tokens = 5;
        let queries = generate_queries_from_fixed_dictionary(num_queries, max_tokens);

        assert_eq!(queries.len(), num_queries);
    }

    #[test]
    fn test_fixed_dictionary_query_length_within_range() {
        let num_queries = 10;
        let max_tokens = 3;
        let queries = generate_queries_from_fixed_dictionary(num_queries, max_tokens);

        for query in queries {
            let token_count = query.split_whitespace().count();
            assert!(token_count > 0 && token_count <= max_tokens);
        }
    }

    #[test]
    fn test_fixed_dictionary_queries_not_empty() {
        let num_queries = 10;
        let max_tokens = 3;
        let queries = generate_queries_from_fixed_dictionary(num_queries, max_tokens);

        for query in queries {
            assert!(!query.is_empty());
        }
    }

    #[test]
    fn test_distribution_basic_functionality() {
        let terms = HashMap::from([("term1".to_string(), 1), ("term2".to_string(), 1)]);
        let queries = generate_queries_from_distribution(5, 3, &terms);
        assert_eq!(queries.len(), 5);
        for query in queries {
            assert!(query.split_whitespace().count() <= 3);
        }
    }

    #[test]
    fn test_distribution_empty_terms() {
        let terms = HashMap::new();
        let queries = generate_queries_from_distribution(5, 3, &terms);
        assert_eq!(queries.len(), 0);
    }

    #[test]
    fn test_distribution_single_term() {
        let terms = HashMap::from([("single_term".to_string(), 1)]);
        let queries = generate_queries_from_distribution(5, 3, &terms);
        let possible_results = vec!["single_term", "single_term single_term", "single_term single_term single_term"];
        for query in queries {
            assert!(possible_results.contains(&query.as_str()));
        }
    }

    #[test]
    fn test_distribution_uniform_weights() {
        let terms = HashMap::from([("term1".to_string(), 1), ("term2".to_string(), 1)]);
        let mut term_counts = HashMap::new();

        for _ in 0..1000 {
            let queries = generate_queries_from_distribution(10, 2, &terms);
            for query in queries {
                for term in query.split_whitespace() {
                    let term = term.to_string();
                    *term_counts.entry(term).or_insert(0) += 1;
                }
            }
        }

        let counts: Vec<usize> = term_counts.values().cloned().collect();
        let max_count = *counts.iter().max().unwrap();
        let min_count = *counts.iter().min().unwrap();
        assert!(max_count - min_count < max_count / 10);
    }

    #[test]
    fn test_distribution_varying_weights() {
        let terms = HashMap::from([("common".to_string(), 10), ("rare".to_string(), 1)]);
        let mut term_counts = HashMap::new();

        for _ in 0..1000 {
            let queries = generate_queries_from_distribution(10, 2, &terms);
            for query in queries {
                for term in query.split_whitespace() {
                    let term = term.to_string();
                    *term_counts.entry(term).or_insert(0) += 1;
                }
            }
        }

        let common_count = *term_counts.get("common").unwrap_or(&0);
        let rare_count = *term_counts.get("rare").unwrap_or(&0);
        assert!(common_count > rare_count);
    }

    #[test]
    fn test_query_from_paragraph_basic_functionality() {
        let paragraph = "This is a test paragraph with several words";
        let queries = pull_query_from_paragraph(paragraph, 3, 4);
        assert_eq!(queries.len(), 3);
        for query in queries {
            println!("{}", query);
            assert!(paragraph.contains(&query));
        }
    }

    #[test]
    fn test_query_from_paragraph_empty_paragraph() {
        let paragraph = "";
        let queries = pull_query_from_paragraph(paragraph, 3, 4);
        assert!(queries.iter().all(|query| query.is_empty()));
    }

    #[test]
    fn test_query_from_paragraph_max_tokens_exceeding_paragraph_length() {
        let paragraph = "Short paragraph";
        let queries = pull_query_from_paragraph(paragraph, 3, 10);
        for query in queries {
            assert!(query.split_whitespace().count() <= 2);
            assert!(paragraph.contains(&query));
        }
    }

    #[test]
    fn test_query_from_paragraph_num_queries_zero() {
        let paragraph = "This is a test paragraph with several words";
        let queries = pull_query_from_paragraph(paragraph, 0, 4);
        assert!(queries.is_empty());
    }
}