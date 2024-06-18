# 1. collect *.rs files
# 2. run each file with `cargo run --example <file_name>`.
# 3. check if it returns 0.

# get the path of this script
SCRIPT_PATH=$(dirname "$(realpath "$0")")

# collect *.rs files in this directory non-recursively
EXAMPLES=$(find "$SCRIPT_PATH" -maxdepth 1 -name "*.rs")

# run each file and check if it returns 0
for example in $EXAMPLES; do
    echo "Running $example"
    cargo run --example "$(basename "$example" .rs)"
    if [ $? -ne 0 ]; then
        echo "Failed to run example $example"
        exit 1
    fi
done

# if all examples ran successfully
echo "All examples ran successfully!"
exit 0
