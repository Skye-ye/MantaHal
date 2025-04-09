FROM ubuntu:22.04

ARG HOME=/root

# 0. Set up mirrors and install basic tools
RUN sed -i s@/archive.ubuntu.com/@/mirrors.tuna.tsinghua.edu.cn/@g /etc/apt/sources.list
RUN sed -i s@/security.ubuntu.com/@/mirrors.tuna.tsinghua.edu.cn/@g /etc/apt/sources.list
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    wget git vim ca-certificates build-essential

# 1. Set up Rust
# - https://learningos.github.io/rust-based-os-comp2022/0setup-devel-env.html#qemu
# - https://www.rust-lang.org/tools/install
# - https://github.com/rust-lang/docker-rust/blob/master/Dockerfile-debian.template

# 1.1. Install
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=nightly-2025-01-18 \
    PROFILE=minimal
RUN set -eux; \
    wget --progress=dot:giga https://sh.rustup.rs -O rustup-init; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile $PROFILE --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME;

# 1.2. Add targets and components
RUN rustup target add riscv64gc-unknown-none-elf && \
    rustup target add loongarch64-unknown-none && \
    rustup component add rust-src && \
    rustup component add rustfmt && \
    rustup component add clippy && \
    rustup component add llvm-tools && \
    cargo install cargo-binutils && \
    cargo install rustfilt

# 2. Sanity checking
RUN rustup --version && \
    cargo --version && \
    rustc --version && \
    rustup toolchain list && \
    rustup target list --installed

# Ready to go
WORKDIR /mnt
