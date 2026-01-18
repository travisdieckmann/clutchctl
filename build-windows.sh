#!/bin/bash

# Build script for Windows cross-compilation

echo "Building clutchctl for Windows..."

# Check if running in Docker/CI or locally
if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "MinGW found, building locally..."
    cargo build --release --target x86_64-pc-windows-gnu

    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ Build successful!"
        echo "Windows executable: target/x86_64-pc-windows-gnu/release/clutchctl.exe"
        ls -lh target/x86_64-pc-windows-gnu/release/clutchctl.exe

        # Create Windows distribution package
        echo ""
        echo "Creating Windows distribution package..."
        mkdir -p dist/clutchctl-windows
        cp target/x86_64-pc-windows-gnu/release/clutchctl.exe dist/clutchctl-windows/
        cp windows/install-driver.bat dist/clutchctl-windows/ 2>/dev/null || true
        cp windows/clutchctl-winusb.inf dist/clutchctl-windows/ 2>/dev/null || true
        cp README.md dist/clutchctl-windows/README.txt 2>/dev/null || true

        echo "Distribution package created in: dist/clutchctl-windows/"
        ls -la dist/clutchctl-windows/
    fi
elif command -v docker &> /dev/null; then
    echo "MinGW not found, using Docker..."
    docker build -f Dockerfile.windows -t clutchctl-windows-builder .

    if [ $? -eq 0 ]; then
        # Extract the built executable to the target directory
        mkdir -p target/x86_64-pc-windows-gnu/release
        docker create --name temp-clutchctl clutchctl-windows-builder
        docker cp temp-clutchctl:/app/target/x86_64-pc-windows-gnu/release/clutchctl.exe ./target/x86_64-pc-windows-gnu/release/clutchctl.exe
        docker rm temp-clutchctl

        echo ""
        echo "✅ Build successful!"
        echo "Windows executable: target/x86_64-pc-windows-gnu/release/clutchctl.exe"
        ls -lh target/x86_64-pc-windows-gnu/release/clutchctl.exe
    fi
else
    echo "❌ Error: Neither MinGW nor Docker found."
    echo ""
    echo "Please install MinGW:"
    echo "  sudo apt-get install gcc-mingw-w64-x86-64"
    echo ""
    echo "Or install Docker:"
    echo "  https://docs.docker.com/engine/install/ubuntu/"
    exit 1
fi