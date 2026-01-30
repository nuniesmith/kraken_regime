#!/bin/bash

# analyze.sh
# Usage: ./analyze.sh <directory_path> [--full]

TARGET_DIR=""
FULL_MODE=false
OUTPUT_FILE="codebase_summary.md"

# --- 1. Argument Parsing ---
for arg in "$@"; do
    if [[ "$arg" == "--full" ]]; then
        FULL_MODE=true
    elif [[ -d "$arg" ]]; then
        TARGET_DIR="$arg"
    else
        echo "Error: '$arg' is not a valid directory."
        echo "Usage: $0 <directory_path> [--full]"
        exit 1
    fi
done

if [[ -z "$TARGET_DIR" ]]; then
    echo "Usage: $0 <directory_path> [--full]"
    exit 1
fi

# --- 2. Setup Output ---
echo "# Codebase Analysis: $TARGET_DIR" > "$OUTPUT_FILE"
echo "Generated on: $(date)" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# --- 3. Generate Directory Tree ---
echo "## Project Structure" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"

# Define exclude pattern for tree (Python, Rust, Kotlin, Git artifacts)
EXCLUDES="node_modules|__pycache__|target|build|.git|.idea|.vscode|venv|.gradle|*.class|*.jar|*.o|*.so"

if command -v tree &> /dev/null; then
    # Use 'tree' if installed (Best visualization)
    tree "$TARGET_DIR" -I "$EXCLUDES" --dirsfirst >> "$OUTPUT_FILE"
else
    # Fallback to 'find' if 'tree' is not installed
    echo "('tree' command not found, using 'find' fallback)" >> "$OUTPUT_FILE"
    find "$TARGET_DIR" -maxdepth 4 -not -path '*/.*' >> "$OUTPUT_FILE"
fi

echo '```' >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# --- 4. Generate File Contents (Only if --full) ---
if [ "$FULL_MODE" = true ]; then
    echo "Processing file contents..."
    echo "## File Contents" >> "$OUTPUT_FILE"

    # Find files, pruning specific build/config directories
    find "$TARGET_DIR" -type d \( \
        -name ".git" -o \
        -name "__pycache__" -o \
        -name "target" -o \
        -name "build" -o \
        -name "venv" -o \
        -name ".idea" -o \
        -name ".gradle" -o \
        -name "node_modules" \
    \) -prune -o -type f -print | while read -r file; do

        # Check if file is text (skips binaries/images)
        if grep -Iq . "$file" 2>/dev/null; then
            echo "### File: $file" >> "$OUTPUT_FILE"

            # Extract extension for Markdown syntax highlighting
            ext="${file##*.}"

            echo "\`\`\`$ext" >> "$OUTPUT_FILE"
            cat "$file" >> "$OUTPUT_FILE"
            echo -e "\n\`\`\`" >> "$OUTPUT_FILE"
            echo "---" >> "$OUTPUT_FILE"
        fi
    done
fi

echo "✅ Analysis complete. Output saved to: $OUTPUT_FILE"
