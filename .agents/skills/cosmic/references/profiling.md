# COSMIC Application Profiling Guide

Reference guide for profiling CPU execution hotspots and memory usage of native COSMIC/Wayland applications during deterministic UI interactions.

## Prerequisites
Ensure target profiling tools are installed:
* **`samply`**: CPU execution profiler (runs on cargo-installed binary).
* **`heaptrack`**: Memory heap allocation profiler.

## Instructions
1. Run the profiling script located at `scripts/profile_run.sh <tool>` (where `<tool>` is `samply` or `heaptrack`, default is `samply`).
2. Analyze output:
   - For `samply`: Load the generated `.json` files into [profiler.firefox.com](https://profiler.firefox.com/).
   - For `heaptrack`: Run `heaptrack_gui heaptrack.<binary>.<pid>.gz` to inspect heap allocation size and leaks.

## Script Details
The profiling script (`scripts/profile_run.sh`):
- Compiles the application in release mode with debug symbols (`RUSTFLAGS="-g" cargo build --release`).
- Attaches the selected profiler (`samply` or `heaptrack`).
- Uses `wtype` to simulate standard user interactions under load.
- Captures visual state verification via `cosmic-screenshot`.
- Cleans up the target process and finalizes profiler data generation.
