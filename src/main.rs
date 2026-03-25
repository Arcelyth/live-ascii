use live_ascii::ffi::*;

fn main() {
    let version = csm_get_version();
    println!("{}", version);
}

