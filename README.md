A program to extract unique alignments with a specified mapping quality in a bam file.
Both primary and secondary alignments of a read will be removed,
if they have higher mapping quality (MapQ) than a specified threshold.

# How to install
Please set up a Rust environment, and then

- `cargo build --release` and find the executable at `target/release`
- or `cargo install --path .`
