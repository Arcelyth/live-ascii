use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(error) = dotenvy::dotenv() {
        if !error.not_found() {
            return Err(error.into());
        }
    }
    let cubism_sdk_path = env::var("CubismSDKDir")?;
    let lib_name = env::var("LibName").unwrap_or("Live2DCubismCore".to_string());
    println!("cargo:rustc-link-search=native={}", cubism_sdk_path);
    println!("cargo:rustc-link-lib={}", lib_name);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=CubismSDKDir");
    println!("cargo:rerun-if-env-changed=LibName");

    Ok(())
}
