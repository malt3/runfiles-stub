#!/usr/bin/env bash
set -euo pipefail

# This script is called by the bazel-contrib release workflow
# It should create the release artifacts in dist/ and output release notes to STDOUT

TAG=$1

echo "Building release archive tool..." >&2
cd tools/create-release-archive
cargo build --release >&2
cd ../..

echo "Creating dist/ directory..." >&2
mkdir -p dist

echo "Creating release archive..." >&2
tools/create-release-archive/target/release/create-release-archive \
    "$TAG" \
    "dist/hermetic_launcher-${TAG}.tar.gz"

echo "Archive created at dist/hermetic_launcher-${TAG}.tar.gz" >&2

# Output release notes to STDOUT (this will be used as the GitHub release body)
cat << EOF
## Hermetic Launcher Bazel Rules ${TAG}

This release contains the Bazel rules for hermetic_launcher.

### Usage

Add to your \`MODULE.bazel\`:

\`\`\`starlark
bazel_dep(name = "hermetic_launcher", version = "${TAG}")
\`\`\`
EOF
