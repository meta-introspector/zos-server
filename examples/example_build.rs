// Example build.rs using mkbuildrs! macro
use zos_server::mkbuildrs;

fn main() {
    // Single macro call patches entire Cargo project
    mkbuildrs!();
}
