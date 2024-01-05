# Positional Inverted Index

A positional inverted index is the core data structure underyling many (most? all?) full-text search engines. The best explanation of them I'm aware of is in Stanford's Information Retrieval textbook: https://nlp.stanford.edu/IR-book/html/htmledition/positional-indexes-1.html

# Results

I implemented a basic positional inverted index and benchmarked in a few different ways. The results convinced me that we don't need to do any special optimization on the core data structures and algorithms beyond finding an efficient way to get and update posting lists in S3.

Query latency is acceptably low (< 80 ms mean, <200 ms max) even for a relatively large corpus with pathologically chosen inputs. See results for more data.

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

# Manual Benchmarking

```sh
$ cargo run -- benchmark 50 3 3 uniform frequency results/frankenstein "benchmarking_data/frankenstein.txt"
Benchmark completed successfully
$ cargo run -- benchmark 50 3 3 uniform frequency results/many_books_small_queries $(find benchmarking_data | grep "/")
Benchmark completed successfully
```

# Manual Plotting

```sh
$ cargo run -- plot_indexing_duration results/frankenstein
Plot completed successfully
$ cargo run -- plot_query_duration results/frankenstein
Plot completed successfully
```

# Notes

## Dependent variables

### Documents

[x] Number of documents
[x] Similarity of documents
[x] Query length
[x] Query relevancy (uniformly random characters <-> strings from documents)

### Algorithm

[x] Token-ordered search vs reverse frequency-ordered search
[ ] Pre-filter step on presence of every token
[ ] Stop word filtering

## Independent variables

[x] Term list size
[x] Posting list sizes per documents indexed
[x] Posting list sizes distribution at end
[x] Indexing latency
[x] Query latency

## Graphs

[x] line - Indexing latency per documents indexed
[ ] line with std dev - That ^
[x] line - Query latency per documents indexed
[ ] line with std dev - That ^
[ ] bar - Most expensive queries (query string, documents, latency)
[ ] bar - Final posting list sizes
[x] line with std dev - Posting list size distribution per documents indexed
[x] line - Term list size distribution per documents indexed