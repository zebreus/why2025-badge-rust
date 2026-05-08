#!/usr/bin/env bash
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
    --generate-cstr
    --impl-debug
    --default-enum-style rust
    --merge-extern-blocks
    # This might be dangerous, when calling host functions
    --explicit-padding
    --no-layout-tests
    # --no-size_t-is-usize
    --generate-inline-functions
    --rustfmt-configuration-file $(pwd)/../rustfmt.toml
    --allowlist-item stdout
    # Extra types and header-only functions
    --allowlist-item 'BADGEVMS_KEY_.*' 
    --allowlist-item 'BADGEVMS_KMOD_.*'
)
for function in "${SIMPLE_FUNCTIONS[@]}" "${EXTERN_SIMPLE_FUNCTIONS[@]}" "${WRAPPED_FUNCTIONS[@]}" "${NORMALIZED_FUNCTION_EXPORTS[@]}"; do
    BINDGEN_COMMAND+=( --allowlist-function "$function" )
done
for var in "${SIMPLE_OBJECTS[@]}" "${WRAPPED_OBJECTS[@]}"; do
    BINDGEN_COMMAND+=( --allowlist-var "$var" )
done

for enum in "${RUST_ENUMS[@]}" ; do
    BINDGEN_COMMAND+=( --rustified-enum "$enum" )
done

for enum in "${BITFIELD_ENUMS[@]}" ; do
    BINDGEN_COMMAND+=( --bitfield-enum "$enum" )
done

function normalize_function_export() {
    local symbol="$1"

    echo "$symbol"
}

function normalize_var_export() {
    local symbol="$1"

    case "$symbol" in
        _ctype_)
            # Upstream exports `_ctype_` in symbols.yml, but the public header only exposes it as
            # a macro alias to `_ctype_b + _CTYPE_OFFSET`. Keep the bindings on the real declared
            # backing object until we have an explicit, selective forwarding strategy for libc-
            # owned symbols (for example linker-wrapped shims that still delegate to the real host
            # libc implementation).
            echo "_ctype_b"
            ;;
        *)
            echo "$symbol"
            ;;
    esac
}

function symbol_kind_for_bindings() {
    local symbol="$1"

    case "$symbol" in
        __errno)
            echo "function"
            ;;
        _ctype_)
            # See docs/adr/0001-ctype-export-normalization.md.
            echo "var"
            ;;
        *)
            echo "auto"
            ;;
    esac
}

function bindings_have_function() {
    local file="$1"
    local symbol="$2"
    local normalized_symbol

    normalized_symbol="$(normalize_function_export "$symbol")"
    grep -q "fn $normalized_symbol(" "$file"
}

function bindings_have_var() {
    local file="$1"
    local symbol="$2"
    local normalized_symbol

    normalized_symbol="$(normalize_var_export "$symbol")"
    grep -aq "$normalized_symbol:" "$file"
}

function badge_target_is_available() {
    local target_libdir

    if ! target_libdir="$(rustc --print target-libdir --target "$BADGE_TARGET" 2>/dev/null)"; then
        return 1
    fi

    [ -d "$target_libdir" ]
}

function emit_linker_function_assertion() {
    local symbol="$1"
    local normalized_symbol
    local assertion_key

    normalized_symbol="$(normalize_function_export "$symbol")"
    assertion_key="function:$normalized_symbol"
    if [ -n "${GENERATED_LINKER_ASSERTIONS[$assertion_key]+x}" ]; then
        return 0
    fi
    GENERATED_LINKER_ASSERTIONS[$assertion_key]=1

    echo "            assert_ne!($normalized_symbol as *const (), core::ptr::null());" >> src/linker_test.rs
}

function emit_linker_var_assertion() {
    local symbol="$1"
    local normalized_symbol
    local assertion_key

    normalized_symbol="$(normalize_var_export "$symbol")"
    assertion_key="var:$normalized_symbol"
    if [ -n "${GENERATED_LINKER_ASSERTIONS[$assertion_key]+x}" ]; then
        return 0
    fi
    GENERATED_LINKER_ASSERTIONS[$assertion_key]=1

    echo "            assert_ne!(core::ptr::addr_of!($normalized_symbol) as *const (), core::ptr::null());" >> src/linker_test.rs
}

function ensure_include_after_pragma_once() {
    local file="$1"
    local include_line="$2"

    if grep -Fqx "$include_line" "$file"; then
        return 0
    fi

    awk -v include_line="$include_line" '
        { print }
        /^#pragma once$/ {
            print ""
            print include_line
        }
    ' "$file" > "$file.tmp"
    mv "$file.tmp" "$file"
}

function patch_headers_for_bindgen() {
    ensure_include_after_pragma_once headers/wrapped_funcs.h "#include <netinet/in.h>"
    ensure_include_after_pragma_once headers/sys/socket.h "#include <netinet/in.h>"
}

function prepare_headers() {
    rm -rf headers
    mkdir -p headers
    cp -r firmware/sdk_include/* headers
    mkdir -p headers/badgevms
    cp -r firmware/badgevms/include/badgevms/* headers/badgevms
    mkdir -p headers/curl
    cp -r firmware/badgevms/include/curl/* headers/curl
    cp ./extra-headers/sdkconfig.h headers
    cp ./extra-headers/missing.h headers
    cp ./extra-headers/gcc-builtins.h headers
    cp ./extra-headers/esp-termios.h headers
    cp -r firmware/components/why_stdio/stdlib/* headers
    cp firmware/badgevms/wrapped_funcs.h headers
    for file in "${FILES[@]}"; do
        if ! test -f "headers/$file"; then
            echo "Missing header file: $file"
            exit 1
        fi
    done

    # Upstream currently relies on transitive includes for in_addr/in6_addr.
    # Patch the copied headers locally so bindgen sees self-contained headers.
    patch_headers_for_bindgen
}

prepare_headers

echo "#define _BEGIN_STD_C" > headers/all.h
echo "#define _END_STD_C" >> headers/all.h
echo "#define _GNU_SOURCE 1" >> headers/all.h
echo "#define _DEFAULT_SOURCE 1" >> headers/all.h
echo "#define __GNU_VISIBLE 1" >> headers/all.h
echo "#define __XSI_VISIBLE 1" >> headers/all.h
echo "#define __BSD_VISIBLE 1" >> headers/all.h
# echo "#define DIR void" >> headers/all.h
for file in "${FILES[@]}"; do
    echo "#include \"$file\"" >> headers/all.h
done
# Pull in __assert_func from the upstream SDK headers.
echo "#include \"assert.h\"" >> headers/all.h
# Include some missing defines
echo "#include \"missing.h\"" >> headers/all.h
# So the tcgetattr function provided by the badge libc uses the termios struct from the esp-idf libc
echo "#include \"esp-termios.h\"" >> headers/all.h
echo "#include \"gcc-builtins.h\"" >> headers/all.h

export BINDGEN_EXTRA_CLANG_ARGS="-isystem headers -target riscv32-esp-elf"

"${BINDGEN_COMMAND[@]}" -o src/generated.rs --generate 'functions,methods,constructors,destructors,types,vars'

# Split in types and functions
sed -Ez      's|unsafe extern "C" \{\n(    [^\n]*\n)+}\n||g' src/generated.rs > src/types.rs
echo $'use crate::types::*;\n' > src/bindings.rs
grep -Pazo '(?s)unsafe extern "C" \{\n(    [^\n]*\n)+}\n' src/generated.rs | tr -d '\0' >> src/bindings.rs

# Remove the unsplit file
rm src/generated.rs

function check_functions {
    local file="$1"
    shift
    local functions=("$@")
    local missing_functions=false
    local missing_functions_list=()
    for function in "${functions[@]}"; do
        case "$(symbol_kind_for_bindings "$function")" in
            var)
                if bindings_have_var "$file" "$function"; then
                    continue
                fi
                ;;
            *)
                if bindings_have_function "$file" "$function"; then
                    continue
                fi
                ;;
        esac

            echo "Missing function: $function"
            missing_functions=true
            missing_functions_list+=("$function")
    done
    if $missing_functions; then
        echo "Missing functions!!!"
        for function in "${missing_functions_list[@]}"; do
            echo "    $function"
        done
        echo "Please update the symbols.yml file or the headers."
        return 1
    fi
}
function check_vars {
    local file="$1"
    shift
    local vars=("$@")
    local missing_vars=false
    local missing_vars_list=()
    for var in "${vars[@]}"; do
        case "$(symbol_kind_for_bindings "$var")" in
            function)
                if bindings_have_function "$file" "$var"; then
                    continue
                fi
                ;;
            *)
                if bindings_have_var "$file" "$var"; then
                    continue
                fi
                ;;
        esac

            echo "Missing variable: $var"
            missing_vars=true
            missing_vars_list+=("$var")
    done
    if $missing_vars; then
        echo "Missing variables!!!"
        for var in "${missing_vars_list[@]}"; do
            echo "    $var"
        done
        echo "Please update the symbols.yml file or the headers."
        return 1
    fi
}

check_functions src/bindings.rs "${SIMPLE_FUNCTIONS[@]}"
check_functions src/bindings.rs "${EXTERN_SIMPLE_FUNCTIONS[@]}"
check_functions src/bindings.rs "${WRAPPED_FUNCTIONS[@]}"
check_functions src/bindings.rs "${NORMALIZED_FUNCTION_EXPORTS[@]}"

check_vars src/bindings.rs "${SIMPLE_OBJECTS[@]}"
check_vars src/bindings.rs "${WRAPPED_OBJECTS[@]}"

# Generate a test that links all symbols

cat <<EOF > src/linker_test.rs
#[cfg(test)]
mod tests {
    #[test]
    fn link_all_symbols() {
        use crate::*;

        unsafe {
EOF

declare -A GENERATED_LINKER_ASSERTIONS=()

for symbol in "${SIMPLE_FUNCTIONS[@]}" "${EXTERN_SIMPLE_FUNCTIONS[@]}" "${WRAPPED_FUNCTIONS[@]}"; do
    case "$(symbol_kind_for_bindings "$symbol")" in
        var)
            emit_linker_var_assertion "$symbol"
            ;;
        *)
            emit_linker_function_assertion "$symbol"
            ;;
    esac
done

for symbol in "${NORMALIZED_FUNCTION_EXPORTS[@]}"; do
    emit_linker_function_assertion "$symbol"
done

for symbol in "${SIMPLE_OBJECTS[@]}" "${WRAPPED_OBJECTS[@]}"; do
    case "$(symbol_kind_for_bindings "$symbol")" in
        function)
            emit_linker_function_assertion "$symbol"
            ;;
        *)
            emit_linker_var_assertion "$symbol"
            ;;
    esac
done

emit_linker_function_assertion printf

cat <<EOF >> src/linker_test.rs

        }
    }
}
EOF

if badge_target_is_available; then
    cargo check --target "$BADGE_TARGET" --lib
else
    echo "Skipping badge target cargo check because $BADGE_TARGET is not installed."
fi