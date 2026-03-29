use std::error::Error;
use std::ffi::c_void;
use std::fs::{self, File};
use std::io::{Read};
use std::ptr;

use live_ascii::context::*;
use live_ascii::ffi::*;
use live_ascii::renderer::*;
use live_ascii::utils::*;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    path: String,    // moc3 file path
    texture: String, // png file path
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut file = File::open(&args.path)?;
    let mut moc_data = Vec::new();
    file.read_to_end(&mut moc_data)?;

    // loading moc3
    let moc_mem = unsafe {
        let mem = allocate_aligned(moc_data.len(), CSM_ALIGNOF_MOC);
        ptr::copy_nonoverlapping(moc_data.as_ptr(), mem, moc_data.len());

        // check moc3 consistency
        let consistency = csmHasMocConsistency(mem as *mut c_void, moc_data.len() as u32);

        if consistency == 0 {
            panic!("The moc3 file is malformed.");
        }

        csmReviveMocInPlace(mem as *mut c_void, moc_data.len() as u32)
    };

    // create a model from moc3
    let model_ptr = unsafe {
        let size = csmGetSizeofModel(moc_mem);
        let mem = allocate_aligned(size as usize, CSM_ALIGNOF_MODEL);
        csmInitializeModelInPlace(moc_mem, mem as *mut c_void, size)
    };

    // load texture
    let mut textures = vec![];
    for entry in fs::read_dir(args.texture)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png") {
            let texture = image::open(path)?.to_rgba8();
            textures.push(image::DynamicImage::ImageRgba8(texture));
        }
    }

    let mut renderer = Renderer::new(model_ptr, textures);

    // initialize terminal
    let mut context = Context::new(false);

    renderer.render(&mut context)?;
    
    Ok(())
}


