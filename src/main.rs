use std::error::Error;
use std::ffi::c_void;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::ptr;

use live_ascii::context::*;
use live_ascii::effect::pose::*;
use live_ascii::expression::manager::*;
use live_ascii::ffi::*;
use live_ascii::live::json::*;
use live_ascii::model_setting::ModelSetting;
use live_ascii::motion::manager::*;
use live_ascii::physics::{Physics, json::*};
use live_ascii::shader::*;
use live_ascii::tracker::*;

use live_ascii::renderer::*;
use live_ascii::utils::*;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    model_setting: String, // model3.json file
    #[arg(short, long)]
    camera: bool,
    #[arg(short, long)]
    text_shader: Option<String>, // file path
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // load model setting
    let mut model_setting = ModelSetting::new(&args.model_setting)?;
    let model3_path = Path::new(&args.model_setting).canonicalize()?;
    let base_dir = model3_path
        .parent()
        .ok_or("Model path has no parent directory")?;

    let file_refs = &model_setting.file_references;

    // get model name
    let name;

    let mut moc_data = Vec::new();
    if let Some(moc_relative_path) = &file_refs.moc {
        let full_moc_path = base_dir.join(moc_relative_path);
        let mut file = File::open(&full_moc_path)?;
        name = get_file_name(moc_relative_path.to_str().ok_or("MOC path is not UTF-8")?);
        file.read_to_end(&mut moc_data)?;
    } else {
        return Err("MOC path not found in JSON".into());
    }

    // loading moc3
    let moc_mem = unsafe {
        if moc_data.is_empty() || moc_data.len() > u32::MAX as usize {
            return Err("MOC file has an invalid size".into());
        }
        let mem = allocate_aligned(moc_data.len(), CSM_ALIGNOF_MOC);
        if mem.is_null() {
            return Err("Failed to allocate MOC memory".into());
        }
        ptr::copy_nonoverlapping(moc_data.as_ptr(), mem, moc_data.len());

        // check moc3 consistency
        let consistency = csmHasMocConsistency(mem as *mut c_void, moc_data.len() as u32);

        if consistency == 0 {
            libc::free(mem.cast());
            return Err("The moc3 file is malformed".into());
        }

        csmReviveMocInPlace(mem as *mut c_void, moc_data.len() as u32)
    };

    if moc_mem.is_null() {
        return Err("Failed to revive MOC file".into());
    }

    // create a model from moc3
    let model_ptr = unsafe {
        let size = csmGetSizeofModel(moc_mem);
        let mem = allocate_aligned(size as usize, CSM_ALIGNOF_MODEL);
        if mem.is_null() {
            return Err("Failed to allocate model memory".into());
        }
        csmInitializeModelInPlace(moc_mem, mem as *mut c_void, size)
    };

    if model_ptr.is_null() {
        return Err("Failed to initialize Cubism model".into());
    }

    // load texture
    let mut textures = vec![];
    for relative_path in &file_refs.textures {
        let full_path = base_dir.join(relative_path);
        let texture = image::open(&full_path)?.to_rgba8();
        textures.push(image::DynamicImage::ImageRgba8(texture));
    }
    // initalize tracker
    let tracker = Tracker::new();

    // initialize terminal
    let mut context = Context::new(
        false,
        model_setting.clone(),
        base_dir.to_str().ok_or("Model directory is not UTF-8")?,
        args.camera,
        tracker,
    );

    let mut shader_manager = ShaderManager::new();
    if let Some(path) = &args.text_shader {
        let content = fs::read_to_string(path)?.replace([' ', '\t', '\n', '\r'], "");
        shader_manager.insert_hd(Shader::Text(content.into()));
    }

    // load live json
    let full_path = Path::new(base_dir).join(&format!("{}.live.json", name));
    if let Ok(data) = fs::read_to_string(&full_path) {
        if let Ok(live) = Live::from_data(data) {
            context.set_live_setting(live);
        } else {
            return Err(format!("Invalid live settings: {:?}", full_path).into());
        }
    }

    // initialize motion manager
    let mut mm = MotionManager::new();

    // initialize renderer
    let mut renderer = unsafe { Renderer::new(model_ptr, textures, shader_manager) };

    // initialize expression
    let mut em = ExpressionManager::new();

    let mut pos = if let Some(pose_file) = model_setting.get_pose_file_name() {
        Some(Pose::from_path(
            base_dir.to_str().ok_or("Model directory is not UTF-8")?,
            pose_file.to_str().ok_or("Pose path is not UTF-8")?,
        )?)
    } else {
        None
    };

    // physics

    let mut ph_file = if let Some(physics_file) = model_setting.get_physics_file_name() {
        let p_json = PhysicsJson::from_path(
            base_dir.to_str().ok_or("Model directory is not UTF-8")?,
            physics_file.to_str().ok_or("Physics path is not UTF-8")?,
        )?;
        Some(Physics::from_json(p_json)?)
    } else {
        None
    };

    renderer.render(
        &mut context,
        &mut mm,
        &mut model_setting,
        &mut em,
        &mut pos,
        &mut ph_file,
    )?;

    Ok(())
}
