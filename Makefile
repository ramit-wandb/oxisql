default: build run

build:
	cargo build

run:
	./target/debug/oxisql -h 127.0.0.1 -P 3306 -u wandb -p wandb -D wandb_dev
