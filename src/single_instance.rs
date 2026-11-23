//! Hold a named kernel object for the lifetime of the desktop application.
#[cfg(windows)]
mod platform {
    use std::{ffi::c_void, io};

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateMutexW(attributes: *const c_void, owner: i32, name: *const u16) -> *mut c_void;
        fn GetLastError() -> u32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    pub struct Instance(*mut c_void);

    impl Instance {
        pub fn acquire() -> io::Result<Option<Self>> {
            // Local scopes the pet to this interactive Windows session. The name
            // stays identical across versions and executable locations.
            Self::named("Local\\BusyCat.DesktopPet.SingleInstance")
        }

        fn named(name: &str) -> io::Result<Option<Self>> {
            let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
            let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
            let error = unsafe { GetLastError() };
            if handle.is_null() {
                return Err(io::Error::from_raw_os_error(error as i32));
            }
            let instance = Self(handle);
            if error == 183 {
                // ERROR_ALREADY_EXISTS
                return Ok(None); // Drop closes this duplicate handle.
            }
            Ok(Some(instance))
        }
    }

    impl Drop for Instance {
        fn drop(&mut self) {
            // Windows also closes the handle if the process crashes or is killed.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn duplicate_is_rejected_and_exit_allows_restart() {
            let name = format!("Local\\BusyCat.Test.{}", std::process::id());
            let first = Instance::named(&name).unwrap().expect("first instance");
            assert!(Instance::named(&name).unwrap().is_none());
            drop(first);
            assert!(Instance::named(&name).unwrap().is_some());
        }
    }
}

#[cfg(windows)]
pub use platform::Instance;

#[cfg(not(windows))]
pub struct Instance;
#[cfg(not(windows))]
impl Instance {
    pub fn acquire() -> std::io::Result<Option<Self>> {
        Ok(Some(Self))
    }
}
