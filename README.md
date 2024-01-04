# Positional Inverted Index

A positional inverted index is the core data structure underyling many (most? all?) full-text search engines. The best explanation of them I'm aware of is in Stanford's Information Retrieval textbook: https://nlp.stanford.edu/IR-book/html/htmledition/positional-indexes-1.html

# Basics

This repo implements a positional inverted index and a basic CLI to interact with it.

Example:

```sh
$ mkdir indices
$ INDEX_NAME=indices/testindex  # the file you want to save the index in. Indices and files are 1-1
$ cargo run $INDEX_NAME index 1 "here is some content"
$ cargo run $INDEX_NAME index 2 "here is some more content"
$ cargo run $INDEX_NAME index 3 "here is even more content"
$ cargo run $INDEX_NAME search "is some"
Search results: [1, 2]
$ cargo run $INDEX_NAME search "here"
Search results: [1, 2, 3]
$ cargo run $INDEX_NAME search "more content"
Search results: [2, 3]
$ cargo run $INDEX_NAME term_list_size
Approximate term list size in bytes: 216
$ cargo run $INDEX_NAME posting_list_sizes
Approximate posting list sizes in bytes: [16, 32, 32, 48, 48, 48]
```
