use std::fs::File;
use std::ffi::c_void;
use std::io::Read;
use std::error::Error;
use std::ptr;

use live_ascii::ffi::*;
use live_ascii::utils::*;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: String,
}

fn main() -> Result<(), Box<dyn Error>> {

    let args = Args::parse();
    let moc_file = args.path;

    let version = csm_get_version();
    println!("{:x}", version);

    let mut file = File::open(moc_file)?;
    let mut moc_data = Vec::new();
    file.read_to_end(&mut moc_data)?;
    println!("{}", moc_data.len());
    let moc_ptr = unsafe {
        let mem = allocate_aligned(moc_data.len(), CSM_ALIGNOF_MOC);

        if mem.is_null() {
            panic!("Failed to allocate aligned.");
        }

        if (mem as usize) % 64 != 0 {
            panic!("Memory alignment exception: Address is not a multiple of 64.");
        }

        ptr::copy_nonoverlapping(moc_data.as_ptr(), mem, moc_data.len());

        let is_consistent = csmHasMocConsistency(mem as *mut c_void, moc_data.len() as u32); 
        let moc3_version = csmGetMocVersion(moc_data.as_ptr() as *const c_void, moc_data.len() as u32);
        println!("file's version: {}", moc3_version);
        if is_consistent == 0 {
            panic!("Moc data verification failed: The file is damaged or not a valid moc3 file.");
        }

        csmReviveMocInPlace(mem as *mut c_void, moc_data.len() as u32)
    };

    if moc_ptr.is_null() {
        panic!("Failed to load moc3");
    }
    let model_ptr = unsafe {
        let size = csmGetSizeofModel(moc_ptr);
        let mem = allocate_aligned(size as usize,  CSM_ALIGNOF_MODEL);
        csmInitializeModelInPlace(moc_ptr, mem as *mut c_void, size)
    };
    Ok(())
}

