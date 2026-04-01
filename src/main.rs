use std::error::Error;
use std::ffi::c_void;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::ptr;

use live_ascii::context::*;
use live_ascii::expression::exp::*;
use live_ascii::ffi::*;
use live_ascii::model::*;
use live_ascii::effect::pose::*;
use live_ascii::model_setting::ModelSetting;
use live_ascii::motion::manager::*;
use live_ascii::motion::player::*;
use live_ascii::motion::json::*;
use live_ascii::motion::amotion::*;

use live_ascii::renderer::*;
use live_ascii::utils::*;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    model_setting: String, // model3.json file
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let model_setting = ModelSetting::new(&args.model_setting)?;
    let model3_path = Path::new(&args.model_setting).canonicalize()?;
    let base_dir = model3_path.parent().unwrap();

    let file_refs = &model_setting.file_references;

    let mut moc_data = Vec::new();
    if let Some(moc_relative_path) = &file_refs.moc {
        let full_moc_path = base_dir.join(moc_relative_path);
        let mut file = File::open(&full_moc_path)?;
        file.read_to_end(&mut moc_data)?;
    } else {
        panic!("MOC path not found in JSON");
    }

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
    for relative_path in &file_refs.textures {
        let full_path = base_dir.join(relative_path);
        if full_path.is_file() {
            let texture = image::open(&full_path)?.to_rgba8();
            textures.push(image::DynamicImage::ImageRgba8(texture));
        }
    }


    // initialize terminal
    let mut context = Context::new(false);

    // initialize motionplayer
    // TODO: handle more motions
    let motions = &file_refs.motions;
    let idle_motions = motions.get("Idle");

    let mut mp = if let Some(m) = idle_motions {
        let full_motion_path = base_dir.join(&m[0].file);
        Some(MotionPlayer::new(full_motion_path.to_str().unwrap())?)
    } else {
        None
    };

    // initialize motion manager
    let mut mm = MotionManager::new();

    // initialize renderer
    let mut renderer = Renderer::new(model_ptr, textures);

    // initialize expression
    let expressions = &file_refs.expressions;
    let mut exp = if expressions.is_empty() {
        None
    } else {
        let full_exp_path = base_dir.join(&expressions[0].file);
        Some(ExpressionMotion::new(full_exp_path.to_str().unwrap())?)
    };
    let motion_file = model_setting.get_motion_file_name("Idle", 0).unwrap();
    let motion_data = MotionData::from_path(base_dir.to_str().unwrap(), motion_file)?;

    let mut idle_motion = CubismMotion::new(motion_data);

    let pose_file = model_setting.get_pose_file_name().unwrap();
    let mut pose = Pose::from_path(base_dir.to_str().unwrap(), pose_file.to_str().unwrap())?;

    renderer.render(
        &mut context,
        &mut mp,
        &mut mm,
        &model_setting,
        &mut exp,
        &mut idle_motion,
        &mut pose,
    )?;

    Ok(())
}
