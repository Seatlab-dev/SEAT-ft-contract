//! Executed before the contract's compilation.

use common::version::build;

fn main() {
    // generates the version information and set the env vars
    // that will be present at compile-time on the contract's
    // compilation
    build::create_version().set_env();
    // makes it so this build.rs step always runs
    // build::setup_rerun();
    //
    // note: if the build.rs step panics, it may not trigger
    // it's automatic rerun, so you'd need to `touch build.rs`
    // to guarantee it's next rerun.
}
