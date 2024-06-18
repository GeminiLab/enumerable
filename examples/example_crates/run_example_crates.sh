# 1. collect all subdirectories in the directory.
# 2. check if the directory contains a Cargo.toml file.
# 3. if it does, run it with the command `cargo run`.
# 4. if it doesn't, skip it.
# 5. check if it returns 0.

# get the path of this script
SCRIPT_PATH=$(dirname "$(realpath "$0")")

# get all subdirectories in the directory
SUBDIRS=$(find "$SCRIPT_PATH" -mindepth 1 -maxdepth 1 -type d)

# for each subdirectory
for SUBDIR in $SUBDIRS
do
    # check if the directory contains a Cargo.toml file
    if [ -f "$SUBDIR/Cargo.toml" ]; then
        echo "Running $SUBDIR"
        # run it with the command `cargo run`
        cargo run --manifest-path "$SUBDIR/Cargo.toml"
        # check if it returns 0
        if [ $? -ne 0 ]; then
            echo "Failed to run example crate $SUBDIR"
            exit 1
        fi
    fi
done

# if all crates run successfully
echo "All example crates ran successfully"
exit 0
