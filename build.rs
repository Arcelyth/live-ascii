use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>{
    dotenvy::dotenv()?;
    let cubism_sdk_path = env::var("CubismSDKDir")?;
    let lib_name = env::var("LibName").unwrap_or("Live2DCubismCore".to_string());
    println!("cargo:rustc-link-search=native={}", cubism_sdk_path);
    println!("cargo:rustc-link-lib={}", lib_name);
    println!("cargo:rerun-if-changed=build.rs");
        
    Ok(())
}
