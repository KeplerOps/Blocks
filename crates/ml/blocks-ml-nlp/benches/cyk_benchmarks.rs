use criterion::{black_box, criterion_group, criterion_main, Criterion};
use blocks_ml_nlp::algorithms::CYKParser;

fn create_benchmark_grammar() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("S", vec!["NP VP"]),
        ("NP", vec!["Det N", "NP PP"]),
        ("VP", vec!["V NP", "VP PP"]),
        ("PP", vec!["P NP"]),
        ("Det", vec!["the", "a"]),
        ("N", vec!["cat", "dog", "mouse", "elephant", "bird"]),
        ("V", vec!["chased", "saw", "liked", "followed", "watched"]),
        ("P", vec!["with", "in", "on", "under", "by"]),
    ]
}

fn benchmark_simple_sentence(c: &mut Criterion) {
    let parser = CYKParser::new(create_benchmark_grammar()).unwrap();
    let sentence = "the cat saw a mouse";
    
    c.bench_function("cyk_simple_sentence", |b| {
        b.iter(|| {
            parser.parse(black_box(sentence)).unwrap();
        })
    });
}

fn benchmark_complex_sentence(c: &mut Criterion) {
    let parser = CYKParser::new(create_benchmark_grammar()).unwrap();
    let sentence = "the cat with a mouse under the bird saw the dog by the elephant";
    
    c.bench_function("cyk_complex_sentence", |b| {
        b.iter(|| {
            parser.parse(black_box(sentence)).unwrap();
        })
    });
}

fn benchmark_invalid_sentence(c: &mut Criterion) {
    let parser = CYKParser::new(create_benchmark_grammar()).unwrap();
    let sentence = "cat the mouse saw"; // Invalid sentence structure
    
    c.bench_function("cyk_invalid_sentence", |b| {
        b.iter(|| {
            parser.parse(black_box(sentence)).unwrap();
        })
    });
}

criterion_group!(
    benches,
    benchmark_simple_sentence,
    benchmark_complex_sentence,
    benchmark_invalid_sentence
);
criterion_main!(benches);