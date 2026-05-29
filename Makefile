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
		-f docker/spacejam.dockerfile \
		-t $(DOCKER_IMAGE):latest \
		-t $(DOCKER_IMAGE):$(VERSION) \
		.

# build the fuzz-paired docker images (regular + interpreter) for AOT-vs-int
# A/B comparison on NUMA hosts.
fuzz: docker
	docker build --platform=linux/amd64 \
		--build-arg SPACEJAM_INTERP=1 \
		-f docker/spacejam.dockerfile \
		-t $(DOCKER_IMAGE):int \
		-t $(DOCKER_IMAGE):$(VERSION)-int \
		.

# push images to ghcr
dpush:
	docker push $(DOCKER_IMAGE):latest
	docker push $(DOCKER_IMAGE):$(VERSION)

# push fuzz-paired images (regular + interpreter) to ghcr
fpush: dpush
	docker push $(DOCKER_IMAGE):int
	docker push $(DOCKER_IMAGE):$(VERSION)-int

# Sample-profile `spacejam fuzz tx` on a trace directory via samply.
# Needs sudo (kernel.perf_event_paranoid=3 blocks unprivileged perf_event_open).
# Override:
#   TRACE_DIR=res/foo/trace OUT=/tmp/foo.json.gz SPACEVM=true make profile-trace
TRACE_DIR ?= res/l2a/trace
OUT       ?= /tmp/spacejam-trace.json.gz

.PHONY: profile-trace
profile-trace:
	CARGO_PROFILE_RELEASE_DEBUG=line-tables-only \
		cargo build --release -p spacejam --features trace
	sudo SPACEVM=$(SPACEVM) $$(command -v samply) record --save-only -o $(OUT) \
		./target/release/spacejam fuzz tx $(TRACE_DIR)
	@echo ""
	@echo "profile saved: $(OUT)"
	@echo "view: samply load $(OUT)"
