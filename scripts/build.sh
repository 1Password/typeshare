#!/bin/bash


set -e

function help() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --version <version>    Version of the release"
    echo "  --target <platform>    Target platform"
    echo "                           Platforms: "
    echo "                             * \"aarch64-apple-darwin\""
    echo "                             * \"x86_64-apple-darwin\""
    echo "                             * \"aarch64-unknown-linux-gnu\""
    echo "                             * \"x86_64-unknown-linux-gnu\""
    echo "                             * \"x86_64-pc-windows-msvc\""
    echo "  --dry-run              Run the script without releasing"
    exit 0
}

# Options:
#   apple-arm64: aarch64-apple-darwin
#   apple-x86: x86_64-apple-darwin
#   linux-arm64: aarch64-unknown-linux-gnu
#   linux-x86: x86_64-unknown-linux-gnu
#   windows-x86: x86_64-pc-windows-msvc
TARGET=""
# --version <version>
VERSION=""
# --dry-run
DRY_RUN=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --target)
            TARGET="$2"
            shift
            shift
            ;;
        --version)
            VERSION="$2"
            shift
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            help
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check if required arguments are provided
if [[ -z "${TARGET}" ]]; then
    echo "Missing required argument: --target"
    exit 1
fi

if [[ -z "$VERSION" ]]; then
    echo "Missing required argument: --version"
    exit 1
fi

# Build the project
pip3 install ziglang
cargo install cargo-zigbuild
rustup target add "${TARGET}"
cargo zigbuild --target "${TARGET}" --release

# Set binary name to typeshare
BINARY_NAME="typeshare"
TARGET_DIR="target/${TARGET}/release"

# Create zip directory
mkdir -p "dist"

OUTPUT_DIR="$(pwd)/dist"

# Create zip file with binary
ZIP_NAME="${BINARY_NAME}-${VERSION}-${TARGET}.zip"
pushd ${TARGET_DIR} && zip "${OUTPUT_DIR}/${ZIP_NAME}" "${BINARY_NAME}${BINARY_SUFFIX}" && popd

# Create manifest file similar to cargo-dist
echo "{\"artifacts\": [{\"path\": \"dist/${ZIP_NAME}\"}]}" > dist-manifest.json

echo "Build complete, contents of dist-manifest.json:"
cat dist-manifest.json

if [[ "$DRY_RUN" == true ]]; then
    echo "ℹ️ Dry run, skipping release upload"
else
    # Upload to release
    cat dist-manifest.json | jq --raw-output ".artifacts[]?.path | select( . != null )" > uploads.txt
    echo "uploading..."
    cat uploads.txt
    gh release upload ${VERSION} $(cat uploads.txt)
    echo "uploaded!"
fi
