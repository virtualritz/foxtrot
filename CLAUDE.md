# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
Foxtrot is an experimental fast STEP file viewer for mechanical CAD files. It's a complete implementation built from the ground up, with custom libraries for parsing STEP files and triangulation, supporting both native GUI and WebAssembly browser deployment.

## Build and Development Commands

### Essential Commands
```bash
# Build entire workspace
cargo build --release

# Run native GUI with a STEP file
cargo run --release -- examples/cube_hole.step

# Run tests
cargo test

# Build WebAssembly version
cd wasm
wasm-pack build --target no-modules
python3 -m http.server --directory deploy

# Regenerate STEP parser from EXPRESS schema (rarely needed)
cargo run --release --example gen_exp -- path/to/APs/10303-214e3-aim-long.exp step/src/ap214.rs
```

## High-Level Architecture

### Workspace Structure
The project is a Cargo workspace with 6 main crates and 1 separate WebAssembly crate:

1. **`step`**: Auto-generated STEP file parser from EXPRESS schemas. The `ap214.rs` file (1.6MB) is auto-generated and very slow to compile. Core parsing logic is in `parse.rs`.

2. **`express`**: EXPRESS schema parser and code generator. Used to generate the STEP parser. Contains 108K lines in `parse.rs` and 36K lines in `gen.rs`.

3. **`triangulate`**: Converts STEP geometry into triangle meshes. Main logic in `triangulate.rs` (31K lines) and `surface.rs` (15K lines). Depends on `cdt`, `nurbs`, and `step`.

4. **`cdt`**: Standalone constrained Delaunay triangulation library with exact geometric predicates. Can handle up to 500M points with `long-indexes` feature.

5. **`nurbs`**: Mathematical algorithms for NURBS and B-spline curves/surfaces. Uses single-character variable names matching 1970s algorithm conventions.

6. **`gui`**: WebGPU-based native viewer application. Entry point is `main.rs` with async loading. Shaders are in `model.wgsl` and `backdrop.wgsl`.

7. **`wasm`** (separate): WebAssembly interface for browser deployment. Excluded from main workspace. Provides single function `step_to_triangle_buf()` for STEP→mesh conversion.

### Key Architectural Patterns

- **Code Generation**: The STEP parser is auto-generated from EXPRESS schemas. Avoid manually editing `step/src/ap214.rs`.
- **Parallelization**: Most crates support optional `rayon` feature for parallel processing (disabled in WebAssembly builds).
- **Exact Predicates**: CDT uses exact geometric predicates for numerical robustness.
- **Separation of Concerns**: Clear boundaries between parsing, geometry, triangulation, and rendering layers.

### Performance Considerations

- Release builds use single codegen unit and include debug symbols
- The `step/src/ap214.rs` file is extremely slow to compile due to its size
- WebAssembly builds disable parallelization features
- CDT supports incremental triangulation for debugging

### Important Files to Know

- `step/src/step_file.rs`: Main interface for STEP file handling
- `triangulate/src/mesh.rs`: Mesh data structures
- `gui/src/app.rs`: Main application logic for native viewer
- `wasm/src/lib.rs`: WebAssembly interface implementation
- Workspace root `Cargo.toml`: Workspace configuration and shared settings