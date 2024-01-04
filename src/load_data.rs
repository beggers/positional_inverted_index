mod idx;

use idx::PositionalInvertedIndex;
use std::fs;
use std::error::Error;
use regex::Regex;

pub fn read_and_index_big_file(&mut index: PositionalInvertedIndex, file_path: &str) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string(file_path)?;
    let re = Regex::new(r"\n\n+")?;
    for cap in re.captures_iter(&data) {
        let doc_id = cap[1].parse::<usize>()?;
        let content = &cap[2];
        index.index_document(doc_id, content);
    }
    Ok(())
}