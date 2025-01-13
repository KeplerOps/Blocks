# blocks-cs-search

A collection of high-performance search algorithms implemented in Rust.

## Features

- Binary search algorithm
- Comprehensive benchmarks
- Thoroughly tested with various input distributions

## Usage

Basic usage:

```rust
use blocks_cs_search::algorithms::binary_search;
// Using binary search
let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
assert_eq!(binary_search(&arr, 5), Some(4));
assert_eq!(binary_search(&arr, 11), None);
```

## Performance

Run the benchmarks with:

```bash
cargo bench
```

View detailed HTML reports in Blocks `target/criterion/report/index.html`

### Benchmark Categories

- Input sizes: 1K, 10K, 100K, 1M elements
- Distributions: sorted, reverse sorted, random, nearly sorted, few unique values
- Comparisons against Rust's standard library search

## Contributing

See the workspace-level contributing guidelines.

## License

The MIT License and the Apache License.
