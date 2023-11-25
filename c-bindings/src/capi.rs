use gst::ffi::GstBuffer;
use gst_video::ffi::GstVideoInfo;
use gst_video::VideoInfo;
use gst::glib::translate::FromGlibPtrNone;
use std::ffi::{c_char, c_uint, CStr};
use std::ptr;
use waylanddisplaycore::WaylandDisplay;

#[no_mangle]
pub extern "C" fn display_init(render_node: *const c_char) -> *mut WaylandDisplay {
    let render_node = if !render_node.is_null() {
        Some(
            unsafe { CStr::from_ptr(render_node) }
                .to_string_lossy()
                .into_owned(),
        )
    } else {
        None
    };

    match WaylandDisplay::new(render_node) {
        Ok(dpy) => Box::into_raw(Box::new(dpy)),
        Err(err) => {
            tracing::error!(?err, "Failed to create wayland display.");
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn display_finish(dpy: *mut WaylandDisplay) {
    std::mem::drop(unsafe { Box::from_raw(dpy) })
}

#[no_mangle]
pub extern "C" fn display_get_devices_len(dpy: *mut WaylandDisplay) -> c_uint {
    let display = unsafe { &mut *dpy };
    display.devices.get().len() as c_uint
}

#[no_mangle]
pub extern "C" fn display_get_devices(
    dpy: *mut WaylandDisplay,
    devices: *mut *const c_char,
    max_len: c_uint,
) -> c_uint {
    let display = unsafe { &mut *dpy };
    let client_devices = unsafe { std::slice::from_raw_parts_mut(devices, max_len as usize) };
    let devices = display.devices.get();

    for (i, string) in devices.iter().take(max_len as usize).enumerate() {
        client_devices[i] = string.as_ptr() as *const _;
    }

    std::cmp::max(max_len, devices.len() as c_uint)
}

#[no_mangle]
pub extern "C" fn display_get_envvars_len(dpy: *mut WaylandDisplay) -> c_uint {
    let display = unsafe { &mut *dpy };
    display.envs.get().len() as c_uint
}

#[no_mangle]
pub extern "C" fn display_get_envvars(
    dpy: *mut WaylandDisplay,
    env_vars: *mut *const c_char,
    max_len: c_uint,
) -> c_uint {
    let display = unsafe { &mut *dpy };
    let client_env_vars = unsafe { std::slice::from_raw_parts_mut(env_vars, max_len as usize) };
    let env_vars = display.envs.get();

    for (i, string) in env_vars.iter().take(max_len as usize).enumerate() {
        client_env_vars[i] = string.as_ptr() as *const _;
    }

    std::cmp::max(max_len, env_vars.len() as c_uint)
}

#[no_mangle]
pub extern "C" fn display_add_input_device(dpy: *mut WaylandDisplay, path: *const c_char) {
    let display = unsafe { &mut *dpy };
    let path = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();

    display.add_input_device(path);
}

#[no_mangle]
pub extern "C" fn display_set_video_info(dpy: *mut WaylandDisplay, info: *const GstVideoInfo) {
    let display = unsafe { &mut *dpy };
    if info.is_null() {
        tracing::error!("Video Info is null");
    }
    let video_info = unsafe { VideoInfo::from_glib_none(info) };

    display.set_video_info(video_info);
}

#[no_mangle]
pub extern "C" fn display_get_frame(dpy: *mut WaylandDisplay) -> *mut GstBuffer {
    let display = unsafe { &mut *dpy };
    match display.frame() {
        Ok(mut frame) => {
            let ptr = frame.make_mut().as_mut_ptr();
            std::mem::forget(frame);
            ptr
        }
        Err(err) => {
            tracing::error!("Rendering error: {}", err);
            ptr::null_mut()
        }
    }
}