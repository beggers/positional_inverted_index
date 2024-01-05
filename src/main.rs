mod benchmark;
mod idx;
mod plot;
mod query_tokens;

use benchmark::benchmark_index;
use idx::{
    PositionalInvertedIndex,
    TokenOrdering
};
use plot::{
    plot_index_latency,
    plot_query_latency
};
use query_tokens::{
    QueryTokenDistribution
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
        .subcommand(SubCommand::with_name("index")
            .about("Indexes a document")
            .arg(Arg::with_name("INDEX")
                .help("Sets the path to the index file")
                .required(true))
            .arg(Arg::with_name("DOC_ID")
                .help("The ID of the document to index")
                .required(true))
            .arg(Arg::with_name("CONTENT")
                .help("The content of the document to index")
                .required(true)))
        .subcommand(SubCommand::with_name("search")
            .about("Searches the index")
            .arg(Arg::with_name("INDEX")
                .help("Sets the path to the index file")
                .required(true))
            .arg(Arg::with_name("QUERY")
                .help("The query string to search for")
                .required(true)))
        .subcommand(SubCommand::with_name("term_list_size")
            .about("Prints the approximate size of the term list in bytes")
            .arg(Arg::with_name("INDEX")
                .help("Sets the path to the index file")
                .required(true)))
        .subcommand(SubCommand::with_name("posting_list_sizes")
            .about("Prints the approximate size of each posting list in bytes")
            .arg(Arg::with_name("INDEX")
                .help("Sets the path to the index file")
                .required(true)))
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
            .arg(Arg::with_name("query_token_distribution")
                .help("The distribution of query tokens")
                .takes_value(true)
                .possible_values(&["fixed", "uniform", "from_document"])
                .required(true))
            .arg(Arg::with_name("Token Search Ordering")
                .help("The ordering of tokens in the query")
                .takes_value(true)
                .possible_values(&["query", "frequency"])
                .required(true))
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to store benchmark results")
                .required(true))
            .arg(Arg::with_name("Filenames")
                .help("The filenames to index")
                .required(true)
                .multiple(true)))
        .subcommand(SubCommand::with_name("plot_index_latency")
            .about("Plots indexing duration results")
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to read benchmark results and write the plot")
                .required(true)))
        .subcommand(SubCommand::with_name("plot_query_latency")
            .about("Plots query duration results")
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to read benchmark results and write the plot")
                .required(true)))
        .subcommand(SubCommand::with_name("plot_posting_list_distribution")
            .about("Plots the distribution of posting list sizes")
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to read benchmark results and write the plot")
                .required(true)))
        .subcommand(SubCommand::with_name("plot_term_list_sizes")
            .about("Plots the term list sizes")
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to read benchmark results and write the plot")
                .required(true)))
        .subcommand(SubCommand::with_name("plot_top_n_queries")
            .about("Plots the top N most expensive queries")
            .arg(Arg::with_name("Target Directory")
                .help("The target directory to read benchmark results and write the plot")
                .required(true))
            .arg(Arg::with_name("N")
                .help("The number of queries to plot")
                .required(true)))
        .get_matches();

    match matches.subcommand() {
        ("index", Some(sub_m)) => {
            let index_path = sub_m.value_of("INDEX").unwrap();
            let mut index = read_or_create_index(index_path);

            let doc_id = sub_m.value_of("DOC_ID").unwrap().parse::<usize>().expect("Invalid document ID");
            let content = sub_m.value_of("CONTENT").unwrap();
            index.index_document(doc_id, content);

            let serialized = serde_json::to_string(&index).expect("Unable to serialize index");
            fs::write(index_path, serialized).expect("Unable to write file");
        },
        ("search", Some(sub_m)) => {
            let index_path = sub_m.value_of("INDEX").unwrap();
            let index = read_or_create_index(index_path);

            let query = sub_m.value_of("QUERY").unwrap();
            let results = index.search(query);
            println!("Search results: {:?}", results);
        },
        ("term_list_size", Some(sub_m)) => {
            let index_path = sub_m.value_of("INDEX").unwrap();
            let index = read_or_create_index(index_path);

            println!("Approximate term list size in bytes: {}", index.approximate_term_list_size_in_bytes());
        },
        ("posting_list_sizes", Some(sub_m)) => {
            let index_path = sub_m.value_of("INDEX").unwrap();
            let index = read_or_create_index(index_path);

            println!("Approximate posting list sizes in bytes: {:?}", index.approximate_posting_list_sizes_in_bytes());
        },
        ("benchmark", Some(sub_m)) => {
            let filenames: Vec<String> = sub_m.values_of("Filenames").unwrap().map(|s| s.to_string()).collect();
            let query_frequency = sub_m.value_of("Query Frequency").unwrap().parse::<usize>().expect("Invalid Query Frequency");
            let num_queries = sub_m.value_of("Num Queries").unwrap().parse::<usize>().expect("Invalid Num Queries");
            let max_query_tokens = sub_m.value_of("Max Query Tokens").unwrap().parse::<usize>().expect("Invalid Max Query Tokens");
            let target_directory = sub_m.value_of("Target Directory").unwrap();
            let query_token_distribution = match sub_m.value_of("query_token_distribution").unwrap() {
                "fixed" => QueryTokenDistribution::Fixed,
                "uniform" => QueryTokenDistribution::Uniform,
                "from_document" => QueryTokenDistribution::FromDocument,
                _ => panic!("Invalid query token distribution"),
            };
            let token_search_ordering = match sub_m.value_of("Token Search Ordering").unwrap() {
                "query" => TokenOrdering::TokenOrder,
                "frequency" => TokenOrdering::AscendingFrequencyOrder,
                _ => panic!("Invalid token search ordering"),
            };
            match benchmark_index(filenames, query_frequency, num_queries, max_query_tokens, query_token_distribution, token_search_ordering, target_directory) {
                Ok(_) => println!("Benchmarking {} completed successfully", target_directory),
                Err(e) => println!("Benchmark failed: {}", e),
            }
        },
        ("plot_index_latency", Some(sub_m)) => {
            let target_directory = sub_m.value_of("Target Directory").unwrap();

            match plot_index_latency(target_directory) {
                Ok(_) => println!("Plot completed successfully"),
                Err(e) => println!("Plot failed: {}", e),
            }
        },
        ("plot_query_latency", Some(sub_m)) => {
            let target_directory = sub_m.value_of("Target Directory").unwrap();

            match plot_query_latency(target_directory) {
                Ok(_) => println!("Plot completed successfully"),
                Err(e) => println!("Plot failed: {}", e),
            }
        },
        ("plot_posting_list_distribution", Some(sub_m)) => {
            let target_directory = sub_m.value_of("Target Directory").unwrap();

            match plot::plot_posting_list_distribution(target_directory) {
                Ok(_) => println!("Plot completed successfully"),
                Err(e) => println!("Plot failed: {}", e),
            }
        },
        ("plot_term_list_sizes", Some(sub_m)) => {
            let target_directory = sub_m.value_of("Target Directory").unwrap();

            match plot::plot_term_list_sizes(target_directory) {
                Ok(_) => println!("Plot completed successfully"),
                Err(e) => println!("Plot failed: {}", e),
            }
        },
        _ => panic!("You must specify a subcommand: either 'index' or 'search'"),
    }
}

fn read_or_create_index(index_path: &str) -> PositionalInvertedIndex {
    let index = if Path::new(index_path).exists() {
        let data = fs::read_to_string(index_path).expect("Unable to read file");
        serde_json::from_str(&data).expect("Unable to parse file")
    } else {
        PositionalInvertedIndex::new()
    };
    index
}