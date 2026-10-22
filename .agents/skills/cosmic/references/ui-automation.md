# COSMIC UI/UX Automation Guide

Reference guide for verifying and inspecting the layout of native COSMIC/Wayland applications using automated keyboard simulation and screen capture.

## Prerequisites
Ensure the system has:
* **`wtype`**: Virtual keyboard simulator for Wayland.
* **`cosmic-screenshot`**: Native screenshot utility.

## Instructions
1. Run the automated navigation and capture script located at `scripts/test_ui.sh`.
2. Inspect the saved frames under `scratch/screens/` for visual defects:
   - Text node overflow/truncation.
   - Clipped/hidden buttons (such as the Join button next to long room names).
   - Missing horizontal/vertical pane separators.
   - Selection indicator highlights on focused list elements.

## Script Details
The automation script (`scripts/test_ui.sh`):
- Launches the target debug binary in the background.
- Synthesizes Wayland keyboard events with `wtype` (e.g., `Tab` and `Return` keystrokes).
- Uses `cosmic-screenshot` to capture non-interactive screenshots of the rendered state.
- Stores output frames in `./scratch/screens/` and terminates the background application process.
