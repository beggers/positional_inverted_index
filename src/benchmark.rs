use crate::PositionalInvertedIndex;

use rand::{seq::SliceRandom, thread_rng, Rng};
use regex::Regex;
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
    time::Instant,
};

pub fn benchmark_index(
    filenames: Vec<String>, 
    query_frequency: usize, 
    num_queries: usize, 
    max_query_tokens: usize,
    target_directory: &str,
) -> Result<(), Box<dyn Error>> {
    let mut index = PositionalInvertedIndex::new();

    fs::create_dir_all(target_directory)?;

    let indexing_time_file_path = Path::new(target_directory).join("indexing_time.txt");
    let search_time_file_path = Path::new(target_directory).join("search_time.txt");
    let term_list_size_file_path = Path::new(target_directory).join("term_list_size.txt");
    let posting_list_size_file_path = Path::new(target_directory).join("posting_list_size.txt");

    let mut indexing_time_file = File::create(indexing_time_file_path)?;
    let mut search_time_file = File::create(search_time_file_path)?;
    let mut term_list_size_file = File::create(term_list_size_file_path)?;
    let mut posting_list_size_file = File::create(posting_list_size_file_path)?;

    let mut paragraph_counter = 0;
    for filename in filenames {
        let paragraphs = read_file_into_paragraphs(&filename)?;

        for paragraph in paragraphs {
            if paragraph.is_empty() {
                continue;
            }

            let start = Instant::now();
            index.index_document(paragraph_counter, &paragraph);
            let duration = start.elapsed();
            writeln!(indexing_time_file, "{} {:?}", paragraph_counter, duration)?;

            if paragraph_counter % query_frequency == 0 {

                let queries = generate_queries_from_fixed_dictionary(num_queries, max_query_tokens);
                for query in queries {
                    let query_start = Instant::now();
                    index.search(&query);
                    let query_duration = query_start.elapsed();

                    let tokens_in_query = query.split_whitespace().count();
                    writeln!(search_time_file, "{} {} {:?}", paragraph_counter, tokens_in_query, query_duration)?;
                }

                let term_list_size = index.approximate_term_list_size_in_bytes();
                let posting_list_sizes = index.approximate_posting_list_sizes_in_bytes();
                writeln!(term_list_size_file, "{} {}", paragraph_counter, term_list_size)?;
                writeln!(posting_list_size_file, "{} {:?}", paragraph_counter, posting_list_sizes)?;
            }

            paragraph_counter += 1;
        }
    }

    Ok(())
}

fn read_file_into_paragraphs(filename: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let re = Regex::new(r"\n\s*\n")?;
    let paragraphs = re.split(&contents)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>();
    Ok(paragraphs)
}

fn generate_queries_from_fixed_dictionary(num_queries: usize, max_tokens: usize) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file_into_paragraphs_zero_paragraphs() {
        let filename = "test_data/0_paragraphs.txt";
        let paragraphs = read_file_into_paragraphs(filename).unwrap();
        assert_eq!(paragraphs.len(), 0);
    }

    #[test]
    fn test_read_file_into_paragraphs_single_paragraph() {
        let filename = "test_data/1_paragraph.txt";
        let paragraphs = read_file_into_paragraphs(filename).unwrap();
        assert_eq!(paragraphs.len(), 1);
    }

    #[test]
    fn test_read_file_into_paragraphs_multiple_paragraphs() {
        let filename = "test_data/3_paragraphs.txt";
        let paragraphs = read_file_into_paragraphs(filename).unwrap();
        assert_eq!(paragraphs.len(), 3);
    }

    #[test]
    fn test_read_file_into_paragraphs_non_existent_file() {
        let filename = "test_data/non_existent_file.txt";
        let result = read_file_into_paragraphs(filename);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_correct_number_of_queries() {
        let num_queries = 10;
        let max_tokens = 5;
        let queries = generate_queries_from_fixed_dictionary(num_queries, max_tokens);

        assert_eq!(queries.len(), num_queries);
    }

    #[test]
    fn test_query_length_within_range() {
        let num_queries = 10;
        let max_tokens = 3;
        let queries = generate_queries_from_fixed_dictionary(num_queries, max_tokens);

        for query in queries {
            let token_count = query.split_whitespace().count();
            assert!(token_count > 0 && token_count <= max_tokens);
        }
    }

    #[test]
    fn test_queries_not_empty() {
        let num_queries = 10;
        let max_tokens = 3;
        let queries = generate_queries_from_fixed_dictionary(num_queries, max_tokens);

        for query in queries {
            assert!(!query.is_empty());
        }
    }
}