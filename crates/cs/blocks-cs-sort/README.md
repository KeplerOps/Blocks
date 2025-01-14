# blocks-cs-sort

A collection of high-performance sorting algorithms implemented in Rust.

## Features

- Multiple sorting algorithms (QuickSort, HeapSort)
- SIMD optimizations (behind `simd` feature flag)
- Parallel sorting implementations (behind `parallel` feature flag)
- Comprehensive benchmarks against standard library
- Thoroughly tested with various input distributions

## Usage

Basic usage:

```rust
use blocks_cs_sort::algorithms::{quicksort, heapsort};
// Using quicksort
let mut numbers = vec![3, 1, 4, 1, 5, 9];
quicksort::sort(&mut numbers);
// Using heapsort
let mut numbers = vec![3, 1, 4, 1, 5, 9];
heapsort::sort(&mut numbers).expect("Sort failed");
```

## Optional Features

- `parallel`: Enable parallel sorting algorithms (uses rayon)
- `simd`: Enable SIMD optimizations for numeric types

## Performance

Run the benchmarks with:

```bash
bash
cargo bench
```

View detailed HTML reports in Blocks `target/criterion/report/index.html`

### Benchmark Categories

- Input sizes: 1K, 10K, 100K, 1M elements
- Distributions: sorted, reverse sorted, random, nearly sorted, few unique values
- Comparisons against Rust's standard library sort

## Contributing

See the workspace-level contributing guidelines.

## License

The MIT License and the Apache License.
