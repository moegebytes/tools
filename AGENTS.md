# AGENTS.md

This file provides guidance to agents working in this repository.

## Project overview

Rust CLI tools for translating games using the NVS/FVP visual novel engine.

## Important paths

- `src/archive.rs`: packed archive operations
- `src/nvsg.rs`: NVSG image decode/encode
- `src/hcb.rs`: HCB disassembly/assembly
- `src/vm/`: VM instruction, opcode, and stack helpers
- `src/utils/`: shared helpers such as `hzc1`, `opcode`, `strings`, and text/filesystem utilities
- `tests/`: roundtrip and format coverage

## Build and development

```bash
cargo test
cargo fmt --check
cargo clippy
```

Keep `cargo fmt --check` clean and `cargo clippy` warning-free. Formatting uses 2-space indent and 120-char line width.

## Format rules that affect correctness

- Packed archives use SJIS filenames and must sort like Windows `lstrcmpiA`: symbols/punctuation < digits < letters (case-insensitive), with SJIS multi-byte characters after ASCII.
- NVSG: the version field is big-endian; other header fields are little-endian; pixel data is stored as BGR/BGRA.
- HZC1: the inner prefix stays uncompressed; only the payload is zlib-compressed.
- HCB layout is `[u32 descriptor_offset][code][descriptor]`, with code starting at offset 4.
- Strings files ignore blank lines, use `;` for whole-line comments, expand `#include` recursively with cycle detection, ignore `#reference`, and treat `#emit empty` as an empty string.
- ASM uses tab-indented mnemonics, `LABEL:` labels, `STRING:N` references, `;` comments, and two-pass assembly for label resolution.

## Conventions

- Use `anyhow` for errors and `bail!` for early returns.
- Print progress and diagnostics to stderr.
- Preserve original filenames when extracting BIN archives.
