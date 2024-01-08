use std::error::Error;
use std::path::Path;
use csv::ReaderBuilder;

pub fn print_top_n_final_posting_lists(target_dir: &str, n: usize) -> Result<(), Box<dyn Error>> {
    let top_n = top_n_final_posting_lists(target_dir, n)?;
    println!("Top {} posting lists:", n);
    for (term, size) in top_n {
        let readable_size = bytes_to_human_readable(size);
        println!("{}: {}", term, readable_size);
    }
    Ok(())
}

fn top_n_final_posting_lists(target_dir: &str, n: usize) -> Result<Vec<(String, usize)>, Box<dyn Error>> {
    let input_path = Path::new(target_dir).join("final_posting_list_sizes.csv");
    let mut rdr = ReaderBuilder::new().from_path(input_path)?;
    let mut records = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let term = record.get(0).unwrap().to_string();
        let size: u32 = record.get(1).unwrap().parse()?;

        records.push((term, size));
    }

    records.sort_by(|a, b| b.1.cmp(&a.1));

    let mut results = Vec::new();
    for (term, size) in records.iter().take(n) {
        results.push((term.clone(), *size as usize));
    }

    Ok(results)
}

fn bytes_to_human_readable(bytes: usize) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut i = 0;

    while size >= 1024.0 && i < UNITS.len() - 1 {
        size /= 1024.0;
        i += 1;
    }

    format!("{:.2} {}", size, UNITS[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_n_sorted() -> Result<(), Box<dyn Error>> {
        let target_dir = "test_data/results1/";
        let output = top_n_final_posting_lists(target_dir, 2)?;

        let expected_output = vec![("term3".to_string(), 300), ("term2".to_string(), 200)];
        assert_eq!(output, expected_output);
        Ok(())
    }

    #[test]
    fn test_top_n_from_unsorted_data() -> Result<(), Box<dyn Error>> {
        let target_dir = "test_data/results2/";
        let output = top_n_final_posting_lists(target_dir, 1)?;

        let expected_output = vec![("cherry".to_string(), 250)];
        assert_eq!(output, expected_output);
        Ok(())
    }

    #[test]
    fn test_bytes_to_human_readable() {
        assert_eq!(bytes_to_human_readable(500), "500.00 B");

        assert_eq!(bytes_to_human_readable(2048), "2.00 KB");

        assert_eq!(bytes_to_human_readable(3_145_728), "3.00 MB");

        assert_eq!(bytes_to_human_readable(3_221_225_472), "3.00 GB");

        assert_eq!(bytes_to_human_readable(1_099_511_627_776), "1.00 TB");
    }
}