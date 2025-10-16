#!/bin/bash
# Performance benchmark runner for Recog Rust implementation
# This script runs the comprehensive benchmarks and displays results

echo "🔬 Running Recog Rust Performance Benchmarks"
echo "============================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the recog-rs directory"
    exit 1
fi

echo "📦 Building benchmarks..."
cargo build --release --benches

if [ $? -ne 0 ]; then
    echo "❌ Error: Failed to build benchmarks"
    exit 1
fi

echo ""
echo "⚡ Running fingerprint matching benchmarks..."
echo ""

# Run the fingerprint matching benchmark
cargo bench --bench fingerprint_matching

echo ""
echo "📄 Running XML loading benchmarks..."
echo ""

# Run the XML loading benchmark
cargo bench --bench xml_loading

echo ""
echo "📊 Benchmark Results Summary"
echo "============================"
echo ""
echo "The benchmarks measure:"
echo "• Fingerprint creation and compilation"
echo "• XML parsing performance with different database sizes"
echo "• Pattern matching speed with simple and complex inputs"
echo "• Batch processing capabilities"
echo "• Parameter interpolation performance"
echo ""
echo "Key metrics to observe:"
echo "• Time per operation (lower is better)"
echo "• Memory usage patterns"
echo "• Scaling characteristics with database size"
echo ""
echo "💡 For detailed HTML reports, check target/criterion/ after running benchmarks"
echo ""
echo "✅ Benchmarks completed!"
