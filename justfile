run:
    cargo fmt
    cargo run --release

build_windows:
    cargo build --release --target x86_64-pc-windows-gnu

build:
    cargo build --release
