
FROM johnpucciarelli/debian-rust-gtk4:v2.1.0

WORKDIR /usr/src/build_rust
COPY . .

RUN apt update -y && apt upgrade -y && apt install -y libdbus-1-dev pkg-config libudev-dev libatk1.0-dev librust-atk-dev libgdk3.0-cil-dev librust-gdk-dev librust-gdk4-dev

RUN cargo build --release
