use crate::PositionalInvertedIndex;

use csv::Writer;
use rand::{seq::SliceRandom, thread_rng, Rng};
use regex::Regex;
use std::{
    error::Error,
    f64,
    fs::self,
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

    let indexing_csv_path = Path::new(target_directory).join("indexing_data.csv");
    let querying_csv_path = Path::new(target_directory).join("querying_data.csv");
    let size_csv_path = Path::new(target_directory).join("size_data.csv");

    let mut indexing_writer = Writer::from_path(indexing_csv_path)?;
    let mut querying_writer = Writer::from_path(querying_csv_path)?;
    let mut size_writer = Writer::from_path(size_csv_path)?;

    indexing_writer.write_record(&["Document Count", "Indexing Duration"])?;
    querying_writer.write_record(&["Document Count", "Toknes in Query", "Query Duration"])?;
    size_writer.write_record(&["Paragraph", "Mean Posting List Size", "Std Dev Posting List Size"])?;

    let mut paragraph_counter = 0;
    for filename in filenames {
        let paragraphs = read_file_into_paragraphs(&filename)?;

        for paragraph in paragraphs {
            if paragraph.is_empty() {
                continue;
            }

            let start = Instant::now();
            index.index_document(paragraph_counter, &paragraph);
            let indexing_duration = start.elapsed();
            indexing_writer.write_record(&[&paragraph_counter.to_string(), &format!("{:?}", indexing_duration)])?;

            if paragraph_counter % query_frequency == 0 {
                let queries = generate_queries_from_fixed_dictionary(num_queries, max_query_tokens);
                for query in queries {
                    let query_start = Instant::now();
                    index.search(&query);
                    let query_duration = query_start.elapsed();

                    let tokens_in_query = query.split_whitespace().count();
                    querying_writer.write_record(&[&paragraph_counter.to_string(), &tokens_in_query.to_string(), &format!("{:?}", query_duration)])?;
                }

                let posting_list_sizes = index.approximate_posting_list_sizes_in_bytes();
                let (mean, std_dev) = compute_mean_and_std_dev(&posting_list_sizes);

                size_writer.write_record(&[&paragraph_counter.to_string(), &mean.to_string(), &std_dev.to_string()])?;
            }

            paragraph_counter += 1;
        }
    }

    indexing_writer.flush()?;
    querying_writer.flush()?;
    size_writer.flush()?;

    Ok(())
}

fn compute_mean_and_std_dev(sizes: &[usize]) -> (f64, f64) {
    if sizes.is_empty() {
        return (0.0, 0.0);
    }

    let sum: usize = sizes.iter().sum();
    let mean = sum as f64 / sizes.len() as f64;

    let variance: f64 = sizes.iter()
        .map(|&size| {
            let diff = size as f64 - mean;
            diff * diff
        })
        .sum::<f64>() / sizes.len() as f64;

    let std_dev = variance.sqrt();
    (mean, std_dev)
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

    #[test]
    fn test_mean_and_std_dev_typical() {
        let sizes = vec![1, 2, 3, 4, 5];
        let (mean, std_dev) = compute_mean_and_std_dev(&sizes);
        assert_eq!(mean, 3.0);
        assert!((std_dev - 1.41421356237).abs() < 1e-10);
    }

    #[test]
    fn test_mean_and_std_dev_empty() {
        let sizes: Vec<usize> = vec![];
        let (mean, std_dev) = compute_mean_and_std_dev(&sizes);
        assert_eq!(mean, 0.0);
        assert_eq!(std_dev, 0.0);
    }

    #[test]
    fn test_mean_and_std_dev_large_numbers() {
        let sizes = vec![1_000_000, 2_000_000, 3_000_000];
        let (mean, std_dev) = compute_mean_and_std_dev(&sizes);
        assert_eq!(mean, 2_000_000.0);
        assert!((std_dev - 816496.580927726).abs() < 1e-6);
    }

    #[test]
    fn test_mean_and_std_dev_single_element() {
        let sizes = vec![42];
        let (mean, std_dev) = compute_mean_and_std_dev(&sizes);
        assert_eq!(mean, 42.0);
        assert_eq!(std_dev, 0.0);
    }
}