#!/bin/bash
set -e

echo "Building Exam Recorder Suite..."
echo "Author: A. Z. M. Arif | https://azmarif.dev"
echo ""

# Build release binaries
cargo build --release

echo ""
echo "Build complete!"
echo "Binaries are located in:"
echo "  - target/release/exam-recorder"
echo "  - target/release/exam-viewer"

