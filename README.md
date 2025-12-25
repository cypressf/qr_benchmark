# QR Code Benchmark

A benchmarking suite for Rust QR code decoding libraries.

## Supported Libraries

- **rqrr**: Pure Rust port of Quirc.
- **rxing**: Rust port of ZXing.
- **bardecoder**: Pure Rust QR decoder.
- **zbar**: Bindings to the ZBar C library.

## Prerequisites

### ZBar

To use the `zbar` feature (enabled by default), you need to have the ZBar C library installed on your system.

**macOS:**
```bash
brew install zbar
```

**Ubuntu/Debian:**
```bash
sudo apt-get install libzbar-dev
```

**Fedora:**
```bash
sudo dnf install zbar-devel
```

## Usage

Run the benchmark with default settings:

```bash
cargo run --bin qr_benchmark
```

### Options

- `-l, --libs <LIBS>`: Specific libraries to benchmark (e.g., `-l rqrr -l rxing`).
- `-n, --iterations <ITERATIONS>`: Number of iterations per image (default: 5).
- `-c, --categories <CATEGORIES>`: Specific categories to benchmark (e.g., `-c blurred`).
- `-o, --output <OUTPUT>`: Path to the output CSV file (default: `raw_measurements.csv`).

### Example

```bash
cargo run --bin qr_benchmark -- --categories blurred --libs rqrr --iterations 10
```

## Extending

You can use `qr_benchmark` as a library to benchmark your own decoder. Implement the `QrDecoder` trait and run the benchmark suite.

See `examples/custom_decoder.rs` for a complete example.

## Analysis

To generate performance visualizations from the benchmark results:

```bash
cargo run --bin analyze
```
