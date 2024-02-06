// Copied from CedrusDB

#![allow(dead_code)]

pub(crate) use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};

use nix::errno::Errno;
use nix::fcntl::{open, openat, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{fsync, mkdir};

pub struct File {
    fd: OwnedFd,
    fid: u64,
}

impl File {
    fn open_file(rootfd: BorrowedFd, fname: &str, truncate: bool) -> nix::Result<OwnedFd> {
        let raw_fd = openat(
            rootfd.as_raw_fd(),
            fname,
            (if truncate { OFlag::O_TRUNC } else { OFlag::empty() }) | OFlag::O_RDWR,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )?;
        Ok(unsafe { OwnedFd::from_raw_fd(raw_fd) })
    }

    fn create_file(rootfd: BorrowedFd, fname: &str) -> OwnedFd {
        let raw_fd = openat(
            rootfd.as_raw_fd(),
            fname,
            OFlag::O_CREAT | OFlag::O_RDWR,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )
        .unwrap();
        unsafe { OwnedFd::from_raw_fd(raw_fd) }
    }

    fn _get_fname(fid: u64) -> String {
        format!("{:08x}.fw", fid)
    }

    pub fn new(fid: u64, flen: u64, rootfd: BorrowedFd) -> nix::Result<Self> {
        let fname = Self::_get_fname(fid);
        let fd = match Self::open_file(rootfd, &fname, false) {
            Ok(fd) => fd,
            Err(e) => match e {
                Errno::ENOENT => {
                    let fd = Self::create_file(rootfd, &fname);
                    nix::unistd::ftruncate(fd.as_fd(), flen as nix::libc::off_t)?;
                    fd
                }
                e => return Err(e),
            },
        };
        Ok(File { fd, fid })
    }

    pub fn get_fd(&self) -> BorrowedFd {
        self.fd.as_fd()
    }
    pub fn get_fid(&self) -> u64 {
        self.fid
    }
    pub fn get_fname(&self) -> String {
        Self::_get_fname(self.fid)
    }

    pub fn sync(&self) {
        fsync(self.fd.as_raw_fd()).unwrap();
    }
}

pub fn touch_dir(dirname: &str, rootfd: BorrowedFd) -> Result<OwnedFd, Errno> {
    use nix::sys::stat::mkdirat;
    if mkdirat(
        rootfd.as_raw_fd(),
        dirname,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IXUSR,
    )
    .is_err()
    {
        let errno = nix::errno::from_i32(nix::errno::errno());
        if errno != nix::errno::Errno::EEXIST {
            return Err(errno)
        }
    }
    Ok(unsafe {
        OwnedFd::from_raw_fd(openat(
            rootfd.as_raw_fd(),
            dirname,
            OFlag::O_DIRECTORY | OFlag::O_PATH,
            Mode::empty(),
        )?)
    })
}

pub fn open_dir(path: &str, truncate: bool) -> Result<(OwnedFd, bool), nix::Error> {
    let mut reset_header = truncate;
    if truncate {
        let _ = std::fs::remove_dir_all(path);
    }
    match mkdir(path, Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IXUSR) {
        Err(e) => {
            if truncate {
                return Err(e)
            }
        }
        Ok(_) => {
            // the DB did not exist
            reset_header = true
        }
    }
    Ok((
        match open(path, OFlag::O_DIRECTORY | OFlag::O_PATH, Mode::empty()) {
            Ok(fd) => unsafe { OwnedFd::from_raw_fd(fd) },
            Err(e) => return Err(e),
        },
        reset_header,
    ))
}
