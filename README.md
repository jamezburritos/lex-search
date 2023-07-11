# lex-search 
Search file contents using [TF-IDF](https://en.wikipedia.org/wiki/Tf%E2%80%93idf) document search. 

## Supported Formats 
Currently only `.xml` files are searchable. 
The corpus used to develop the tool is from [docs.gl](https://github.com/BSVino/docs.gl), which is made avaliable as a body of xhtml files.

## Building 
To download the source and build the executable:
```sh
$ git clone https://github.com/jamezburritos/lex-search.git
$ cd lex-search
$ cargo build --release
```

## Usage 
To search a body of documents, first index their contents:
```sh
$ ./target/build/search index <directory>
```
Then use that index to search the corpus:
```sh
$ ./target/build/search search "query"
```
