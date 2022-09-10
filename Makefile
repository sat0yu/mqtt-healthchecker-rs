current_dir := $(shell pwd)

release: mqtt-healthchecker-rs.tar.gz mqtt-healthchecker-rs_musl.tar.gz

mqtt-healthchecker-rs.tar.gz: buider_image
	docker run --rm -it -v "$(current_dir)":/home/rust/src -w /home/rust/src buider_image cargo build --release
	tar czvf mqtt-healthchecker-rs.tar.gz target/release/mqtt-healthchecker-rs

mqtt-healthchecker-rs_musl.tar.gz:
	docker run --rm -it -v "$(current_dir)":/home/rust/src ekidd/rust-musl-builder cargo build --release
	tar czvf mqtt-healthchecker-rs_musl.tar.gz target/x86_64-unknown-linux-musl/release/mqtt-healthchecker-rs

buider_image:
	docker build ./.devcontainer -t mqtt_healthchecker_buider
