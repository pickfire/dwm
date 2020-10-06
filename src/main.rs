use anyhow::Result;
use std::ffi::CString;
use std::{process, ptr};
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::{xproto::*, Error};
use x11rb::COPY_DEPTH_FROM_PARENT;

fn check_other_wm<C: Connection>(conn: &C, screen: &Screen) -> Result<(), ReplyError> {
    let values = ChangeWindowAttributesAux::new().event_mask(
        EventMask::SubstructureRedirect | EventMask::SubstructureNotify | EventMask::EnterWindow,
    );
    let res = conn.change_window_attributes(screen.root, &values)?.check();
    if let Err(ReplyError::X11Error(Error::Access(_))) = res {
        eprintln!("another window manager is already running");
        process::exit(1);
    }
    res
}

fn setup() -> Result<()> {
    unsafe {
        // sigchld
        signal_hook::register(signal_hook::SIGCHLD, || {
            while 0 < libc::waitpid(-1, ptr::null_mut(), libc::WNOHANG) {}
        })?;
    }

    todo!()
}

fn main() -> Result<()> {
    if unsafe { libc::setlocale(libc::LC_CTYPE, CString::default().as_ptr()) }.is_null() {
        panic!("cannot set locale");
    }

    let (conn, screen_num) = x11rb::connect(None).expect("cannot open display");
    let screen = conn.setup();
    let screen = &screen.roots[screen_num];

    check_other_wm(&conn, screen)?;
    setup()?;

    let win_id = conn.generate_id()?;
    conn.create_window(
        COPY_DEPTH_FROM_PARENT,
        win_id,
        screen.root,
        0,
        0,
        100,
        100,
        0,
        WindowClass::InputOutput,
        0,
        &CreateWindowAux::new().background_pixel(screen.white_pixel),
    )?;
    conn.map_window(win_id)?;
    conn.flush().ok();
    loop {
        println!("Event: {:?}", conn.wait_for_event()?);
    }
}
