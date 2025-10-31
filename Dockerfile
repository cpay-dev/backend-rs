FROM rust:1.91.0-slim-trixie AS builder

ARG PACKAGE
ARG BIN

WORKDIR /workspace

ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV PATH=/usr/local/cargo/bin:/usr/local/rustup/bin:$PATH

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY crates/api/dashboard/Cargo.toml crates/api/dashboard/Cargo.toml

RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
  --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
  cargo fetch --locked

COPY . .

RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
  --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
  --mount=type=cache,id=cargo-target,target=/workspace/target \
  BIN_NAME="${BIN:-$PACKAGE}" \
  && cargo build --locked --release -p "$PACKAGE" \
  && install -Dm755 "target/release/${BIN_NAME}" /out/app \
  && ln -sf app "/out/${BIN_NAME}"

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime

USER nonroot:nonroot
WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /out/ /usr/local/bin/

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

ENTRYPOINT ["/usr/local/bin/app"]
