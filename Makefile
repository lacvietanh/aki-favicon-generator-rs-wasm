.PHONY: build clean

build:
	wasm-pack build --target web --release
	wasm-opt -Oz --all-features pkg/aki_favicon_generator_bg.wasm -o pkg/aki_favicon_generator_bg.wasm

clean:
	cargo clean
	rm -rf pkg demo/dist demo/node_modules
