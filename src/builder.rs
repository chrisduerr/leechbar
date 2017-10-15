use image::{DynamicImage, GenericImage, Rgba};
use error::*;
use util;
use bar;

/// The bar configuration.
///
/// This is used to configure the bar. After configuration, the bar can be created using the
/// [`spawn`] method.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use leechbar::BarBuilder;
///
/// // All method calls that take parameters are optional
/// BarBuilder::new()
///     .background_color(255, 0, 255, 255)
///     .foreground_color(0, 255, 0, 255)
///     .font("Fira Mono Medium 14")
///     .output("DVI-1")
///     .name("MyBar")
///     .height(30)
///     .spawn()
///     .unwrap();
/// ```
///
/// [`spawn`]: struct.BarBuilder.html#method.spawn
pub struct BarBuilder {
    pub(crate) background_image: Option<DynamicImage>,
    pub(crate) background_color: u32,
    pub(crate) foreground_color: (f64, f64, f64, f64),
    pub(crate) output: Option<String>,
    pub(crate) font: Option<String>,
    pub(crate) name: String,
    pub(crate) height: u16,
    _new_lock: (),
}

impl BarBuilder {
    /// Create a new instance of the `BarBuilder` with default parameters.
    pub fn new() -> Self {
        BarBuilder::default()
    }

    /// Change the default foreground color.
    ///
    /// This takes the rgba values of the color as an ingeger from 0 to 255.
    pub fn foreground_color(mut self, red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        self.foreground_color = (
            f64::from(red) / 255.,
            f64::from(green) / 255.,
            f64::from(blue) / 255.,
            f64::from(alpha) / 255.,
        );
        self
    }

    /// Change the default background color.
    ///
    /// This takes the rgba values of the color as an ingeger from 0 to 255.
    pub fn background_color(mut self, red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        self.background_color = util::color(red, green, blue, alpha);
        self
    }

    /// Change the default background image.
    ///
    /// This takes an image and sets it as the default background for the bar. The image is not
    /// resized or modified in any way, so it is required to manually adjust it to fit the
    /// specified bar geometry.
    pub fn background_image(mut self, image: DynamicImage) -> Self {
        self.background_image = Some(image);
        self
    }

    /// Change the default name of the bar.
    ///
    /// This name is used by your Window Manager.
    pub fn name<T: Into<String>>(mut self, name: T) -> Self {
        self.name = name.into();
        self
    }

    /// Change the default font of the bar.
    ///
    /// This font is used for each block unless manually overwritten.
    pub fn font<T: Into<String>>(mut self, font: T) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Change the default height of the bar.
    ///
    /// This specifies the vertical height used in pixels.
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Change the default output the bar should be displayed on.
    ///
    /// This uses RANDR to get the output with the specified name. An example value for a DVI
    /// output would be `DVI-0`.
    ///
    /// If not specified the primary output is selected.
    pub fn output<T: Into<String>>(mut self, output: T) -> Self {
        self.output = Some(output.into());
        self
    }

    /// Spawn the bar with the currently configured settings.
    ///
    /// This creates a window and registers it as a bar on Xorg.
    pub fn spawn(self) -> Result<bar::Bar> {
        let bar = bar::Bar::new(self)?;
        Ok(bar)
    }
}

impl Default for BarBuilder {
    fn default() -> Self {
        let pixel = Rgba {
            data: [0, 0, 0, 255],
        };
        let mut background = DynamicImage::new_rgba8(1, 1);
        background.put_pixel(0, 0, pixel);

        BarBuilder {
            background_image: None,
            background_color: 0,
            foreground_color: (1., 1., 1., 1.),
            output: None,
            name: "leechbar".into(),
            font: None,
            height: 30,
            _new_lock: (),
        }
    }
}
