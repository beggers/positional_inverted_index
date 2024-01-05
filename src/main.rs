mod benchmark;
mod idx;
mod plot;

use benchmark::benchmark_index;
use idx::PositionalInvertedIndex;
use plot::{
    plot_indexing_duration,
    plot_query_duration
};
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
        .subcommand(SubCommand::with_name("benchmark")
            .about("Runs a benchmarking suite")
            .arg(Arg::with_name("Query Frequency")
                .help("Frequency of queries during benchmarking")
                .required(true))
            .arg(Arg::with_name("Num Queries")
                .help("Number of queries to run")
                .required(true))
            .arg(Arg::with_name("Max Query Tokens")
                .help("Maximum number of tokens per query")
                .required(true))
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to store benchmark results")
                .required(true))
            .arg(Arg::with_name("Filenames")
                .help("The filenames to index")
                .required(true)
                .multiple(true)))
        .subcommand(SubCommand::with_name("plot_indexing_duration")
            .about("Plots indexing duration results")
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to read benchmark results and write the plot")
                .required(true)))
        .subcommand(SubCommand::with_name("plot_query_duration")
            .about("Plots query duration results")
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to read benchmark results and write the plot")
                .required(true)))
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
        ("benchmark", Some(sub_m)) => {
            let filenames: Vec<String> = sub_m.values_of("Filenames").unwrap().map(|s| s.to_string()).collect();
            let query_frequency = sub_m.value_of("Query Frequency").unwrap().parse::<usize>().expect("Invalid Query Frequency");
            let num_queries = sub_m.value_of("Num Queries").unwrap().parse::<usize>().expect("Invalid Num Queries");
            let max_query_tokens = sub_m.value_of("Max Query Tokens").unwrap().parse::<usize>().expect("Invalid Max Query Tokens");
            let target_directory = sub_m.value_of("Target Directory").unwrap();

            match benchmark_index(filenames, query_frequency, num_queries, max_query_tokens, target_directory) {
                Ok(_) => println!("Benchmark completed successfully"),
                Err(e) => println!("Benchmark failed: {}", e),
            }
        },
        ("plot_indexing_duration", Some(sub_m)) => {
            let target_directory = sub_m.value_of("Target Directory").unwrap();

            match plot_indexing_duration(target_directory) {
                Ok(_) => println!("Plot completed successfully"),
                Err(e) => println!("Plot failed: {}", e),
            }
        },
        ("plot_query_duration", Some(sub_m)) => {
            let target_directory = sub_m.value_of("Target Directory").unwrap();

            match plot_query_duration(target_directory) {
                Ok(_) => println!("Plot completed successfully"),
                Err(e) => println!("Plot failed: {}", e),
            }
        },
        _ => panic!("You must specify a subcommand: either 'index' or 'search'"),
    }
}