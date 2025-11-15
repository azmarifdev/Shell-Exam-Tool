#!/bin/bash
set -e

VERSION="1.0.0"
PACKAGE_NAME="exam-recorder-suite"
BUILD_DIR="rpm-build"
SPEC_DIR="$BUILD_DIR/SPECS"
SOURCES_DIR="$BUILD_DIR/SOURCES"
BUILDROOT_DIR="$BUILD_DIR/BUILDROOT"

echo "Building RPM package..."

# Clean previous build
rm -rf "$BUILD_DIR"

# Build binaries
cargo build --release

# Create directory structure
mkdir -p "$SPEC_DIR"
mkdir -p "$SOURCES_DIR"
mkdir -p "$BUILDROOT_DIR"

# Copy binaries to sources
cp target/release/exam-recorder "$SOURCES_DIR/"
cp target/release/exam-viewer "$SOURCES_DIR/"

# Create spec file
cat > "$SPEC_DIR/exam-recorder-suite.spec" <<EOF
Name:           $PACKAGE_NAME
Version:        $VERSION
Release:        1%{?dist}
Summary:        Exam Recorder Suite - Secure terminal session recorder
License:        MIT
URL:            https://azmarif.dev
Source0:        exam-recorder
Source1:        exam-viewer

%description
Exam Recorder Suite consists of two tools:
- exam-recorder: Student-side secure terminal session recorder
- exam-viewer: Instructor-side decrypter, analyzer, and log viewer

%prep

%build

%install
mkdir -p %{buildroot}/usr/local/bin
install -m 755 %{SOURCE0} %{buildroot}/usr/local/bin/exam-recorder
install -m 755 %{SOURCE1} %{buildroot}/usr/local/bin/exam-viewer

%files
/usr/local/bin/exam-recorder
/usr/local/bin/exam-viewer

%changelog
* $(date +"%a %b %d %Y") A. Z. M. Arif <https://azmarif.dev> - $VERSION-1
- Initial release
EOF

# Build RPM (requires rpmbuild)
if command -v rpmbuild &> /dev/null; then
    rpmbuild --define "_topdir $(pwd)/$BUILD_DIR" -bb "$SPEC_DIR/exam-recorder-suite.spec"
    echo "RPM package created in $BUILD_DIR/RPMS/"
else
    echo "Warning: rpmbuild not found. Install rpm-build package to build RPM."
fi

