#!/bin/bash
# Package Forge release binaries for distribution

set -e

VERSION=${1:-$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "forge_cli") | .version')}
TARGET=${2:-$(rustc -vV | sed -n 's|host: ||p')}
OUTPUT_DIR="release-packages"

echo "Packaging Forge v${VERSION} for ${TARGET}"

# Build release binaries
echo "Building release binaries..."
cargo build --release --workspace

# Create output directory
mkdir -p "${OUTPUT_DIR}"

# Determine OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "${OS}" in
  linux*)
    PLATFORM="linux"
    CLI_NAME="forge"
    ;;
  darwin*)
    PLATFORM="macos"
    CLI_NAME="forge"
    ;;
  mingw*|msys*|cygwin*)
    PLATFORM="windows"
    CLI_NAME="forge.exe"
    ;;
  *)
    echo "Unsupported OS: ${OS}"
    exit 1
    ;;
esac

# Map architecture
case "${ARCH}" in
  x86_64|amd64)
    ARCH_NAME="x64"
    ;;
  aarch64|arm64)
    ARCH_NAME="arm64"
    ;;
  *)
    echo "Unsupported architecture: ${ARCH}"
    exit 1
    ;;
esac

PACKAGE_NAME="forge-v${VERSION}-${PLATFORM}-${ARCH_NAME}"
PACKAGE_DIR="${OUTPUT_DIR}/${PACKAGE_NAME}"

echo "Creating package: ${PACKAGE_NAME}"

# Create package directory
mkdir -p "${PACKAGE_DIR}/bin"
mkdir -p "${PACKAGE_DIR}/configs"
mkdir -p "${PACKAGE_DIR}/docs"

# Copy binaries
echo "Copying binaries..."
cp "target/release/${CLI_NAME}" "${PACKAGE_DIR}/bin/"

# Copy configuration files
echo "Copying configuration files..."
cp configs/default.yaml "${PACKAGE_DIR}/configs/"
if [ -f "configs/local.yaml.example" ]; then
  cp configs/local.yaml.example "${PACKAGE_DIR}/configs/"
fi

# Copy documentation
echo "Copying documentation..."
cp README.md "${PACKAGE_DIR}/"
cp LICENSE "${PACKAGE_DIR}/" 2>/dev/null || echo "No LICENSE file found"

# Create installation instructions
cat > "${PACKAGE_DIR}/INSTALL.txt" << 'EOF'
Forge - Local LLM Platform
Installation Instructions

1. Extract this archive to your desired installation directory
2. Add the bin/ directory to your PATH:
   - Linux/macOS: export PATH="$PATH:/path/to/forge/bin"
   - Windows: Add to System Environment Variables

3. Run the daemon:
   forge serve

4. Use the CLI:
   forge chat
   forge models list
   forge --help

Configuration:
- Default configuration is in configs/default.yaml
- Create a local.yaml for custom settings

For more information, see README.md
EOF

# Create archive
echo "Creating archive..."
cd "${OUTPUT_DIR}"

if [ "${PLATFORM}" = "windows" ]; then
  # Create ZIP for Windows
  if command -v zip > /dev/null; then
    zip -r "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}"
    echo "Created: ${OUTPUT_DIR}/${PACKAGE_NAME}.zip"
  else
    echo "Warning: zip command not found, skipping archive creation"
  fi
else
  # Create tar.gz for Linux/macOS
  tar -czf "${PACKAGE_NAME}.tar.gz" "${PACKAGE_NAME}"
  echo "Created: ${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz"
fi

cd ..

# Generate checksums
echo "Generating checksums..."
cd "${OUTPUT_DIR}"
if [ "${PLATFORM}" = "windows" ] && [ -f "${PACKAGE_NAME}.zip" ]; then
  sha256sum "${PACKAGE_NAME}.zip" > "${PACKAGE_NAME}.zip.sha256"
elif [ -f "${PACKAGE_NAME}.tar.gz" ]; then
  sha256sum "${PACKAGE_NAME}.tar.gz" > "${PACKAGE_NAME}.tar.gz.sha256"
fi
cd ..

echo ""
echo "Packaging complete!"
echo "Package location: ${OUTPUT_DIR}/"
ls -lh "${OUTPUT_DIR}/"
