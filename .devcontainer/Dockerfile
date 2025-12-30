FROM rust:1.91.1-bookworm

ENV CARGO_BUILD_TARGET_DIR=/tmp/target
RUN rustup component add rustfmt clippy
RUN cargo install cargo-update
RUN cargo install cargo-lambda

# Install Zig for cargo-lambda
RUN cd /tmp && \
    wget -q https://ziglang.org/download/0.13.0/zig-linux-x86_64-0.13.0.tar.xz && \
    tar -xf zig-linux-x86_64-0.13.0.tar.xz && \
    mv zig-linux-x86_64-0.13.0 /usr/local/zig && \
    rm zig-linux-x86_64-0.13.0.tar.xz && \
    ln -s /usr/local/zig/zig /usr/local/bin/zig