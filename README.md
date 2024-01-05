# Positional Inverted Index

A positional inverted index is the core data structure underyling many (most? all?) full-text search engines. The best explanation of them I'm aware of is in Stanford's Information Retrieval textbook: https://nlp.stanford.edu/IR-book/html/htmledition/positional-indexes-1.html

# Basics

This repo implements a positional inverted index and a basic CLI to interact with it.

Example:

```sh
$ mkdir indices
$ INDEX_NAME=indices/testindex  # the file you want to save the index in. Indices and files are 1-1
$ cargo run index $INDEX_NAME 1 "here is some content"
$ cargo run index $INDEX_NAME 2 "here is some more content"
$ cargo run index $INDEX_NAME 3 "here is even more content"
$ cargo run search $INDEX_NAME "is some"
Search results: [1, 2]
$ cargo run search $INDEX_NAME "here"
Search results: [1, 2, 3]
$ cargo run search $INDEX_NAME "more content"
Search results: [2, 3]
$ cargo run term_list_size $INDEX_NAME
Approximate term list size in bytes: 216
$ cargo run posting_list_sizes $INDEX_NAME
Approximate posting list sizes in bytes: [16, 32, 32, 48, 48, 48]
```

# Benchmarking

```sh
$ cargo run -- benchmark 50 3 3 fixed results/frankenstein "benchmarking_data/frankenstein.txt"
Benchmark completed successfully
$ cargo run -- benchmark 50 3 3 fixed results/many_books_small_queries $(find benchmarking_data | grep "/")
Benchmark completed successfully
```

# Plotting

```sh
$ cargo run -- plot_indexing_duration results/frankenstein
Plot completed successfully
$ cargo run -- plot_query_duration results/frankenstein
Plot completed successfully
```
