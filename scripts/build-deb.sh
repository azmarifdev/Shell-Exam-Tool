#!/bin/bash
set -e

VERSION="1.0.0"
PACKAGE_NAME="exam-recorder-suite"
BUILD_DIR="deb-build"
BINARY_DIR="$BUILD_DIR/usr/local/bin"
DEBIAN_DIR="$BUILD_DIR/DEBIAN"

echo "Building Debian package..."

# Clean previous build
rm -rf "$BUILD_DIR"

# Build binaries
cargo build --release

# Create directory structure
mkdir -p "$BINARY_DIR"
mkdir -p "$DEBIAN_DIR"

# Copy binaries
cp target/release/exam-recorder "$BINARY_DIR/"
cp target/release/exam-viewer "$BINARY_DIR/"

# Create control file
cat > "$DEBIAN_DIR/control" <<EOF
Package: $PACKAGE_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Maintainer: A. Z. M. Arif <https://azmarif.dev>
Description: Exam Recorder Suite - Secure terminal session recorder for exams
 Exam Recorder Suite consists of two tools:
  - exam-recorder: Student-side secure terminal session recorder
  - exam-viewer: Instructor-side decrypter, analyzer, and log viewer
Homepage: https://azmarif.dev
EOF

# Create postinst script
cat > "$DEBIAN_DIR/postinst" <<'EOF'
#!/bin/bash
set -e
chmod +x /usr/local/bin/exam-recorder
chmod +x /usr/local/bin/exam-viewer
EOF
chmod +x "$DEBIAN_DIR/postinst"

# Build package
dpkg-deb --build "$BUILD_DIR" "${PACKAGE_NAME}_${VERSION}_amd64.deb"

echo "Debian package created: ${PACKAGE_NAME}_${VERSION}_amd64.deb"

