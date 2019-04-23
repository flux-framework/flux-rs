#[macro_use]
extern crate error_chain;

mod future;
use future::*;

use flux_sys;

// set up error chain module
mod errors;
use crate::errors::FluxResultCheck;
use crate::errors::*;

use std::ffi::{CStr, CString};

// type Result<T> = std::result::Result<T, Box<std::error::Error>>;

#[derive(Debug)]
pub struct Flux {
    handle: *mut flux_sys::flux_t,
}

trait ToPtr {
    fn to_ptr_or_null(self: &Self) -> *const std::os::raw::c_char;
}

impl ToPtr for CString {
    fn to_ptr_or_null(self: &Self) -> *const std::os::raw::c_char {
        if self.as_bytes().len() == 0 {
            std::ptr::null()
        } else {
            self.as_ptr()
        }
    }
}

impl Flux {
    fn open(uri: &str, flags: u32) -> Result<Flux> {
        let p = if uri.len() == 0 {
            std::ptr::null()
        } else {
            CString::new(uri)?.as_ptr()
        };
        unsafe { flux_sys::flux_open(p, flags as i32) }
            .flux_check()
            .map(|f| Flux { handle: f })
    }
    fn attr_get(self: &mut Self, name: &str) -> Result<&str> {
        let s = CString::new(name)?;
        unsafe {
            let cstr_ptr = flux_sys::flux_attr_get(self.handle, s.as_ptr()).flux_check()?;
            Ok(CStr::from_ptr(cstr_ptr).to_str()?)
        }
    }
    fn service_register(self: &mut Self, name: &str) -> Result<FluxFuture> {
        let s = CString::new(name)?;
        Ok(unsafe {
            FluxFuture::from_ptr(
                flux_sys::flux_service_register(self.handle, s.as_ptr()).flux_check()?,
            )
        })
    }
    fn kvs_lookup(
        self: &mut Self,
        flags: flux_sys::kvs_op::Type,
        key: &str,
    ) -> Result<FluxKvsFuture> {
        let k = CString::new(key)?;
        let fut_ptr =
            unsafe { flux_sys::flux_kvs_lookup(self.handle, flags as i32, k.as_ptr()) }
                .flux_check()?;
        Ok(FluxKvsFuture::from_ptr(fut_ptr))
    }
}

impl Drop for Flux {
    fn drop(&mut self) {
        unsafe { flux_sys::flux_close(self.handle) }
    }
}

fn reactor_create(flags: i32) -> *mut flux_sys::flux_reactor_t {
    unsafe { flux_sys::flux_reactor_create(flags) }
}

fn main() -> Result<()> {
    eprintln!("starting");
    let mut h = Flux::open("", 0)?;
    // h.service_register("sched")?.get()?;
    eprintln!("got a handle!");
    eprintln!("Hello, world! size:{:?}", h.attr_get("size")?);
    let mut composite = h.kvs_lookup(0, "thing")?
        .and_then(|fi| {
            eprintln!("kvs result:{:?}", fi.lookup_get()?);
            h.kvs_lookup(0, "other_thing")
        })?
        .and_then(|fi| {
            eprintln!("kvs result2:{:?}", fi.lookup_get()?);
            h.kvs_lookup(0, "other_thing")
        })?
        .then(|f| {
            // f is strongly typed with kvs future methods
            println!("kvs result3:{:?}", f.lookup_get().unwrap());
        })?;
    composite.wait_for(-1.0)?;
    unsafe {
        flux_sys::flux_reactor_run(flux_sys::flux_get_reactor(h.handle), 0);
    };
    Ok(())
}
