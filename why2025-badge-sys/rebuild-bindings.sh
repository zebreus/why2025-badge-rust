#!/usr/bin/env bash
set -e

if test -d firmware; then
    cd firmware
    git fetch
    git checkout origin/main
    cd -
else
    # Using a fork, because we need a small patch to the headers
    git clone https://gitlab.com/zebreus/firmware firmware
    git checkout origin/main
fi

FILES=( $(yq --raw-output '.include[]' firmware/badgevms/symbols.yml ) )
SIMPLE_FUNCTIONS=( $(yq --raw-output '.simple_function[]' firmware/badgevms/symbols.yml ) )
EXTERN_SIMPLE_FUNCTIONS=( $(yq --raw-output '.simple_function_extern[]' firmware/badgevms/symbols.yml ) )
WRAPPED_FUNCTIONS=( $(yq --raw-output '.wrapped_function[]' firmware/badgevms/symbols.yml ) )
SIMPLE_OBJECTS=( $(yq --raw-output '.simple_object[]' firmware/badgevms/symbols.yml ) )
# Exclude errno from the wrapped objects, as it appears to be a function. fix this if you need it.
WRAPPED_OBJECTS=( $(yq --raw-output '.wrapped_object[]' firmware/badgevms/symbols.yml | grep -v '__errno') )

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
    --no-size_t-is-usize
    --generate-inline-functions
    --rustfmt-configuration-file $(pwd)/../rustfmt.toml
    --allowlist-item stdout
)
for function in "${SIMPLE_FUNCTIONS[@]}" "${EXTERN_SIMPLE_FUNCTIONS[@]}" "${WRAPPED_FUNCTIONS[@]}"; do
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
        if [ "$function" == "_ctype_" ]; then
            # ctype is wrongly? listed as a function
            continue
        fi
        if ! grep -q "fn $function(" "$file"; then
            echo "Missing function: $function"
            missing_functions=true
            missing_functions_list+=("$function")
        fi
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
        if ! grep -aq "$var:" "$file"; then
            echo "Missing variable: $var"
            missing_vars=true
            missing_vars_list+=("$var")
        fi
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

for symbol in "${SIMPLE_FUNCTIONS[@]}" "${EXTERN_SIMPLE_FUNCTIONS[@]}" "${WRAPPED_FUNCTIONS[@]}"; do
    if [ "$symbol" == "_ctype_" ]; then
        # ctype is wrongly? listed as a function
        continue
    fi
    echo "            assert_ne!($symbol as *const (), core::ptr::null());" >> src/linker_test.rs
done

cat <<EOF >> src/linker_test.rs

            assert_ne!(printf as *const (), core::ptr::null());
        }
    }
}
EOF