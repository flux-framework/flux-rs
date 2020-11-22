#[macro_use]
extern crate error_chain;

pub mod my_future;
use my_future::*;

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
    pub fn open(uri: &str, flags: u32) -> Result<Flux> {
        let p = if uri.len() == 0 {
            std::ptr::null()
        } else {
            CString::new(uri)?.as_ptr()
        };
        unsafe { flux_sys::flux_open(p, flags as i32) }
            .flux_check()
            .map(|f| Flux { handle: f })
    }
    pub fn attr_get(self: &mut Self, name: &str) -> Result<&str> {
        let s = CString::new(name)?;
        unsafe {
            let cstr_ptr = flux_sys::flux_attr_get(self.handle, s.as_ptr()).flux_check()?;
            Ok(CStr::from_ptr(cstr_ptr).to_str()?)
        }
    }
    pub fn service_register(self: &mut Self, name: &str) -> Result<FluxFuture> {
        let s = CString::new(name)?;
        Ok(unsafe {
            FluxFuture::from_ptr(
                flux_sys::flux_service_register(self.handle, s.as_ptr()).flux_check()?,
            )
        })
    }
    pub fn kvs_lookup(
        self: &mut Self,
        flags: flux_sys::kvs_op::Type,
        key: &str,
    ) -> Result<FluxKvsFuture> {
        let k = CString::new(key)?;
        let fut_ptr = unsafe { flux_sys::flux_kvs_lookup(self.handle, std::ptr::null(), flags as i32, k.as_ptr()) }
            .flux_check()?;
        Ok(FluxKvsFuture::from_ptr(fut_ptr))
    }
    pub fn get_handle(&self) -> *const flux_sys::flux_t {
        self.handle
    }
    pub fn get_handle_mut(&self) -> *mut flux_sys::flux_t {
        self.handle
    }
}

impl Drop for Flux {
    fn drop(&mut self) {
        unsafe { flux_sys::flux_close(self.handle) }
    }
}

pub fn reactor_create(flags: i32) -> *mut flux_sys::flux_reactor_t {
    unsafe { flux_sys::flux_reactor_create(flags) }
}
