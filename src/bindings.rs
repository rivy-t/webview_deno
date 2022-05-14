use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;
use webview_official_sys::webview_t;
use webview_official_sys::DispatchFn;

static CHANNEL: Lazy<(Sender<(String, String)>, Receiver<(String, String)>)> = Lazy::new(|| unbounded());

macro_rules! export {
    ($rename: ident, fn $name:ident($( $arg:ident : $type:ty ),*) -> $ret:ty) => {
        #[no_mangle]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "This function is unsafe for obvious reasons."]
        pub unsafe extern "C" fn $rename($( $arg : $type),*) -> $ret {
            webview_official_sys::$name($( $arg ),*)
        }
    };
    ($rename: ident, fn $name:ident($( $arg:ident : $type:ty ),*)) => {
        export!($rename, fn $name($( $arg : $type),*) -> ());
    }
}

// https://github.com/rust-lang/rfcs/issues/2771
export!(deno_webview_create, fn webview_create(debug: c_int, window: *mut c_void) -> webview_t);
export!(deno_webview_destroy, fn webview_destroy(w: webview_t));
export!(deno_webview_step, fn webview_step(w: webview_t, blocking: i32) -> i32);
export!(deno_webview_run, fn webview_run(w: webview_t));
export!(deno_webview_terminate, fn webview_terminate(w: webview_t));
export!(deno_webview_dispatch, fn webview_dispatch(w: webview_t, fn_: Option<DispatchFn>, arg: *mut c_void));
export!(deno_webview_get_window, fn webview_get_window(w: webview_t) -> *mut c_void);
export!(deno_webview_set_title, fn webview_set_title(w: webview_t, title: *const c_char));
export!(deno_webview_set_size, fn webview_set_size(w: webview_t, width: c_int, height: c_int, hints: c_int));
export!(deno_webview_navigate, fn webview_navigate(w: webview_t, url: *const c_char));
export!(deno_webview_init, fn webview_init(w: webview_t, js: *const c_char));
export!(deno_webview_eval, fn webview_eval(w: webview_t, js: *const c_char));
export!(deno_webview_return, fn webview_return(w: webview_t, seq: *const c_char, status: c_int, result: *const c_char));

/// # Safety
///
/// webview pointer must be non NULL. It must be obtained using
/// `webview_create`.
#[no_mangle]
pub unsafe extern "C" fn deno_webview_bind(w: webview_t, name: *const c_char) {
  extern "C" fn callback(
    seq: *const c_char,
    req: *const c_char,
    w: *mut c_void,
  ) {
    let seq = unsafe {
      CStr::from_ptr(seq)
        .to_str()
        .expect("No null bytes in parameter seq")
    }.to_string();
    let req = unsafe {
      CStr::from_ptr(req)
        .to_str()
        .expect("No null bytes in parameter req")
    }.to_string();
    println!("seq req {} {}", seq, req);

    CHANNEL.0.send((seq, req)).unwrap();
  }

  webview_official_sys::webview_bind(
    w,
    name,
    Some(callback),
    w,
  )
}

#[no_mangle]
pub extern "C" fn deno_webview_get_recv() -> *const u8 {
  let recv = CHANNEL.1.recv().unwrap();
  let mut recv_buf = Vec::new();

  recv_buf.extend_from_slice(&u32::to_be_bytes(recv.0.len() as u32));
  recv_buf.extend_from_slice(recv.0.as_bytes());

  recv_buf.extend_from_slice(&u32::to_be_bytes(recv.1.len() as u32));
  recv_buf.extend_from_slice(recv.1.as_bytes());

  println!("recv {} {}", recv.0, recv.1);

  let ptr = recv_buf.as_ptr();
  std::mem::forget(recv_buf);
  ptr
}
