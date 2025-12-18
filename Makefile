# Spacejam Makefile
# 
# This Makefile is used to build the Spacejam binary for all platforms.
# It uses the cargo build command to build the binary for each platform.
# It then bundles the binaries into a single tarball.
# 
# Usage:
# make bundle
# make macos-arm64
# make macos-amd64
# make linux-arm64
# make linux-amd64
export AWS_LC_SYS_C_STD = 11
export AWS_LC_SYS_CMAKE_BUILDER = 1

# build all targets
bundle: macos-arm64 macos-amd64 linux-amd64 linux-arm64 tar-all

# make tarball for all
tar-all: tar-spacejam tar-spacevm

# make tarball for spacejam
tar-spacejam:
	mkdir -p target/bundle
	tar -czf target/bundle/spacejam-0.7.2-macos-arm64.tar.gz -C target/aarch64-apple-darwin/prod spacejam
	tar -czf target/bundle/spacejam-0.7.2-macos-amd64.tar.gz -C target/x86_64-apple-darwin/prod spacejam
	tar -czf target/bundle/spacejam-0.7.2-linux-amd64.tar.gz -C target/x86_64-unknown-linux-gnu/prod spacejam
	tar -czf target/bundle/spacejam-0.7.2-linux-arm64.tar.gz -C target/aarch64-unknown-linux-gnu/prod spacejam

# make tarball for spacevm
tar-spacevm:
	tar -czf target/bundle/spacevm-0.7.2-macos-arm64.tar.gz -C target/aarch64-apple-darwin/prod libspacevm.dylib
	tar -czf target/bundle/spacevm-0.7.2-macos-amd64.tar.gz -C target/x86_64-apple-darwin/prod libspacevm.dylib
	tar -czf target/bundle/spacevm-0.7.2-linux-amd64.tar.gz -C target/x86_64-unknown-linux-gnu/prod libspacevm.so
	tar -czf target/bundle/spacevm-0.7.2-linux-arm64.tar.gz -C target/aarch64-unknown-linux-gnu/prod libspacevm.so

# build macos-arm64
macos-arm64:
	cargo b --profile prod --target aarch64-apple-darwin

# build macos-amd64
macos-amd64:
	cargo b --profile prod --target x86_64-apple-darwin

# build linux-arm64
linux-arm64:
	cargo b --profile prod --target aarch64-unknown-linux-gnu

# build linux-amd64
linux-amd64:
	cargo b --profile prod --target x86_64-unknown-linux-gnu
