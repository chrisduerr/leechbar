error_chain! {
    foreign_links {
        XcbConnectionError(::xcb::ConnError);
    }

    errors {
        /// Unable to find a screen.
        ///
        /// This error occurs when the connection was possible, but no screen could be found.
        XcbNoScreenError(err: ()) {
            description("no screen found"),
            display("no screen found"),
        }

        /// Unable to set a window property.
        ///
        /// This returns an XCB error code. These codes are defined [here].
        ///
        /// [here]: https://cgit.freedesktop.org/xorg/proto/xproto/tree/X.h#n346
        XcbPropertyError(code: u8) {
            description("unable to set window property"),
            display("unable to set window property: '{}'", code),
        }

        /// Unable to get screen resources.
        ///
        /// This returns an XCB error code. These codes are defined [here].
        ///
        /// [here]: https://cgit.freedesktop.org/xorg/proto/xproto/tree/X.h#n346
        XcbScreenResourcesError(code: u8) {
            description("unable to get screen resources"),
            display("unable to get screen resources: '{}'", code),
        }

        /// Unable to get primary screen information. This only occurs when not specifying an
        /// output manually.
        ///
        /// This returns an XCB error code. These codes are defined [here].
        ///
        /// [here]: https://cgit.freedesktop.org/xorg/proto/xproto/tree/X.h#n346
        PrimaryScreenInfoError(code: u8) {
            description("unable to get primary screen information"),
            display("unable to get primary screen information: '{}'", code),
        }
    }
}
