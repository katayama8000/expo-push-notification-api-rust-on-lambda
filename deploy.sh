#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Navigate to the project root directory (assuming this script is in the root)
cd "$(dirname "$0")"

# Clean previous build artifacts
cargo clean

# Set environment variables for cross-compilation of openssl-sys
export PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig"
export AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR="/usr/lib/aarch64-linux-gnu"
export AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR="/usr/include/aarch64-linux-gnu"
export CFLAGS="-I/usr/include/aarch64-linux-gnu -I/usr/include"

# Run the cargo lambda build command
cargo lambda build --release

echo "Build completed successfully!"

# Source AWS credentials
source ./setup_deploy_env.sh

echo "AWS credentials loaded."

# Deploy the lambda function
cargo lambda deploy

echo "Deployment completed successfully!"