# This script is used to run all the tests in the Nova language.
cargo build --release
nova="./target/release/nova"

# Run the tests
$nova run demo/demo.nv
$nova run demo/let.nv
$nova run demo/loops.nv
$nova run Aoc2024/nvaoc1.nv
$nova run Aoc2024/nvaoc2.nv
$nova run Aoc2024/nvaoc3.nv
$nova run Aoc2024/nvaoc4.nv

# Run the standard library tests
$nova run std/core.nv
$nova run std/iter.nv
$nova run std/list.nv
$nova run std/string.nv
$nova run std/hashmap.nv
$nova run std/io.nv
$nova run std/tui.nv
$nova run std/tuple.nv
