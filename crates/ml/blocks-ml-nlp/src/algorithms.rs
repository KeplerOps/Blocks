/*!
Collection of NLP algorithms implementations.

Each algorithm is implemented in its own module with comprehensive documentation,
tests, and benchmarks where appropriate.
*/

pub mod cyk;
pub mod earley;
pub mod viterbi;
pub mod forward_backward;
pub mod em;
pub mod word2vec;
pub mod glove;
pub mod bpe;
pub mod beam_search;
pub mod ngram;

// Re-exports for convenience
pub use self::cyk::CYKParser;
pub use self::earley::EarleyParser;
pub use self::viterbi::ViterbiTagger;
pub use self::forward_backward::ForwardBackward;
pub use self::em::EMAlgorithm;
pub use self::word2vec::{Word2Vec, SkipGram, CBOW};
pub use self::glove::GloVe;
pub use self::bpe::BPETokenizer;
pub use self::beam_search::BeamSearch;
pub use self::ngram::NGramModel;