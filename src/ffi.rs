//! FFI for Cubsim live2d's API

use std::ffi::{c_char, c_void};

/// # Types

/// Cubism moc.
#[repr(C)]
pub struct CsmMoc {
    _unused: [u8; 0],
}

/// Cubism model.
#[repr(C)]
pub struct CsmModel {
    _unused: [u8; 0],
}

/// Cubism version identifier.
pub type CsmVersion = u32;
/// moc3 version identifier.
pub type CsmMocVersion = u32;
/// Bitfield.
pub type CsmFlags = u8;
/// Parameter type.
pub type CsmParameterType = i32;

/// # Alignment constraints.
/// Necessary alignment for mocs (in bytes).
pub const CSM_ALIGNOF_MOC: usize = 64;
/// Necessary alignment for models (in bytes).
pub const CSM_ALIGNOF_MODEL: usize = 16;

/// # moc3 file format version.
/// unknown
pub const CSM_MOC_VERSION_UNKNOWN: CsmMocVersion = 0;
/// moc3 file version 3.0.00 - 3.2.07
pub const CSM_MOC_VERSION_30: CsmMocVersion = 1;
/// moc3 file version 3.3.00 - 3.3.03
pub const CSM_MOC_VERSION_33: CsmMocVersion = 2;
/// moc3 file version 4.0.00 - 4.1.05
pub const CSM_MOC_VERSION_40: CsmMocVersion = 3;
/// moc3 file version 4.2.00 - 4.2.04
pub const CSM_MOC_VERSION_42: CsmMocVersion = 4;
/// moc3 file version 5.0.00 -
pub const CSM_MOC_VERSION_50: CsmMocVersion = 5;

/// # Parameter types.
/// Normal parameter.
pub const CSM_PARAMETER_TYPE_NORMAL: CsmParameterType = 0;
/// Parameter for blend shape.
pub const CSM_PARAMETER_TYPE_BLEND_SHAPE: CsmParameterType = 1;

/// # Bit masks for non-dynamic drawable flags.
/// Additive blend mode mask.
pub const CSM_BLEND_ADDITIVE: u8 = 1 << 0;
/// Multiplicative blend mode mask.
pub const CSM_BLEND_MULTIPLICATIVE: u8 = 1 << 1;
/// Double-sidedness mask.
pub const CSM_IS_DOUBLE_SIDED: u8 = 1 << 2;
/// Clipping mask inversion mode mask.
pub const CSM_IS_INVERTED_MASK: u8 = 1 << 3;

/// # Bit masks for dynamic drawable flags.
/// Flag set when visible.
pub const CSM_IS_VISIBLE: u8 = 1 << 0;
/// Flag set when visibility did change.
pub const CSM_VISIBILITY_DID_CHANGE: u8 = 1 << 1;
/// Flag set when opacity did change.
pub const CSM_OPACITY_DID_CHANGE: u8 = 1 << 2;
/// Flag set when draw order did change.
pub const CSM_DRAW_ORDER_DID_CHANGE: u8 = 1 << 3;
/// Flag set when render order did change.
pub const CSM_RENDER_ORDER_DID_CHANGE: u8 = 1 << 4;
/// Flag set when vertex positions did change.
pub const CSM_VERTEX_POSITIONS_DID_CHANGE: u8 = 1 << 5;
/// Flag set when blend color did change.
pub const CSM_BLEND_COLOR_DID_CHANGE: u8 = 1 << 6;

/// 2 component vector.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CsmVector2 {
    pub x: f32,
    pub y: f32,
}

/// 4 component vector.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CsmVector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Log handler.
/// # Arguments
/// * `message` - Null-terminated string message to log.
pub type CsmLogFunction = Option<unsafe extern "C" fn(message: *const c_char)>;


unsafe extern "C" {
    /* ------- *
     * VERSION *
     * ------- */

    /// Queries Core version.
    ///
    /// # Returns
    /// Core version.
    pub fn csmGetVersion() -> CsmVersion;

    /// Gets Moc file supported latest version.
    ///
    /// # Returns
    /// csmMocVersion (Moc file latest format version).
    pub fn csmGetLatestMocVersion() -> CsmMocVersion;

    /// Gets Moc file format version.
    ///
    /// # Arguments
    /// * `address` - Address of moc.
    /// * `size` - Size of moc (in bytes).
    ///
    /// # Returns
    /// csmMocVersion
    pub fn csmGetMocVersion(address: *const c_void, size: u32) -> CsmMocVersion;

    /* ----------- *
     * CONSISTENCY *
     * ----------- */

    /// Checks consistency of a moc.
    ///
    /// # Arguments
    /// * `address` - Address of unrevived moc. The address must be aligned to 'csmAlignofMoc'.
    /// * `size` - Size of moc (in bytes).
    ///
    /// # Returns
    /// '1' if Moc is valid; '0' otherwise.
    pub fn csmHasMocConsistency(address: *mut c_void, size: u32) -> i32;

    /* ------- *
     * LOGGING *
     * ------- */

    /// Queries log handler.
    ///
    /// # Returns
    /// Log handler.
    pub fn csmGetLogFunction() -> CsmLogFunction;

    /// Sets log handler.
    ///
    /// # Arguments
    /// * `handler` - Handler to use.
    pub fn csmSetLogFunction(handler: CsmLogFunction);

    /* --- *
     * MOC *
     * --- */

    /// Tries to revive a moc from bytes in place.
    ///
    /// # Arguments
    /// * `address` - Address of unrevived moc. The address must be aligned to 'csmAlignofMoc'.
    /// * `size` - Size of moc (in bytes).
    ///
    /// # Returns
    /// Valid pointer on success; '0' otherwise.
    pub fn csmReviveMocInPlace(address: *mut c_void, size: u32) -> *mut CsmMoc;

    /* ----- *
     * MODEL *
     * ----- */

    /// Queries size of a model in bytes.
    ///
    /// # Arguments
    /// * `moc` - Moc to query.
    ///
    /// # Returns
    /// Valid size on success; '0' otherwise.
    pub fn csmGetSizeofModel(moc: *const CsmMoc) -> u32;

    /// Tries to instantiate a model in place.
    ///
    /// # Arguments
    /// * `moc` - Source moc.
    /// * `address` - Address to place instance at. Address must be aligned to 'csmAlignofModel'.
    /// * `size` - Size of memory block for instance (in bytes).
    ///
    /// # Returns
    /// Valid pointer on success; '0' otherwise.
    pub fn csmInitializeModelInPlace(
        moc: *const CsmMoc,
        address: *mut c_void,
        size: u32,
    ) -> *mut CsmModel;

    /// Updates a model.
    ///
    /// # Arguments
    /// * `model` - Model to update.
    pub fn csmUpdateModel(model: *mut CsmModel);

    /* ------ *
     * CANVAS *
     * ------ */

    /// Reads info on a model canvas.
    ///
    /// # Arguments
    /// * `model` - Model query.
    /// * `outSizeInPixels` - Canvas dimensions.
    /// * `outOriginInPixels` - Origin of model on canvas.
    /// * `outPixelsPerUnit` - Aspect used for scaling pixels to units.
    pub fn csmReadCanvasInfo(
        model: *const CsmModel,
        outSizeInPixels: *mut CsmVector2,
        outOriginInPixels: *mut CsmVector2,
        outPixelsPerUnit: *mut f32,
    );

    /* ---------- *
     * PARAMETERS *
     * ---------- */

    /// Gets number of parameters.
    ///
    /// # Arguments
    /// * `model` - Model to query.
    ///
    /// # Returns
    /// Valid count on success; '-1' otherwise.
    pub fn csmGetParameterCount(model: *const CsmModel) -> i32;

    /// Gets parameter IDs.
    /// All IDs are null-terminated ANSI strings.
    ///
    /// # Returns
    /// Valid pointer on success; '0' otherwise.
    pub fn csmGetParameterIds(model: *const CsmModel) -> *const *const c_char;

    /// Gets parameter types.
    ///
    /// # Returns
    /// Valid pointer on success; '0' otherwise.
    pub fn csmGetParameterTypes(model: *const CsmModel) -> *const CsmParameterType;

    /// Gets minimum parameter values.
    pub fn csmGetParameterMinimumValues(model: *const CsmModel) -> *const f32;

    /// Gets maximum parameter values.
    pub fn csmGetParameterMaximumValues(model: *const CsmModel) -> *const f32;

    /// Gets default parameter values.
    pub fn csmGetParameterDefaultValues(model: *const CsmModel) -> *const f32;

    /// Gets read/write parameter values buffer.
    pub fn csmGetParameterValues(model: *mut CsmModel) -> *mut f32;

    /// Gets Parameter Repeat informations.
    pub fn csmGetParameterRepeats(model: *const CsmModel) -> *const i32;

    /// Gets number of key values of each parameter.
    pub fn csmGetParameterKeyCounts(model: *const CsmModel) -> *const i32;

    /// Gets key values of each parameter.
    pub fn csmGetParameterKeyValues(model: *const CsmModel) -> *const *const f32;

    /* ----- *
     * PARTS *
     * ----- */

    /// Gets number of parts.
    pub fn csmGetPartCount(model: *const CsmModel) -> i32;

    /// Gets parts IDs.
    /// All IDs are null-terminated ANSI strings.
    pub fn csmGetPartIds(model: *const CsmModel) -> *const *const c_char;

    /// Gets read/write part opacities buffer.
    pub fn csmGetPartOpacities(model: *mut CsmModel) -> *mut f32;

    /// Gets part's parent part indices.
    pub fn csmGetPartParentPartIndices(model: *const CsmModel) -> *const i32;

    /* --------- *
     * DRAWABLES *
     * --------- */

    /// Gets number of drawables.
    pub fn csmGetDrawableCount(model: *const CsmModel) -> i32;

    /// Gets drawable IDs.
    pub fn csmGetDrawableIds(model: *const CsmModel) -> *const *const c_char;

    /// Gets constant drawable flags.
    pub fn csmGetDrawableConstantFlags(model: *const CsmModel) -> *const CsmFlags;

    /// Gets dynamic drawable flags.
    pub fn csmGetDrawableDynamicFlags(model: *const CsmModel) -> *const CsmFlags;

    /// Gets drawable texture indices.
    pub fn csmGetDrawableTextureIndices(model: *const CsmModel) -> *const i32;

    /// Gets drawable draw orders.
    pub fn csmGetDrawableDrawOrders(model: *const CsmModel) -> *const i32;

    /// Gets drawable render orders.
    /// The higher the order, the more up front a drawable is.
    pub fn csmGetDrawableRenderOrders(model: *const CsmModel) -> *const i32;

    /// Gets drawable opacities.
    pub fn csmGetDrawableOpacities(model: *const CsmModel) -> *const f32;

    /// Gets numbers of masks of each drawable.
    pub fn csmGetDrawableMaskCounts(model: *const CsmModel) -> *const i32;

    /// Gets mask indices of each drawable.
    pub fn csmGetDrawableMasks(model: *const CsmModel) -> *const *const i32;

    /// Gets number of vertices of each drawable.
    pub fn csmGetDrawableVertexCounts(model: *const CsmModel) -> *const i32;

    /// Gets vertex position data of each drawable.
    pub fn csmGetDrawableVertexPositions(model: *const CsmModel) -> *const *const CsmVector2;

    /// Gets texture coordinate data of each drawables.
    pub fn csmGetDrawableVertexUvs(model: *const CsmModel) -> *const *const CsmVector2;

    /// Gets number of triangle indices for each drawable.
    pub fn csmGetDrawableIndexCounts(model: *const CsmModel) -> *const i32;

    /// Gets triangle index data for each drawable.
    pub fn csmGetDrawableIndices(model: *const CsmModel) -> *const *const u16;

    /// Gets multiply color data for each drawable.
    pub fn csmGetDrawableMultiplyColors(model: *const CsmModel) -> *const CsmVector4;

    /// Gets screen color data for each drawable.
    pub fn csmGetDrawableScreenColors(model: *const CsmModel) -> *const CsmVector4;

    /// Gets drawable's parent part indices.
    pub fn csmGetDrawableParentPartIndices(model: *const CsmModel) -> *const i32;

    /// Resets all dynamic drawable flags.
    pub fn csmResetDrawableDynamicFlags(model: *mut CsmModel);
}

pub fn csm_get_version() -> u32 {
    unsafe { csmGetVersion() }
}

