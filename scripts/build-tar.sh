#!/bin/bash
set -e

VERSION="1.0.0"
PACKAGE_NAME="exam-recorder-suite"
ARCHIVE_NAME="${PACKAGE_NAME}-${VERSION}-linux-amd64"
BUILD_DIR="tar-build"

echo "Building tar.gz archive..."

# Clean previous build
rm -rf "$BUILD_DIR"
rm -f "${ARCHIVE_NAME}.tar.gz"

# Build binaries
cargo build --release

# Create directory structure
mkdir -p "$BUILD_DIR/bin"
mkdir -p "$BUILD_DIR/share/doc/$PACKAGE_NAME"

# Copy binaries
cp target/release/exam-recorder "$BUILD_DIR/bin/"
cp target/release/exam-viewer "$BUILD_DIR/bin/"

# Copy documentation
cp README.md "$BUILD_DIR/share/doc/$PACKAGE_NAME/"

# Create install script
cat > "$BUILD_DIR/install.sh" <<'EOF'
#!/bin/bash
set -e

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"

echo "Installing Exam Recorder Suite..."
echo "Install prefix: $INSTALL_PREFIX"

mkdir -p "$INSTALL_PREFIX/bin"
cp bin/exam-recorder "$INSTALL_PREFIX/bin/"
cp bin/exam-viewer "$INSTALL_PREFIX/bin/"

chmod +x "$INSTALL_PREFIX/bin/exam-recorder"
chmod +x "$INSTALL_PREFIX/bin/exam-viewer"

echo "Installation complete!"
echo "Binaries installed to: $INSTALL_PREFIX/bin"
EOF
chmod +x "$BUILD_DIR/install.sh"

# Create archive
tar -czf "${ARCHIVE_NAME}.tar.gz" -C "$BUILD_DIR" .

echo "Archive created: ${ARCHIVE_NAME}.tar.gz"

