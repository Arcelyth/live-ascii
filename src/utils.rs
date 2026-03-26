use std::ptr;

pub fn allocate_aligned(size: usize, alignment: usize) -> *mut u8 {
    let mut ptr: *mut libc::c_void = ptr::null_mut();
    unsafe {
        libc::posix_memalign(&mut ptr, alignment, size);
    }
    ptr as *mut u8
} 
