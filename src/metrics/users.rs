use libc::{c_char, c_int, c_short, c_void, pid_t, size_t};
use std::fs::File;
use std::mem;
use std::os::unix::prelude::*;

const UT_LINESIZE: usize = 32;
const UT_NAMESIZE: usize = 32;
const UT_HOSTSIZE: usize = 256;
static UTMP_FILE_PATH: &'static str = "/var/run/utmp";
static UTMPX_FILE_PATH: &'static str = "/var/run/utmpx";

#[repr(C)]
#[derive(Debug)]
pub struct exit_status {
    pub e_termination: c_short,
    pub e_exit: c_short,
}

#[repr(C)]
#[derive(Debug)]
pub struct ut_tv {
    pub tv_sec: i32,
    pub tv_usec: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct utmp {
    pub ut_type: c_short,
    pub ut_pid: pid_t,
    pub ut_line: [c_char; UT_LINESIZE],
    pub ut_id: [c_char; 4],
    pub ut_user: [c_char; UT_NAMESIZE],
    pub ut_host: [c_char; UT_HOSTSIZE],
    pub ut_exit: exit_status,
    pub ut_session: i32,
    pub ut_tv: ut_tv,
    pub ut_addr_v6: [i32; 4],
    pub __glibc_reserved: [c_char; 20],
}

impl Default for exit_status {
    fn default() -> exit_status {
        exit_status {
            e_termination: 0,
            e_exit: 0,
        }
    }
}

impl Default for ut_tv {
    fn default() -> ut_tv {
        ut_tv {
            tv_sec: 0,
            tv_usec: 0,
        }
    }
}

impl Default for utmp {
    fn default() -> utmp {
        utmp {
            ut_type: 0,
            ut_pid: 0,
            ut_line: [0; UT_LINESIZE],
            ut_id: [0; 4],
            ut_user: [0; UT_NAMESIZE],
            ut_host: [0; UT_HOSTSIZE],
            ut_exit: Default::default(),
            ut_session: 0,
            ut_tv: Default::default(),
            ut_addr_v6: [0; 4],
            __glibc_reserved: [0; 20],
        }
    }
}

extern "C" {
    pub fn read(fd: c_int, buf: *mut c_void, count: size_t) -> usize;
}

/// Get the currently logged user from /var/run/utmp
/// UTMP Struct is the same as the one from C utmp.h
/// The check to see if the utmp struct is from a user respect the C standarts
/// ut_type == USER_PROCESS == 7
/// Transform the ut_user (&[i8; 32]) to &[u8] unsing unsafe transmute for speed concern
/// Also relying on C's read (might change later, not sure)
#[cfg(target_family = "unix")]
pub fn get_utmp() -> Vec<String> {
	let mut users: Vec<String> = Vec::new();
	let utmp_file = match File::open(UTMP_FILE_PATH) {
		Ok(val) => val,
		Err(_) => return users
	};
	let mut utmp_struct: utmp = Default::default();
    let buffer: *mut c_void = &mut utmp_struct as *mut _ as *mut c_void;

    unsafe {
        while read(utmp_file.as_raw_fd(), buffer, mem::size_of::<utmp>()) != 0 {
            let cbuffer = &*(buffer as *mut utmp) as &utmp;
            let cuser = std::mem::transmute::<&[i8], &[u8]>(&cbuffer.ut_user);

            if cbuffer.ut_type == 7 {
				let csuser = std::str::from_utf8(cuser).unwrap_or("unknown").trim_matches('\0');
                dbg!(csuser);
				users.push(csuser.to_string());
            }
        }
	}
	users
}


// #[cfg(target_os = "macos")]
// pub fn get_utmp() -> Vec<String> {
// 	todo!()
// }

#[cfg(target_os = "windows")]
pub fn get_utmp() -> Vec<String> {
	todo!()
}