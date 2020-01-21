# TODO: Should use our own SDK container to build the lambda. Replace with SDK container when it's easier to pull.
BUILD_CONTAINER ?= clux/muslrust

build:
	docker run --network=host \
			--user "$(id -u):$(id -g)" \
			-v ${PWD}:/volume:ro \
			-v ${PWD}/target:/volume/target \
			-e SKIP_README=1 \
			--rm -t $(BUILD_CONTAINER) \
			cargo build --locked --release --target x86_64-unknown-linux-musl

zip: build ./target/x86_64-unknown-linux-musl/release/bootstrap
	zip -j refresh_timestamp_lambda.zip ./target/x86_64-unknown-linux-musl/release/bootstrap
