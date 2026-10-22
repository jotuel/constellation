#!/bin/bash
set -e

# Compile the binary in release mode (with debug symbols for clean stacks)
echo "Building binary with debug info..."
RUSTFLAGS="-g" cargo build --release

# Select the profiling tool (samply, heaptrack, or direct)
PROFILE_TOOL=${1:-"samply"} 
BINARY="./target/release/cosmic-ext-constellations"
SCREENS_DIR="./scratch/screens"
mkdir -p "$SCREENS_DIR"

echo "Running profile using: $PROFILE_TOOL"

case $PROFILE_TOOL in
    "samply")
        # Start samply in background; it will automatically launch the app
        samply record $BINARY &
        TOOL_PID=$!
        ;;
    "heaptrack")
        # Start heaptrack in background
        heaptrack $BINARY &
        TOOL_PID=$!
        ;;
    *)
        echo "Running directly without native profiler..."
        $BINARY &
        TOOL_PID=$!
        ;;
esac

# Wait for application window to load and acquire focus
sleep 15

# Trigger the UI automation sequence via wtype
echo "Simulating user interaction..."
for i in {1..6}; do
    wtype -P Tab -p Tab
    sleep 0.2
done
wtype -P Return -p Return
sleep 5

# Capture visual feedback to verify target state was reached
cosmic-screenshot --interactive=false --notify=false --save-dir "$SCREENS_DIR"
mv "$SCREENS_DIR"/Screenshot_*.png "$SCREENS_DIR"/profiling_visual_state.png

# Kill the target application (which causes the profiler to finish and write logs)
APP_NAME=$(basename "$BINARY")
killall "$APP_NAME" || true
wait $TOOL_PID || true

echo "Profiling session completed."
