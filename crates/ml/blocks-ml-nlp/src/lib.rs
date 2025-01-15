/*!
This crate provides a collection of Natural Language Processing (NLP) algorithms implemented in Rust.

Each algorithm is implemented with a focus on:
- Performance optimizations
- Memory efficiency
- Comprehensive testing
- Clear documentation
- Modern Rust idioms

# Available Algorithms

## Parsing
- [`CYKParser`](mod@algorithms::cyk): Cocke-Younger-Kasami parsing algorithm for context-free grammars
- [`EarleyParser`](mod@algorithms::earley): Earley parsing algorithm for context-free grammars

## Sequence Labeling
- [`ViterbiTagger`](mod@algorithms::viterbi): Part-of-Speech tagging using Viterbi algorithm
- [`ForwardBackward`](mod@algorithms::forward_backward): Forward-Backward algorithm for Hidden Markov Models
- [`EMAlgorithm`](mod@algorithms::em): Expectation-Maximization for Hidden Markov Models

## Word Embeddings
- [`Word2Vec`](mod@algorithms::word2vec): Skip-gram and CBOW implementations
- [`GloVe`](mod@algorithms::glove): Global Vectors for Word Representation

## Text Processing
- [`BPETokenizer`](mod@algorithms::bpe): Byte-Pair Encoding for subword tokenization
- [`BeamSearch`](mod@algorithms::beam_search): Beam Search decoding for sequence generation
- [`NGramModel`](mod@algorithms::ngram): N-gram Language Model implementation

# Usage Example

```rust
use blocks_ml_nlp::algorithms::cyk::CYKParser;

let grammar = vec![
    ("S", vec!["NP VP"]),
    ("NP", vec!["Det N"]),
    ("VP", vec!["V NP"]),
    ("Det", vec!["the"]),
    ("N", vec!["cat"]),
    ("V", vec!["saw"]),
];

let parser = CYKParser::new(grammar);
let sentence = "the cat saw the cat";
let parse_tree = parser.parse(sentence).expect("Parse should succeed");
```
*/

pub mod algorithms;
pub mod error;
mod utils;

pub use error::{Result, NLPError};
