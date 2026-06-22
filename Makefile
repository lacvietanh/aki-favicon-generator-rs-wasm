.PHONY: build clean

build:
	wasm-pack build --target web --release
	wasm-opt -Oz --all-features pkg/aki_favicon_generator_bg.wasm -o pkg/aki_favicon_generator_bg.wasm

sync-akitao: build
	rm -rf /Volumes/DEV/www/akitao.com/app/tools/favicon-generator/wasm/*
	cp -r pkg/* /Volumes/DEV/www/akitao.com/app/tools/favicon-generator/wasm/

clean:
	cargo clean
	rm -rf pkg demo/dist demo/node_modules
