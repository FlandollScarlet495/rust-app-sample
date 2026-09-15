use debug_util::debug_print;
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn plugin_hello(msg_ptr: *const c_char) {
    debug_print("[DEBUG] plugin_hello started processing...");

    let msg = if msg_ptr.is_null() {
        debug_print("[DEBUG] WARNING: Received null pointer in plugin_hello!");
        "[Null Pointer]".to_string()
    } else {
        unsafe { CStr::from_ptr(msg_ptr).to_string_lossy().into_owned() }
    };

    // プラグイン自体のメインの出力はそのまま
    println!("Hello from plugin_sample DLL! -> {}", msg);

    debug_print("[DEBUG] plugin_hello finished successfully.");
}
