.PHONY: services

# Build cargo-jam and then build the jam-null-authorizer service
services:
	cargo build --release -p cargo-jam
	./target/release/cargo-jam build -p services/jam-null-authorizer
