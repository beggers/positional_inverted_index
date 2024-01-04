mod benchmark;
mod idx;

use idx::PositionalInvertedIndex;
use clap::{
    App,
    Arg,
    SubCommand
};
use std::{
    fs,
    path::Path
};

fn main() {
    let matches = App::new("Positional Inverted Index CLI")
        .version("0.1")
        .author("beggers")
        .about("Manages a positional inverted index")
        .arg(Arg::with_name("INDEX")
            .help("Sets the path to the index file")
            .required(true)
            .index(1))
        .subcommand(SubCommand::with_name("index")
            .about("Indexes a document")
            .arg(Arg::with_name("DOC_ID")
                .help("The ID of the document to index")
                .required(true))
            .arg(Arg::with_name("CONTENT")
                .help("The content of the document to index")
                .required(true)))
        .subcommand(SubCommand::with_name("search")
            .about("Searches the index")
            .arg(Arg::with_name("QUERY")
                .help("The query string to search for")
                .required(true)))
        .subcommand(SubCommand::with_name("term_list_size")
            .about("Prints the approximate size of the term list in bytes"))
        .subcommand(SubCommand::with_name("posting_list_sizes")
            .about("Prints the approximate size of each posting list in bytes"))
        .get_matches();

    let index_path = matches.value_of("INDEX").unwrap();

    let mut index = if Path::new(index_path).exists() {
        let data = fs::read_to_string(index_path).expect("Unable to read file");
        serde_json::from_str(&data).expect("Unable to parse file")
    } else {
        PositionalInvertedIndex::new()
    };

    match matches.subcommand() {
        ("index", Some(sub_m)) => {
            let doc_id = sub_m.value_of("DOC_ID").unwrap().parse::<usize>().expect("Invalid document ID");
            let content = sub_m.value_of("CONTENT").unwrap();
            index.index_document(doc_id, content);

            let serialized = serde_json::to_string(&index).expect("Unable to serialize index");
            fs::write(index_path, serialized).expect("Unable to write file");
        },
        ("search", Some(sub_m)) => {
            let query = sub_m.value_of("QUERY").unwrap();
            let results = index.search(query);
            println!("Search results: {:?}", results);
        },
        ("term_list_size", Some(_)) => {
            println!("Approximate term list size in bytes: {}", index.approximate_term_list_size_in_bytes());
        },
        ("posting_list_sizes", Some(_)) => {
            println!("Approximate posting list sizes in bytes: {:?}", index.approximate_posting_list_sizes_in_bytes());
        },
        _ => panic!("You must specify a subcommand: either 'index' or 'search'"),
    }
}