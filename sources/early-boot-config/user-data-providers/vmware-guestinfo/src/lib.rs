/// VMware guestinfo
extern crate log;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(not(target_arch = "x86_64"))]
mod not_x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[cfg(not(target_arch = "x86_64"))]
pub use not_x86_64::*;
