//! Durable project directory identity. Evidence files keep a separate
//! short-lived `dev:ino` race check.
use anyhow::{Context, Result, ensure};
use std::{fs, path::Path, time::UNIX_EPOCH};

pub(super) fn current(path: &Path) -> Result<String> {
    crate::util::reject_symlink_components(path)?;
    let metadata = fs::metadata(path).context("inspect project directory identity")?;
    ensure!(metadata.is_dir(), "project root is not a directory");
    let birth = metadata
        .created()
        .context("filesystem does not provide project creation identity")?
        .duration_since(UNIX_EPOCH)
        .context("invalid project creation time")?
        .as_nanos();
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::MetadataExt;
        let uuid = macos_volume_uuid(path)?;
        let after = fs::metadata(path)?;
        ensure!(
            metadata.dev() == after.dev()
                && metadata.ino() == after.ino()
                && metadata.created()? == after.created()?,
            "project directory changed during identity inspection"
        );
        Ok(format!("v2:mac:{uuid}:{}:{birth}", metadata.ino()))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        use std::os::unix::fs::MetadataExt;
        let after = fs::metadata(path)?;
        ensure!(
            metadata.dev() == after.dev()
                && metadata.ino() == after.ino()
                && metadata.created()? == after.created()?,
            "project directory changed during identity inspection"
        );
        Ok(format!(
            "v2:unix:{}:{}:{birth}",
            metadata.dev(),
            metadata.ino()
        ))
    }
    #[cfg(windows)]
    {
        let (volume, file_id, handle_birth) = windows_identity(path)?;
        let after = windows_identity(path)?;
        ensure!(
            birth == handle_birth
                && after == (volume.clone(), file_id, handle_birth)
                && metadata.created()? == fs::metadata(path)?.created()?,
            "project directory changed during identity inspection"
        );
        Ok(format!("v2:win:{volume}:{file_id}:{birth}"))
    }
    #[cfg(all(not(unix), not(windows)))]
    {
        Ok(format!("v2:creation:{birth}"))
    }
}

#[cfg(windows)]
fn windows_identity(path: &Path) -> Result<(String, String, u128)> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt};
    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[repr(C)]
    struct FileInfo {
        attributes: u32,
        created: FileTime,
        _accessed: FileTime,
        _modified: FileTime,
        _volume_serial: u32,
        _size_high: u32,
        _size_low: u32,
        _links: u32,
        _file_index_high: u32,
        _file_index_low: u32,
    }
    #[repr(C)]
    struct FileIdInfo {
        volume_serial: u64,
        file_id: [u8; 16],
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: *const c_void,
            disposition: u32,
            flags: u32,
            template: *mut c_void,
        ) -> *mut c_void;
        fn GetFileInformationByHandle(handle: *mut c_void, info: *mut FileInfo) -> i32;
        fn GetFileInformationByHandleEx(
            handle: *mut c_void,
            class: i32,
            info: *mut c_void,
            size: u32,
        ) -> i32;
        fn GetFinalPathNameByHandleW(
            handle: *mut c_void,
            buffer: *mut u16,
            length: u32,
            flags: u32,
        ) -> u32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    let mut name: Vec<u16> = path.as_os_str().encode_wide().collect();
    name.push(0);
    // Open the directory itself, including a reparse-point check. Do not follow
    // a leaf alias while deciding which durable namespace it owns.
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            0x80, // FILE_READ_ATTRIBUTES
            7,    // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
            std::ptr::null(),
            3,                         // OPEN_EXISTING
            0x0200_0000 | 0x0020_0000, // BACKUP_SEMANTICS | OPEN_REPARSE_POINT
            std::ptr::null_mut(),
        )
    };
    ensure!(
        handle as isize != -1,
        "open project directory identity: {}",
        std::io::Error::last_os_error()
    );
    let result = (|| -> Result<(String, String, u128)> {
        let mut info = std::mem::MaybeUninit::<FileInfo>::uninit();
        ensure!(
            unsafe { GetFileInformationByHandle(handle, info.as_mut_ptr()) } != 0,
            "read project file ID: {}",
            std::io::Error::last_os_error()
        );
        let info = unsafe { info.assume_init() };
        ensure!(
            info.attributes & 0x400 == 0,
            "project root is a reparse point"
        );
        let mut extended = std::mem::MaybeUninit::<FileIdInfo>::uninit();
        ensure!(
            unsafe {
                GetFileInformationByHandleEx(
                    handle,
                    0x12, // FileIdInfo: 128-bit ID, including ReFS.
                    extended.as_mut_ptr().cast(),
                    std::mem::size_of::<FileIdInfo>() as u32,
                )
            } != 0,
            "read persistent project file ID: {}",
            std::io::Error::last_os_error()
        );
        let extended = unsafe { extended.assume_init() };
        ensure!(
            extended.file_id.iter().any(|byte| *byte != 0),
            "filesystem did not provide a stable file ID"
        );
        let file_id = format!(
            "{:016x}-{}",
            extended.volume_serial,
            extended
                .file_id
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        let ticks = (u64::from(info.created.high) << 32) | u64::from(info.created.low);
        ensure!(
            ticks >= 116_444_736_000_000_000,
            "invalid project creation time"
        );
        let birth = u128::from(ticks - 116_444_736_000_000_000) * 100;
        let mut volume_path = [0_u16; 4096];
        let len = unsafe { GetFinalPathNameByHandleW(handle, volume_path.as_mut_ptr(), 4096, 1) };
        ensure!(
            len > 0 && len < volume_path.len() as u32,
            "read persistent volume identity: {}",
            std::io::Error::last_os_error()
        );
        let path = String::from_utf16(&volume_path[..len as usize])?;
        let volume = path
            .strip_prefix(r"\\?\Volume{")
            .and_then(|value| value.split_once('}'))
            .map(|(id, _)| id.to_ascii_lowercase())
            .filter(|id| id.len() == 36 && id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-'))
            .context("filesystem did not provide a persistent volume GUID")?;
        Ok((volume, file_id, birth))
    })();
    let _ = unsafe { CloseHandle(handle) };
    result
}

pub(super) fn matches(path: &Path, saved: &str) -> Result<bool> {
    Ok(saved.starts_with("v2:") && current(path)? == saved)
}

#[cfg(target_os = "macos")]
fn macos_volume_uuid(path: &Path) -> Result<String> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    #[repr(C)]
    struct AttrList {
        bitmap_count: u16,
        reserved: u16,
        common: u32,
        volume: u32,
        directory: u32,
        file: u32,
        fork: u32,
    }
    unsafe extern "C" {
        fn getattrlist(
            path: *const std::ffi::c_char,
            attributes: *const AttrList,
            buffer: *mut std::ffi::c_void,
            buffer_size: usize,
            options: std::ffi::c_ulong,
        ) -> std::ffi::c_int;
    }
    let name = CString::new(path.as_os_str().as_bytes())?;
    // sys/attr.h: ATTR_BIT_MAP_COUNT=5, ATTR_VOL_UUID=0x00040000,
    // FSOPT_NOFOLLOW=1. The returned fixed record is length + UUID bytes.
    let attributes = AttrList {
        bitmap_count: 5,
        reserved: 0,
        common: 0,
        volume: 0x0004_0000,
        directory: 0,
        file: 0,
        fork: 0,
    };
    let mut bytes = [0_u8; 20];
    // SAFETY: name is NUL-terminated; attributes and buffer have the C ABI
    // layout and remain alive for the call. The kernel receives the exact size.
    let code = unsafe {
        getattrlist(
            name.as_ptr(),
            &attributes,
            bytes.as_mut_ptr().cast(),
            bytes.len(),
            1,
        )
    };
    ensure!(
        code == 0,
        "read persistent volume UUID: {}",
        std::io::Error::last_os_error()
    );
    ensure!(
        u32::from_ne_bytes(bytes[..4].try_into()?) == 20 && bytes[4..].iter().any(|b| *b != 0),
        "filesystem did not return a persistent volume UUID"
    );
    Ok(bytes[4..].iter().map(|b| format!("{b:02x}")).collect())
}
