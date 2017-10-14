//! Error Types.
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
