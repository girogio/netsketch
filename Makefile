.SILENT:

.PHONY: all

server:
	cargo run --package netsketch-server -- --address localhost --port 6666

client-a:
	cargo run --package netsketch -- --address localhost --port 6666 --nickname girogio

client-b:
	cargo run --package netsketch -- --address localhost --port 6666 --nickname mario

install:
	cargo install --path ns-client
	cargo install --path ns-server
