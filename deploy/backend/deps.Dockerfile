FROM rust-builder:1.95-bookworm AS planner

WORKDIR /app/backend

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/crates ./crates
COPY backend/patches ./patches
COPY backend/xtask ./xtask

RUN cargo chef prepare --recipe-path recipe.json

FROM rust-builder:1.95-bookworm

WORKDIR /app/backend

COPY --from=planner /app/backend/recipe.json recipe.json
COPY backend/patches ./patches

RUN cargo chef cook --release --locked --recipe-path recipe.json
