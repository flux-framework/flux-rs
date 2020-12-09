use crate::errors::*;

use std::cell::RefCell;
use std::ffi::{CStr, CString};

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct FluxFuture {
    f: *mut flux_sys::flux_future_t,
}

pub trait FromPtr: Sized {
    fn from_ptr(_: *mut flux_sys::flux_future_t) -> Self;
}

impl FromPtr for FluxFuture {
    fn from_ptr(f: *mut flux_sys::flux_future_t) -> FluxFuture {
        FluxFuture { f }
    }
}
impl FromPtr for FluxKvsFuture {
    fn from_ptr(f: *mut flux_sys::flux_future_t) -> FluxKvsFuture {
        FluxKvsFuture {
            f: FluxFuture::from_ptr(f),
        }
    }
}

pub trait MyFuture: FromPtr {
    // must implement
    fn get_inner_mut(&mut self) -> *mut flux_sys::flux_future_t;
    fn get_inner(&self) -> *const flux_sys::flux_future_t;
    fn forget(&mut self);

    // helpers
    unsafe extern "C" fn callback(
        f: *mut flux_sys::flux_future_t,
        arg: *mut ::std::os::raw::c_void,
    ) {
        let mut closure = Box::<RefCell<Box<dyn FnMut(&mut Self)>>>::from_raw(arg as *mut _);
        let mut future = Self::from_ptr(f);
        (closure.get_mut())(&mut future);
        // closure collected here, be sure this is ok
    }
    fn package_flux_continuation<F: FnMut(&mut Self)>(
        func: F,
    ) -> (flux_sys::flux_continuation_f, *mut std::os::raw::c_void) {
        let closure: Box<RefCell<Box<dyn FnMut(&mut Self)>>> =
            Box::new(RefCell::new(Box::new(func)));
        let erased_closure = Box::into_raw(closure) as *mut ::std::os::raw::c_void;
        (Some(Self::callback), erased_closure)
    }
    unsafe extern "C" fn and_cb<R: MyFuture>(
        f: *mut flux_sys::flux_future_t,
        arg: *mut ::std::os::raw::c_void,
    ) {
        let mut closure =
            Box::<RefCell<Box<dyn FnMut(&mut Self) -> Result<R>>>>::from_raw(arg as *mut _);
        let mut future = Self::from_ptr(f);
        match &mut (closure.get_mut())(&mut future) {
            Ok(next) => {
                flux_sys::flux_future_continue(future.get_inner_mut(), next.get_inner_mut())
                    .flux_check()
                    .unwrap(); // panic on failure
                               // next.forget(); // keep it from being reaped here
            }
            Err(_) => {
                //TODO see if we can get the num here
                flux_sys::flux_future_continue_error(f, -1, std::ptr::null())
            }
        };
        // closure collected here, be sure this is ok
    }
    fn package_flux_and_continuation<R: MyFuture, F: FnMut(&mut Self) -> Result<R>>(
        func: F,
    ) -> (flux_sys::flux_continuation_f, *mut std::os::raw::c_void) {
        let closure: Box<RefCell<Box<dyn FnMut(&mut Self) -> Result<R>>>> =
            Box::new(RefCell::new(Box::new(func)));
        let erased_closure = Box::into_raw(closure) as *mut ::std::os::raw::c_void;
        (Some(Self::and_cb::<R>), erased_closure)
    }

    fn create<F: FnMut(&mut Self)>(func: F) -> Result<Self> {
        let (cb, arg) = Self::package_flux_continuation(func);
        Ok(Self::from_ptr(
            unsafe { flux_sys::flux_future_create(cb, arg) }.flux_check()?,
        ))
    }

    // For all futures
    fn get(&mut self) -> Result<()> {
        unsafe {
            flux_sys::flux_future_get(self.get_inner_mut(), std::ptr::null_mut()).flux_check()?
        };
        Ok(())
    }
    // fn fulfill(self: &mut Self) -> Result<()> {
    //     unsafe {
    //         flux_sys::flux_future_fulfill(self.get_inner_mut(), std::ptr::null_mut()).flux_check()?
    //     };
    //     return Ok(());
    // }

    fn then_within<F: FnMut(&mut Self)>(mut self, timeout: f64, func: F) -> Result<Self> {
        let (cb, arg) = Self::package_flux_continuation(func);
        unsafe { flux_sys::flux_future_then(self.get_inner_mut(), timeout, cb, arg) }
            .flux_check()?;
        Ok(self)
    }

    fn then<F: FnMut(&mut Self)>(self, func: F) -> Result<Self> {
        self.then_within(-1.0, func)
    }

    fn and_then<F: FnMut(&mut Self) -> Result<R>, R: MyFuture>(&mut self, func: F) -> Result<R> {
        let (cb, arg) = Self::package_flux_and_continuation(func);
        Ok(R::from_ptr(
            unsafe { flux_sys::flux_future_and_then(self.get_inner_mut(), cb, arg) }
                .flux_check()?,
        ))
    }

    fn or_then<R: MyFuture, F: FnMut(&mut Self) -> Result<R>>(&mut self, func: F) -> Result<R> {
        let (cb, arg) = Self::package_flux_and_continuation(func);
        Ok(R::from_ptr(
            unsafe { flux_sys::flux_future_or_then(self.get_inner_mut(), cb, arg) }.flux_check()?,
        ))
    }

    fn is_ready(&mut self) -> bool {
        unsafe { flux_sys::flux_future_is_ready(self.get_inner_mut()) }
    }

    fn wait_for(&mut self, timeout: f64) -> Result<()> {
        unsafe { flux_sys::flux_future_wait_for(self.get_inner_mut(), timeout) }.flux_check()
    }
}

impl Future for FluxFuture {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_ready() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub trait KvsFuture: MyFuture {}

impl MyFuture for FluxFuture {
    fn get_inner_mut(&mut self) -> *mut flux_sys::flux_future_t {
        self.f
    }
    fn get_inner(&self) -> *const flux_sys::flux_future_t {
        self.f
    }
    fn forget(&mut self) {
        self.f = std::ptr::null_mut();
    }
}

impl Drop for FluxFuture {
    fn drop(&mut self) {
        if !self.f.is_null() {
            // TODO: how can we re-enable this?
            // unsafe { flux_sys::flux_future_destroy(self.f) };
            // self.forget();
        }
    }
}

// impl std::ops::Deref for FluxFuture {
//     type Target = flux_sys::flux_future_t;
//     fn deref(&self) -> &Self::Target {
//         self.f
//     }
// }
// impl std::ops::DerefMut for FluxFuture {
//     fn deref_mut(&mut self) -> &mut flux_sys::flux_future_t {
//         self.f
//     }
// }

#[derive(Debug)]
pub struct FluxKvsFuture {
    f: FluxFuture,
}

impl FluxKvsFuture {
    pub fn lookup_get(&mut self) -> Result<CString> {
        let mut res_ptr: *const ::std::os::raw::c_char = ::std::ptr::null_mut();
        unsafe { flux_sys::flux_kvs_lookup_get(self.f.f, &mut res_ptr) }.flux_check()?;
        if res_ptr.is_null() {
            bail!("null result from kvs_lookup_get");
        }
        Ok(CString::from(unsafe { CStr::from_ptr(res_ptr) }))
    }
}

impl MyFuture for FluxKvsFuture {
    fn get_inner_mut(&mut self) -> *mut flux_sys::flux_future_t {
        self.f.get_inner_mut()
    }
    fn get_inner(&self) -> *const flux_sys::flux_future_t {
        self.f.get_inner()
    }
    fn forget(&mut self) {
        self.f.forget()
    }
}

impl KvsFuture for FluxKvsFuture {}

impl std::ops::Deref for FluxKvsFuture {
    type Target = FluxFuture;
    fn deref(&self) -> &Self::Target {
        &self.f
    }
}
impl std::ops::DerefMut for FluxKvsFuture {
    fn deref_mut(&mut self) -> &mut FluxFuture {
        &mut self.f
    }
}
