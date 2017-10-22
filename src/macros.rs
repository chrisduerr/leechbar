// Utility macro for setting window properties
// This returns a Result
macro_rules! set_prop {
    ($conn:expr, $window:expr, $name:expr, @atom $value:expr) => {
        {
            match xcb::intern_atom($conn, true, $value).get_reply() {
                Ok(atom) => set_prop!($conn, $window, $name, &[atom.atom()], "ATOM", 32),
                Err(e) => Err(ErrorKind::XcbPropertyError(e.error_code())),
            }
        }
    };
    ($conn:expr, $window:expr, $name:expr, $data:expr) => {
        {
            set_prop!($conn, $window, $name, $data, "CARDINAL", 32)
        }
    };
    ($conn:expr, $window:expr, $name:expr, $data:expr, $type:expr, $size:expr) => {
        {
            let type_atom = xcb::intern_atom($conn, true, $type).get_reply();
            let property = xcb::intern_atom($conn, true, $name).get_reply();
            match (type_atom, property) {
                (Ok(type_atom), Ok(property)) => {
                    let property = property.atom();
                    let type_atom = type_atom.atom();
                    let mode = xcb::PROP_MODE_REPLACE as u8;
                    xcb::change_property($conn, mode, $window, property, type_atom, $size, $data);
                    Ok(())
                },
                (Err(e), _) | (_, Err(e)) => Err(ErrorKind::XcbPropertyError(e.error_code())),
            }
        }
    };
}

// Log a result with the error log level
macro_rules! err {
    ($res:expr, $msg:expr, $($args:expr),*) => {
        {
            if let Err(err) = $res {
                error!(concat!($msg, ": {}"), $($args),*, err);
            }
        }
    };
    ($res:expr, $msg:expr) => {
        {
            if let Err(err) = $res {
                error!(concat!($msg, ": {}"), err);
            }
        }
    }
}

// Attempts an XCB operation and returns an error when it fails
macro_rules! xtry {
    ($func:ident, $($args:expr),*) => {
        {
            xcb::$func($($args),*)
                .request_check()
                .map_err(|e| ErrorKind::XError(e.error_code().to_string()))?;
        }
    };
    (@render $func:ident, $($args:expr),*) => {
        {
            xcb::render::$func($($args),*)
                .request_check()
                .map_err(|e| ErrorKind::XError(e.error_code().to_string()))?;
        }
    };
}
