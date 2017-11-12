extern crate chan;
#[macro_use]
extern crate lazy_static;
extern crate leechbar;
extern crate libc;
extern crate libpulse_sys;

use leechbar::{Bar, BarBuilder, Component, Foreground, Text};
use std::sync::{Arc, Mutex};
use libpulse_sys::*;
use std::ptr;

// Set the 100% volume
const MAX_VOL: f64 = 65536.;

// Create globals because the pulse event queue has no access to any struct
lazy_static! {
    // This is the current volume
    static ref VOLUME: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
    // This channel is used for prompting the bar to redraw
    static ref CHANNEL: (chan::Sender<()>, chan::Receiver<()>) = chan::sync(0);
}

// Volume component struct
struct VolumeComponent {
    bar: Bar,
    last_volume: u8,
    text: Text,
}

// Create the volume component
impl VolumeComponent {
    fn new(bar: Bar) -> Self {
        // Start pulse listening
        unsafe { start_volume_listener() };

        // Set default text to "0"
        let text = Text::new(&bar, "0", None, None).unwrap();
        Self {
            bar,
            text,
            last_volume: 255,
        }
    }
}

// Implement leechbar::Component for the volume component
impl Component for VolumeComponent {
    // Update bar when `VOLUME` has changed
    fn update(&mut self) -> bool {
        // Lock the volume temporarily
        let vol_lock = VOLUME.lock().unwrap();

        // Don't redraw without change
        if *vol_lock == self.last_volume {
            return false;
        }

        // Redraw text if it changed
        self.text = Text::new(&self.bar, &vol_lock.to_string(), None, None).unwrap();
        self.last_volume = *vol_lock;
        true
    }

    // Redraw when global channel receives message
    fn redraw_timer(&mut self) -> chan::Receiver<()> {
        CHANNEL.1.clone()
    }

    // Draw the updated text
    fn foreground(&self) -> Foreground {
        self.text.clone().into()
    }
}

// Start the pulseaudio listener
unsafe fn start_volume_listener() {
    // Start the async main loop
    let pa_mainloop = pa_threaded_mainloop_new();
    pa_threaded_mainloop_start(pa_mainloop);

    // Create a pulseaudio context
    let pa_mainloop_api = pa_threaded_mainloop_get_api(pa_mainloop);
    let pa_context = pa_context_new(pa_mainloop_api, ptr::null());

    // Register the callback for successful context connection
    pa_context_set_state_callback(pa_context, Some(pa_context_callback), ptr::null_mut());
    pa_context_connect(pa_context, ptr::null(), PA_CONTEXT_NOFLAGS, ptr::null());
}

// Callback when pulseaudio context connected
unsafe extern "C" fn pa_context_callback(pa_context: *mut pa_context, _: *mut libc::c_void) {
    // Check the context state
    match pa_context_get_state(pa_context) {
        // Ignore these states
        PA_CONTEXT_CONNECTING | PA_CONTEXT_AUTHORIZING | PA_CONTEXT_SETTING_NAME => (),
        // If the state is ready, we can subscribe to pulse events
        PA_CONTEXT_READY => {
            // Setup the callback for the subscriptyon
            pa_context_set_subscribe_callback(
                pa_context,
                Some(pa_subscription_callback),
                ptr::null_mut(),
            );

            // Subscribe to all sink events
            let pa_operation =
                pa_context_subscribe(pa_context, PA_SUBSCRIPTION_MASK_SINK, None, ptr::null_mut());
            pa_operation_unref(pa_operation);
        }
        _ => {
            // Abort if connection to pulse was not possible
            let error = pa_strerror(pa_context_errno(pa_context));
            pa_context_unref(pa_context);
            panic!("Pulse connection failure: {:?}", error);
        }
    };
}

// Sink event callback
unsafe extern "C" fn pa_subscription_callback(
    pa_context: *mut pa_context,
    _: Enum_pa_subscription_event_type,
    _: u32,
    _: *mut libc::c_void,
) {
    // Get the sink info
    let pa_operation =
        pa_context_get_sink_info_list(pa_context, Some(pa_sink_callback), ptr::null_mut());
    pa_operation_unref(pa_operation);
}

// Get the volume percentage from a sink
unsafe extern "C" fn pa_sink_callback(
    _: *mut Struct_pa_context,
    pa_sink_info: *const Struct_pa_sink_info,
    _: i32,
    _: *mut libc::c_void,
) {
    if !pa_sink_info.is_null() {
        let vol = if (*pa_sink_info).mute == 1 {
            // Set volume to 0 if sink is muted
            0.
        } else {
            // Calculate the volume percentage
            (100. * f64::from(pa_cvolume_avg(&(*pa_sink_info).volume)) / MAX_VOL).round()
        };

        // Update the global state
        let mut lock = VOLUME.lock().unwrap();
        *lock = vol as u8;
        CHANNEL.0.send(());
    }
}

fn main() {
    // Create a new bar
    let mut bar = BarBuilder::new().spawn().unwrap();

    // Add an instance of the component to the bar
    let comp = VolumeComponent::new(bar.clone());
    bar.add(comp);

    // Start the event loop that handles all X events
    bar.start_event_loop();
}
