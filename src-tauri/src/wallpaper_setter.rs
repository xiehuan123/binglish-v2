use std::path::Path;

#[cfg(target_os = "windows")]
pub fn set_wallpaper(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    };

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide.as_ptr() as *mut _),
            SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
        )
        .map_err(|e| format!("Failed to set wallpaper: {e}"))?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_wallpaper(path: &Path) -> Result<(), String> {
    use std::ffi::c_void;

    let stem = path.file_stem().unwrap().to_string_lossy();
    let path_a = path.with_file_name(format!("{stem}_display_a.jpg"));
    let path_b = path.with_file_name(format!("{stem}_display_b.jpg"));
    let (cache_path, old_path) = if path_a.exists() { (path_b, path_a) } else { (path_a, path_b) };
    let _ = std::fs::remove_file(&old_path);
    std::fs::copy(path, &cache_path).map_err(|e| format!("Copy failed: {e}"))?;

    let path_str = cache_path.to_str().ok_or("Invalid path encoding")?.to_string();

    extern "C" {
        static _dispatch_main_q: c_void;
        fn dispatch_sync_f(queue: *const c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
    }

    struct Ctx {
        path: String,
        result: Result<(), String>,
    }

    extern "C" fn do_set(raw: *mut c_void) {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};
        use std::ffi::CString;

        let ctx = unsafe { &mut *(raw as *mut Ctx) };
        let c_path = match CString::new(ctx.path.as_str()) {
            Ok(p) => p,
            Err(e) => { ctx.result = Err(format!("Invalid path: {e}")); return; }
        };

        unsafe {
            let workspace: *mut Object =
                msg_send![Class::get("NSWorkspace").unwrap(), sharedWorkspace];
            let path_ns: *mut Object =
                msg_send![Class::get("NSString").unwrap(), stringWithUTF8String: c_path.as_ptr()];
            let url: *mut Object =
                msg_send![Class::get("NSURL").unwrap(), fileURLWithPath: path_ns];
            let screens: *mut Object =
                msg_send![Class::get("NSScreen").unwrap(), screens];
            let count: usize = msg_send![screens, count];
            let options: *mut Object =
                msg_send![Class::get("NSDictionary").unwrap(), dictionary];

            for i in 0..count {
                let screen: *mut Object = msg_send![screens, objectAtIndex: i];
                let mut error: *mut Object = std::ptr::null_mut();
                let ok: bool = msg_send![
                    workspace,
                    setDesktopImageURL: url
                    forScreen: screen
                    options: options
                    error: &mut error
                ];
                if !ok {
                    if !error.is_null() {
                        let desc: *mut Object = msg_send![error, localizedDescription];
                        let cstr: *const std::ffi::c_char = msg_send![desc, UTF8String];
                        if !cstr.is_null() {
                            let msg = std::ffi::CStr::from_ptr(cstr).to_string_lossy();
                            ctx.result = Err(format!("Set wallpaper failed: {msg}"));
                            return;
                        }
                    }
                    ctx.result = Err("Set wallpaper failed".to_string());
                    return;
                }
            }
        }
        ctx.result = Ok(());
    }

    let mut ctx = Ctx { path: path_str, result: Ok(()) };
    unsafe {
        dispatch_sync_f(
            &_dispatch_main_q as *const c_void,
            &mut ctx as *mut Ctx as *mut c_void,
            do_set,
        );
    }
    ctx.result
}
