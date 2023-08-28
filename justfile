fw_target := "thumbv6m-none-eabi"
wasm_target := "wasm32-unknown-unknown"


build-fw:
    cargo build --package sysbadge-fw --target {{fw_target}} --release -Z build-std=compiler_builtins,core,alloc

run-fw: build-fw
    probe-run --chip RP2040 ./target/{{fw_target}}/release/sysbadge-fw

build-simulator:
    cargo build --package sysbadge-simulator --release

run-simulator: build-simulator
    ./target/release/sysbadge-simulator

build-wasm:
    cargo build --target {{wasm_target}} -Z build-std=core,alloc,std,panic_abort --release --package sysbadge-web

build-wasm-bindings: build-wasm
    wasm-bindgen ./target/{{wasm_target}}/release/sysbadge_web.wasm --out-dir ./target/wasm32-unknown-unknown/release/pkg --target bundler

build-webpack: build-wasm-bindings
    cd ./web && yarn build