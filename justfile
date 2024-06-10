run:
    cargo fmt
    RUST_LOG=info mangohud cargo run --release

build_windows:
    cargo build --release --target x86_64-pc-windows-gnu

build:
    cargo build --release
