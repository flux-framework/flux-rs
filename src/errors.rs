error_chain! {
    foreign_links {
        Null(::std::ffi::NulError);
        Utf8(::std::str::Utf8Error);
        Io(::std::io::Error);
    }
}

pub trait FluxResultCheck {
    type Res;
    fn flux_check(self) -> Result<Self::Res>
        where
            Self: std::marker::Sized;
}

impl FluxResultCheck for std::os::raw::c_int {
    type Res = ();
    fn flux_check(self) -> Result<()>
        where
            Self: std::marker::Sized,
        {
            println!("{:?}", self);
            if self < 0 {
                Err(std::io::Error::last_os_error().into())
            } else {
                Ok(())
            }
        }
}

impl<T> FluxResultCheck for *mut T {
    type Res = Self;
    fn flux_check(self) -> Result<Self>
        where
            Self: std::marker::Sized,
        {
            if self.is_null() {
                Err(std::io::Error::last_os_error().into())
            } else {
                Ok(self)
            }
        }
}

impl<T> FluxResultCheck for *const T {
    type Res = Self;
    fn flux_check(self) -> Result<Self>
        where
            Self: std::marker::Sized,
        {
            if self.is_null() {
                Err(std::io::Error::last_os_error().into())
            } else {
                Ok(self)
            }
        }
}

