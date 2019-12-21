use std::ffi::{OsString, OsStr, CString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixDatagram;
use std::fs::File;
use std::io::Read;
use std::mem::MaybeUninit;

use libc::ioctl;

mod ffi;
mod error;

#[derive(Debug)]
pub struct Interface {
    name: OsString,
}

impl Interface {
    pub fn find_interface<S: AsRef<str>>(name: S) -> Result<Option<Interface>, error::InterfaceListError> {
        let ifs = interfaces()?;

        Ok(ifs.into_iter().find(|i| i.name == name.as_ref()))
    }

    pub fn get_connected_essid(&self) -> Result<CString, error::EssidFetchError> {
        let socket = UnixDatagram::unbound()?;
        let socketfd = socket.as_raw_fd();

        let mut wreq: ffi::iwreq = unsafe { MaybeUninit::zeroed().assume_init() };

        let mut name = [0u8; 16];

        name[0..self.name.len()].copy_from_slice(self.name.as_bytes());

        unsafe { wreq.ifr_ifrn.ifrn_name.copy_from_slice(std::mem::transmute(&name[..])) };

        wreq.u.essid.length = (ffi::IW_ESSID_MAX_SIZE + 1) as u16;

        let mut name: MaybeUninit<[u8; ffi::IW_ESSID_MAX_SIZE as usize + 1]> = MaybeUninit::uninit();

        wreq.u.essid.pointer = name.as_mut_ptr() as *mut _;

        let ret = unsafe { ioctl(socketfd, ffi::SIOCGIWESSID.into(), &mut wreq as *mut _) };
        if ret == -1 {
            return Err(std::io::Error::last_os_error())?;
        }

        Ok(CString::new(unsafe {&name.assume_init()[..wreq.u.essid.length as usize - 1]})?)
    }

    pub fn get_name(&self) -> &OsStr {
        &self.name
    }
}

pub fn interfaces() -> Result<Vec<Interface>, error::InterfaceListError> {

    let mut proc_wireless = File::open("/proc/net/wireless")?;

    let mut buf = vec![];

    proc_wireless.read_to_end(&mut buf)?;

    Ok(buf.split(|&b| b == b'\n').skip(2)
        .flat_map(|line| line.split(|&c| c == b':').next())
        .flat_map(|line| line.rsplit(|c| c.is_ascii_whitespace()).next())
        .filter(|line| line.len() != 0)
        .map(|n| OsStr::from_bytes(n))
        .map(|n| Interface { name: n.to_os_string() })
        .collect())
}
