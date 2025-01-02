# Changelog
All notable changes to blocks-cs-sort will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Version numbers are managed by the workspace.

## [Unreleased]

## [Initial Release] - 2025-01-01
### Added
- Initial release
- QuickSort implementation
- HeapSort implementation
- Parallel sorting algorithms (behind `parallel` feature)
- SIMD optimizations (behind `simd` feature)
- Comprehensive benchmarking suite
- Support for various input distributions in tests
- Documentation and examples

### Performance
- QuickSort performs within 2-4x of standard library sort
- HeapSort provides stable performance across different input distributions
- Parallel implementations show significant speedup on large datasets 