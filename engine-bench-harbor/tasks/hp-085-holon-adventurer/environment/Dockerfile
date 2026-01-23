FROM rust:1.82-bookworm

WORKDIR /app

# Only copy scaffold workspace into the image.
# Tests and solutions are injected by Harbor at runtime.
COPY scaffold/ /app/

# Overwrite the target card with the task stub.
# Note: scaffold IS the tcg_expansions crate, so paths are /app/src/...
COPY stubs/hp_085_holon_adventurer.rs /app/src/hp/cards/hp_085_holon_adventurer.rs
