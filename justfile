fw_target := "thumbv6m-none-eabi"


build-fw:
    cargo build --package sysbadge-fw --target {{fw_target}} --release -Z build-std=compiler_builtins,core,alloc

run-fw: build-fw
    probe-run --chip RP2040 ./target/{{fw_target}}/release/sysbadge-fw

build-simulator:
    cargo build --package sysbadge-simulator --release

run-simulator: build-simulator
    ./target/release/sysbadge-simulator