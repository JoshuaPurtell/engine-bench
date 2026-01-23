FROM rust:1.82-bookworm

WORKDIR /app

# Only copy scaffold workspace into the image.
# Tests and solutions are injected by Harbor at runtime.
COPY scaffold/ /app/

# Overwrite the target card with the task stub.
# Note: scaffold IS the tcg_expansions crate, so paths are /app/src/...
COPY stubs/df_018_ledian.rs /app/src/df/cards/df_018_ledian.rs
