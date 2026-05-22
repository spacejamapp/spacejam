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

DOCKER_IMAGE := ghcr.io/spacejamapp/spacejam
VERSION := $(shell awk '/^\[workspace.package\]/{f=1} f && /^version/{gsub(/"/,"",$$3); print $$3; exit}' Cargo.toml)

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

# build linux-amd64 with full-spec constants
linux-amd64-full:
	cargo b --profile prod --target x86_64-unknown-linux-gnu --no-default-features --features full

# build both tiny and full binaries for docker
linux-amd64-both:
	cargo b --profile prod -p spacejam --target x86_64-unknown-linux-gnu
	cp target/x86_64-unknown-linux-gnu/prod/spacejam target/x86_64-unknown-linux-gnu/prod/spacejam-tiny
	cargo b --profile prod -p spacejam --target x86_64-unknown-linux-gnu --no-default-features --features full
	cp target/x86_64-unknown-linux-gnu/prod/spacejam target/x86_64-unknown-linux-gnu/prod/spacejam-full

# build the docker image, tagging both :latest and :$(VERSION)
docker: linux-amd64-both
	docker build --platform=linux/amd64 \
		-f docker/spacejam.Dockerfile \
		-t $(DOCKER_IMAGE):latest \
		-t $(DOCKER_IMAGE):$(VERSION) \
		.

# push images to ghcr
dpush:
	docker push $(DOCKER_IMAGE):latest
	docker push $(DOCKER_IMAGE):$(VERSION)
