/// File association and URL scheme default-handler commands.
use serde::Serialize;
use crate::errors::Result;

#[derive(Debug, Clone, Serialize)]
pub struct FileAssociationStatus {
    pub torrent_files: bool,
    pub magnet_links: bool,
}

#[tauri::command]
pub async fn check_file_associations() -> Result<FileAssociationStatus> {
    #[cfg(target_os = "macos")]
    {
        Ok(macos_associations::check())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(FileAssociationStatus {
            torrent_files: false,
            magnet_links: false,
        })
    }
}

#[tauri::command]
pub async fn set_default_for_torrents() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos_associations::set_torrent_default()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

#[tauri::command]
pub async fn set_default_for_magnets() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos_associations::set_magnet_default()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod macos_associations {
    use super::FileAssociationStatus;
    use crate::errors::{Result, WhenThenError};
    use std::os::raw::c_void;

    type CFTypeRef = *const c_void;
    type CFAllocatorRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFBundleRef = *const c_void;
    type CFIndex = isize;
    type Boolean = u8;

    type LSRolesMask = u32;
    const K_LS_ROLES_ALL: LSRolesMask = 0xFFFFFFFF;
    const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFAllocatorDefault: CFAllocatorRef;
        static kUTTagClassFilenameExtension: CFStringRef;

        fn CFBundleGetMainBundle() -> CFBundleRef;
        fn CFBundleGetIdentifier(bundle: CFBundleRef) -> CFStringRef;
        fn CFStringCreateWithBytes(
            alloc: CFAllocatorRef,
            bytes: *const u8,
            num_bytes: CFIndex,
            encoding: u32,
            is_external: Boolean,
        ) -> CFStringRef;
        fn CFStringGetCString(
            s: CFStringRef,
            buffer: *mut u8,
            buffer_size: CFIndex,
            encoding: u32,
        ) -> Boolean;
        fn CFStringGetLength(s: CFStringRef) -> CFIndex;
        fn CFStringCompare(
            a: CFStringRef,
            b: CFStringRef,
            flags: u64,
        ) -> i64;
        fn CFRelease(cf: CFTypeRef);
    }

    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn UTTypeCreatePreferredIdentifierForTag(
            tag_class: CFStringRef,
            tag: CFStringRef,
            conforming_to: CFTypeRef,
        ) -> CFStringRef;
        fn LSCopyDefaultRoleHandlerForContentType(
            content_type: CFStringRef,
            role: LSRolesMask,
        ) -> CFStringRef;
        fn LSSetDefaultRoleHandlerForContentType(
            content_type: CFStringRef,
            role: LSRolesMask,
            bundle_id: CFStringRef,
        ) -> i32;
        fn LSCopyDefaultHandlerForURLScheme(
            scheme: CFStringRef,
        ) -> CFStringRef;
        fn LSSetDefaultHandlerForURLScheme(
            scheme: CFStringRef,
            bundle_id: CFStringRef,
        ) -> i32;
    }

    // kCFCompareCaseInsensitive = 1
    const K_CF_COMPARE_CASE_INSENSITIVE: u64 = 1;

    unsafe fn cfstring_from_str(s: &str) -> CFStringRef {
        CFStringCreateWithBytes(
            kCFAllocatorDefault,
            s.as_ptr(),
            s.len() as CFIndex,
            K_CF_STRING_ENCODING_UTF8,
            0,
        )
    }

    unsafe fn cfstring_to_string(s: CFStringRef) -> Option<String> {
        if s.is_null() {
            return None;
        }
        let len = CFStringGetLength(s);
        // 4 bytes per char (UTF-8 worst case) + null terminator
        let buf_size = (len * 4 + 1) as usize;
        let mut buf = vec![0u8; buf_size];
        if CFStringGetCString(s, buf.as_mut_ptr(), buf_size as CFIndex, K_CF_STRING_ENCODING_UTF8) != 0 {
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            Some(String::from_utf8_lossy(&buf[..end]).into_owned())
        } else {
            None
        }
    }

    /// Get our own bundle identifier.
    unsafe fn own_bundle_id() -> Option<CFStringRef> {
        let bundle = CFBundleGetMainBundle();
        if bundle.is_null() {
            return None;
        }
        let id = CFBundleGetIdentifier(bundle);
        if id.is_null() { None } else { Some(id) }
    }

    /// Case-insensitive CFString equality check. Returns false if either is null.
    unsafe fn cfstrings_equal(a: CFStringRef, b: CFStringRef) -> bool {
        if a.is_null() || b.is_null() {
            return false;
        }
        CFStringCompare(a, b, K_CF_COMPARE_CASE_INSENSITIVE) == 0
    }

    pub fn check() -> FileAssociationStatus {
        unsafe {
            let bundle_id = match own_bundle_id() {
                Some(id) => id,
                None => return FileAssociationStatus { torrent_files: false, magnet_links: false },
            };

            // Check .torrent file association
            let ext = cfstring_from_str("torrent");
            let uti = UTTypeCreatePreferredIdentifierForTag(
                kUTTagClassFilenameExtension,
                ext,
                std::ptr::null(),
            );
            CFRelease(ext);

            let torrent_default = if !uti.is_null() {
                let current = LSCopyDefaultRoleHandlerForContentType(uti, K_LS_ROLES_ALL);
                let is_us = cfstrings_equal(current, bundle_id);
                if !current.is_null() { CFRelease(current); }
                CFRelease(uti);
                is_us
            } else {
                false
            };

            // Check magnet: URL scheme
            let scheme = cfstring_from_str("magnet");
            let current_magnet = LSCopyDefaultHandlerForURLScheme(scheme);
            let magnet_default = cfstrings_equal(current_magnet, bundle_id);
            if !current_magnet.is_null() { CFRelease(current_magnet); }
            CFRelease(scheme);

            FileAssociationStatus {
                torrent_files: torrent_default,
                magnet_links: magnet_default,
            }
        }
    }

    pub fn set_torrent_default() -> Result<()> {
        unsafe {
            let bundle_id = own_bundle_id().ok_or_else(|| {
                WhenThenError::Internal("Could not get bundle identifier".into())
            })?;

            let ext = cfstring_from_str("torrent");
            let uti = UTTypeCreatePreferredIdentifierForTag(
                kUTTagClassFilenameExtension,
                ext,
                std::ptr::null(),
            );
            CFRelease(ext);

            if uti.is_null() {
                return Err(WhenThenError::Internal("Could not resolve UTI for .torrent".into()));
            }

            let status = LSSetDefaultRoleHandlerForContentType(uti, K_LS_ROLES_ALL, bundle_id);
            CFRelease(uti);

            if status != 0 {
                return Err(WhenThenError::Internal(
                    format!("LSSetDefaultRoleHandlerForContentType failed ({})", status),
                ));
            }

            // Also log for debugging
            if let Some(id_str) = cfstring_to_string(bundle_id) {
                tracing::info!("Set default .torrent handler to {}", id_str);
            }

            Ok(())
        }
    }

    pub fn set_magnet_default() -> Result<()> {
        unsafe {
            let bundle_id = own_bundle_id().ok_or_else(|| {
                WhenThenError::Internal("Could not get bundle identifier".into())
            })?;

            let scheme = cfstring_from_str("magnet");
            let status = LSSetDefaultHandlerForURLScheme(scheme, bundle_id);
            CFRelease(scheme);

            if status != 0 {
                return Err(WhenThenError::Internal(
                    format!("LSSetDefaultHandlerForURLScheme failed ({})", status),
                ));
            }

            if let Some(id_str) = cfstring_to_string(bundle_id) {
                tracing::info!("Set default magnet: handler to {}", id_str);
            }

            Ok(())
        }
    }
}
