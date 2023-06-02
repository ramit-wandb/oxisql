default: build run

watch:
	cargo watch -w src -x 'run -- -h 127.0.0.1 -P 3306 -u wandb -p wandb -D wandb_dev'
	#cargo watch -w src -x 'run -- -h 127.0.0.1 -P 3306 -u wandb -p wandb -D wandb_dev -e "select count(*) from runs"'

build:
	cargo build

run:
	./target/debug/oxisql -h 127.0.0.1 -P 3306 -u wandb -p wandb -D wandb_dev

release:
	cargo build --release
	rm ~/.local/bin/oxisql
	cp ./target/release/oxisql ~/.local/bin/
