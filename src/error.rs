//! Error Types.

use std::fmt;

error_chain! {
    foreign_links {
        XcbConnectionError(::xcb::ConnError);
    }

    errors {
        /// Unable to find a screen.
        ///
        /// This error occurs when the connection was possible, but no screen could be found.
        XcbNoScreenError(err: ()) {
            description("No screen found"),
            display("No screen found"),
        }

        /// Unable to set a window property.
        ///
        /// This returns an XCB error code. These codes are defined [here].
        ///
        /// [here]: https://cgit.freedesktop.org/xorg/proto/xproto/tree/X.h#n346
        XcbPropertyError(code: u8) {
            description("Unable to set window property"),
            display("Unable to set window property: '{}'", code),
        }

        /// Unable to get screen resources.
        ///
        /// This returns an XCB error code. These codes are defined [here].
        ///
        /// [here]: https://cgit.freedesktop.org/xorg/proto/xproto/tree/X.h#n346
        XcbScreenResourcesError(code: u8) {
            description("Unable to get screen resources"),
            display("Unable to get screen resources: '{}'", code),
        }

        /// Unable to get primary screen information. This only occurs when not specifying an
        /// output manually.
        ///
        /// This returns an XCB error code. These codes are defined [here].
        ///
        /// [here]: https://cgit.freedesktop.org/xorg/proto/xproto/tree/X.h#n346
        PrimaryScreenInfoError(code: u8) {
            description("Unable to get primary screen information"),
            display("Unable to get primary screen information: '{}'", code),
        }

        /// Indicates an error with an XCB request. This is usually not because of parameters
        /// specified, but because there was an issue with the X.Org connection.
        XError(description: String) {
            description("Unable to send XCB request"),
            display("Unable to send XCB request: '{}'", description),
        }

        /// The screen does not support a 32 bit visual.
        ScreenDepthError(arg: ()) {
            description("Invalid screen depth support"),
            display("The screen does not support 32 bit depth visuals"),
        }
    }
}

/// Different types of bar creation errors.
///
/// These are all the different errors that can occur during the creation of the bar.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BarErrorKind {
    /// Unable to connect to the X.Org server. Please make sure you are running X.Org and not
    /// Wayland.
    ConnectionRefused,
    /// No primary output could be found. This is most likely because you have only one output and
    /// it is not set as primary.
    ///
    /// You can set the `primary` flag on your output using `xrandr --output <OUTPUT> --primary`.
    /// If this does not work, you can set the output directly using
    /// [`output`](struct.BarBuilder.html#method.output).
    NoPrimaryOutput,
    /// The specified output could not be found. Please make sure the correct name is used. You can
    /// find out the name of your outputs using `xrandr`.
    OutputNotFound,
}

impl BarErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            BarErrorKind::ConnectionRefused => "Unable to connect to X.Org",
            BarErrorKind::NoPrimaryOutput => "Unable to find primary output (see docs)",
            BarErrorKind::OutputNotFound => "Unable to find specified output",
        }
    }
}

/// Bar creation error.
///
/// This error is returned when anything went wrong during the creation of the bar.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct BarError {
    /// Different types of bar creation errors.
    pub kind: BarErrorKind,
}

impl From<BarErrorKind> for BarError {
    fn from(kind: BarErrorKind) -> BarError {
        BarError { kind }
    }
}

impl fmt::Display for BarError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.kind.as_str())
    }
}

impl ::std::error::Error for BarError {
    fn description(&self) -> &str {
        self.kind.as_str()
    }
}
