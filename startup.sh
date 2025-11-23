#!/bin/bash
set -e


# Install Rust
echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# Load Rust environment for this script
if [ -f "$HOME/.cargo/env" ]; then
	. "$HOME/.cargo/env"
else
	export PATH="$HOME/.cargo/bin:$PATH"
fi


# Install Wasmtime
echo "Installing Wasmtime..."
curl https://wasmtime.dev/install.sh -sSf | bash || true
export WASMTIME_HOME="$HOME/.wasmtime"
export PATH="$WASMTIME_HOME/bin:$PATH"

# Install RocksDB dependencies
echo "Installing RocksDB dependencies..."
sudo apt-get update
sudo apt-get install -y build-essential librocksdb-dev pkg-config

# Clone your repo
echo "Cloning your blockchain repo..."
git clone https://github.com/actionsproof/Blockchain-.git poa-chain || true
cd poa-chain

# Build the node
echo "Building the node..."
cargo build --release

# Run the node (example, adjust as needed)
echo "Running the node..."
cd node
cargo run --release
