from .. import run, crate
import os

run(
    f"wasm-pack build crates/{crate}_ffi --target web --out-dir ../../build/typescript/{crate} -- --color always --no-default-features --features ffi_wasm"
)

# Remove the generated .gitignore file from the pkg directory
if os.path.exists(f"build/typescript/{crate}/.gitignore"):
    os.remove(f"build/typescript/{crate}/.gitignore")