# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project overview

Rust CLI tools for translating games using NVS/FVP visual novel engine.

## Project structure

```
Cargo.toml
build.rs           -- Windows resource embedding (icon, manifest)
src/
  main.rs          -- CLI entry point (argh), dispatch
  cli.rs           -- CLI argument definitions
  lib.rs           -- Library re-exports (main.rs imports from lib crate)
  archive.rs       -- Packed archive ls, extract, get, pack, replace, validate
  nvsg.rs          -- NVSG image info, decode + encode
  hcb.rs           -- HCB bytecode disassembly/assembly
  utils/
    mod.rs         -- Utility module re-exports
    bitmap.rs      -- Pixel format conversions (BGR/RGB swap, grayscale, mask)
    fs.rs          -- Filesystem helpers
    hzc1.rs        -- HZC1 zlib compression wrapper (decompress/compress)
    opcode.rs      -- HCB opcode definitions, encode/decode, mnemonics
    png.rs         -- PNG load/save
    strings.rs     -- Strings file load/save (#include, #emit, #reference, comments)
    text.rs        -- SJIS sort comparison for archive entry ordering
tests/
  archive.rs       -- Archive pack/unpack/get/replace tests
  hcb.rs           -- HCB disassembly/assembly roundtrip and string loading tests
  hzc1.rs          -- HZC1 decompression error handling tests
  nvsg.rs          -- NVSG decode/encode roundtrip tests
  fixtures/        -- Binary test fixtures (graph.bin, *.nvsg, *.hcb)
```

## Building and testing

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy
```

Verify that `cargo fmt --check` and `cargo clippy` produce no warnings before committing. Formatting is configured in `rustfmt.toml` (2-space indent, 120-char max width).

## Key format details

- **Packed archives**: flat-file containers with SJIS filenames. Entries must be sorted to match the engine's `lstrcmpiA`-based binary search: symbols/punctuation < digits < letters (case-insensitive), with SJIS multi-byte characters after ASCII.
- **NVSG images**: 5 types (BGR, BGRA, Parts, Mask, Gaiji). Version field is big-endian, all other header fields are little-endian. Pixel data is BGR/BGRA order (not RGB).
- **HZC1**: zlib compression wrapper used by NVSG. The inner header (prefix) is stored uncompressed; only pixel data is zlib-compressed. Shared logic lives in `utils/hzc1.rs`.
- **HCB bytecode**: binary format with `[u32 descriptor_offset][code][descriptor]`. Code base starts at offset 4. Descriptor contains entry point, global counts, game mode, SJIS title, syscalls, and custom syscalls.
- **Strings file** (`strings.txt`): flat text, one string per line. Blank lines ignored. `;` at line start is a comment. `#include <file>` expands included files recursively (with cycle detection). `#reference <file>` is informational metadata for other tools and is skipped during loading. `#emit empty` emits an empty string.
- **ASM format**: tab-indented mnemonics, `LABEL:` references, `STRING:N` (1-indexed) string references, `;` comments. Config `entry_point` and `custom_syscalls[].address` reference labels by name. Assembler uses two-pass: pass 1 collects labels, pass 2 emits bytes.

## Conventions

- Use `anyhow` for error handling, `bail!` for early returns.
- Print progress/diagnostics to stderr, not stdout.
- Filenames extracted from BIN archives use their original names.
