#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

exec "$SCRIPT_DIR/../why2025-badge-sys-bindings/rebuild-bindings.sh" "$@"#!/usr/bin/env bash
set -e

FIRMWARE_REPO_URL="https://gitlab.com/why2025/team-badge/firmware"

if test -d firmware; then
    git -C firmware remote set-url origin "$FIRMWARE_REPO_URL"
    git -C firmware fetch origin
    git -C firmware checkout origin/main
else
    git clone "$FIRMWARE_REPO_URL" firmware
    git -C firmware checkout origin/main
fi

FILES=( $(yq --raw-output '.include[]' firmware/badgevms/symbols.yml ) )
SIMPLE_FUNCTIONS=( $(yq --raw-output '.simple_function[]' firmware/badgevms/symbols.yml ) )
EXTERN_SIMPLE_FUNCTIONS=( $(yq --raw-output '.simple_function_extern[]' firmware/badgevms/symbols.yml ) )
WRAPPED_FUNCTIONS=( $(yq --raw-output '.wrapped_function[]' firmware/badgevms/symbols.yml ) )
SIMPLE_OBJECTS=( $(yq --raw-output '.simple_object[]' firmware/badgevms/symbols.yml ) )
WRAPPED_OBJECTS=( $(yq --raw-output '.wrapped_object[]' firmware/badgevms/symbols.yml ) )
NORMALIZED_FUNCTION_EXPORTS=( __errno )
BADGE_TARGET=riscv32imafc-unknown-none-elf
RAW_BINDINGS_DIR=../why2025-badge-sys-bindings/src

# Enums are by default represented as Rust enums. Add them here to make them bitfields insteads.
BITFIELD_ENUMS=(
    window_flag_t
)

RUST_ENUMS=(
    keyboard_scancode_t
    pixel_format_t
)

BINDGEN_COMMAND=(
    bindgen
    headers/all.h
    --use-array-pointers-in-arguments
    --use-core
    --rust-edition 2024
    --rust-target 1.89
    exec "$SCRIPT_DIR/../why2025-badge-sys-bindings/rebuild-bindings.sh" "$@"