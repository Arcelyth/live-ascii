use std::path::PathBuf;
use std::ptr;

pub fn allocate_aligned(size: usize, alignment: usize) -> *mut u8 {
    let mut ptr: *mut libc::c_void = ptr::null_mut();
    unsafe {
        libc::posix_memalign(&mut ptr, alignment, size);
    }
    ptr as *mut u8
}

pub fn sort_path(files: &mut Vec<PathBuf>) {
    files.sort_by_key(|path| path.file_name().unwrap().to_string_lossy().to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sort_path_test() {
        let mut files = vec![
            PathBuf::from("texture_09.png"),
            PathBuf::from("texture_03.png"),
            PathBuf::from("texture_06.png"),
            PathBuf::from("texture_08.png"),
            PathBuf::from("texture_01.png"),
            PathBuf::from("texture_07.png"),
            PathBuf::from("texture_11.png"),
            PathBuf::from("texture_02.png"),
            PathBuf::from("texture_10.png"),
            PathBuf::from("texture_00.png"),
            PathBuf::from("texture_04.png"),
            PathBuf::from("texture_05.png"),
        ];
        sort_path(&mut files);
        let expected = vec![
            PathBuf::from("texture_00.png"),
            PathBuf::from("texture_01.png"),
            PathBuf::from("texture_02.png"),
            PathBuf::from("texture_03.png"),
            PathBuf::from("texture_04.png"),
            PathBuf::from("texture_05.png"),
            PathBuf::from("texture_06.png"),
            PathBuf::from("texture_07.png"),
            PathBuf::from("texture_08.png"),
            PathBuf::from("texture_09.png"),
            PathBuf::from("texture_10.png"),
            PathBuf::from("texture_11.png"),
        ];

        assert_eq!(files, expected);
    }
}
