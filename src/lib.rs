#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::mem::MaybeUninit;
use std::ffi::{CString, CStr};
use std::ptr::NonNull;
use std::convert::TryInto;

mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Debug)]
pub enum IwError {
    NoSocket(i32),
    NoRange(i32),
    ScanFailed(i32),
    InvalidInterfaceName,
}

#[derive(Debug)]
pub struct AccessPoint {
    essid: Option<String>,
}

pub fn scan_wifi(interface: &str) -> Result<Vec<AccessPoint>, IwError> {
    // Opening a socket is safe
    let sock = unsafe { ffi::iw_sockets_open() };

    if sock < 0 {
        return Err(IwError::NoSocket(sock));
    }

    let mut range: MaybeUninit<ffi::iwrange> = MaybeUninit::uninit();

    let interface = if let Ok(interface) = CString::new(interface) {
        interface
    } else {
        return Err(IwError::InvalidInterfaceName);
    };

    // The socket is correctly initialized, as well as range has the correct size, thus this is
    // safe
    let result = unsafe { ffi::iw_get_range_info(sock, interface.as_ptr(), range.as_mut_ptr()) };

    if result < 0 {
        return Err(IwError::NoRange(result));
    }

    // If the result is >= 0 no errors were reported and we can assume that `range` is initialized
    let range = unsafe { range.assume_init() };

    let mut head: MaybeUninit<ffi::wireless_scan_head> = MaybeUninit::uninit();

    // head has the correct size
    // Casting the interface as 'mut' is safe here, as the interface does _not_ get modified
    // internally by iwlib
    let result = unsafe { ffi::iw_scan(sock, interface.as_ptr() as *mut _, range.we_version_compiled as i32, head.as_mut_ptr()) };

    if result < 0 {
        return Err(IwError::ScanFailed(result));
    }

    // If the result is >= 0 no errors were reported and we can assume that `head` is initialized
    let head = unsafe { head.assume_init() };

    let mut ret = vec![];
    let mut next = NonNull::new(head.result);

    while let Some(value) = next {
        let result = unsafe { value.as_ref() };

        let config = &result.b;

        let mut ap = AccessPoint { essid: None };

        if config.has_essid != 0 {
            let essid = unsafe { CStr::from_ptr(&config.essid as *const i8) };
            ap.essid = essid.to_str().map(String::from).ok();
        }

        next = NonNull::new(result.next);
        unsafe { ffi::free(value.as_ptr() as *mut std::ffi::c_void) };

        ret.push(ap);
    }

    Ok(ret)
}
