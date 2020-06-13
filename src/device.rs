use nix::mount::{mount, MsFlags};
use nix::sys::stat;
use nix::unistd;

use nix::sys::stat::makedev;
use nix::sys::stat::mknod;
use nix::sys::stat::Mode;
use std::error::Error;
use std::path::Path;

struct Device<'a> {
    path: &'static str,
    dev_type: nix::sys::stat::SFlag,
    major: u64,
    minor: u64,
    file_mode: &'a [nix::sys::stat::Mode],
}

const DEVICES: [Device; 4] = [
    Device {
        path: "/dev/console",
        dev_type: nix::sys::stat::SFlag::S_IFCHR,
        major: 5,
        minor: 1,
        file_mode: &[Mode::S_IRWXU],
    },
    Device {
        path: "/dev/null",
        dev_type: nix::sys::stat::SFlag::S_IFCHR,
        major: 1,
        minor: 3,
        file_mode: &[Mode::S_IRWXU],
    },
    /*
        Device {
            path: "/dev/sda",
            dev_type: nix::sys::stat::SFlag::S_IFBLK,
            major: 8,
            minor: 0,
            file_mode: &[Mode::S_IRWXU],
        },
        Device {
            path: "/dev/sda1",
            dev_type: nix::sys::stat::SFlag::S_IFBLK,
            major: 8,
            minor: 1,
            file_mode: &[Mode::S_IRWXU],
        },
        Device {
            path: "/dev/sdb",
            dev_type: nix::sys::stat::SFlag::S_IFBLK,
            major: 8,
            minor: 16,
            file_mode: &[Mode::S_IRWXU],
        },
        Device {
            path: "/dev/sdb1",
            dev_type: nix::sys::stat::SFlag::S_IFBLK,
            major: 8,
            minor: 17,
            file_mode: &[Mode::S_IRWXU],
        },
    */
    Device {
        path: "/dev/random",
        dev_type: nix::sys::stat::SFlag::S_IFCHR,
        major: 1,
        minor: 8,
        file_mode: &[Mode::S_IRWXU, Mode::S_IRGRP, Mode::S_IROTH],
    },
    Device {
        path: "/dev/urandom",
        dev_type: nix::sys::stat::SFlag::S_IFCHR,
        major: 1,
        minor: 9,
        file_mode: &[Mode::S_IRWXU, Mode::S_IRGRP, Mode::S_IROTH],
    },
];

pub fn make_basic_devices() -> Result<(), Box<dyn Error>> {
    for dev in DEVICES.iter() {
        let mode = dev.file_mode.iter().fold(dev.file_mode[0], |m, i| m | *i);
        mknod(dev.path, dev.dev_type, mode, makedev(dev.major, dev.minor))?
    }

    Ok(())
}

pub fn mount_basics() -> Result<(), Box<dyn Error>> {
    let path_dev: &'static [u8] = b"/dev";
    let path_proc: &'static [u8] = b"/proc";
    let path_sys: &'static [u8] = b"/sys";
    let path_dev_pts: &'static [u8] = b"/dev/pts";
    let path_dev_socket: &'static [u8] = b"/dev/socket";

    if !Path::new("/dev").is_dir() {
        unistd::mkdir(path_dev, stat::Mode::S_IRWXU)?;
    }
    if !Path::new("/proc").is_dir() {
        unistd::mkdir(path_proc, stat::Mode::S_IRWXU)?;
    }
    if !Path::new("/sys").is_dir() {
        unistd::mkdir(path_sys, stat::Mode::S_IRWXU)?;
    }

    mount(
        Some(b"tmpfs".as_ref()),
        path_dev,
        Some(b"tmpfs".as_ref()),
        MsFlags::MS_NOSUID,
        Some(b"mode=0755".as_ref()),
    )
    .unwrap_or_else(|e| panic!("mount of /dev failed: {}", e));

    unistd::mkdir(path_dev_pts, stat::Mode::S_IRWXU)?;
    unistd::mkdir(path_dev_socket, stat::Mode::S_IRWXU)?;

    mount(
        Some(b"devpts".as_ref()),
        path_dev_pts,
        Some(b"devpts".as_ref()),
        MsFlags::empty(),
        Some(b"".as_ref()),
    )
    .unwrap_or_else(|e| panic!("mount of /dev/pts failed: {}", e));

    mount(
        Some(b"proc".as_ref()),
        path_proc,
        Some(b"proc".as_ref()),
        MsFlags::empty(),
        Some(b"".as_ref()),
    )
    .unwrap_or_else(|e| panic!("mount of /proc failed: {}", e));

    mount(
        Some(b"sysfs".as_ref()),
        path_sys,
        Some(b"sysfs".as_ref()),
        MsFlags::empty(),
        Some(b"".as_ref()),
    )
    .unwrap_or_else(|e| panic!("mount of /sys failed: {}", e));

    Ok(())
}
