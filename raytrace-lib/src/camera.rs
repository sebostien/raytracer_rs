use crate::{ray::Ray, Rotation, Vec3, VIEWPORT_DISTANCE};

#[derive(Debug)]
pub struct Camera {
    /// The position of the camera.
    position: Vec3,
    /// Rotation of the camera.
    rotation: Rotation,
    /// The viewport to sends rays through.
    viewport: Viewport,
    /// The field-of-view in radians for the camera.
    fov: f64,
    /// The distance from the camera to the viewport.
    distance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraNewError {
    DirectionZero,
}

impl std::fmt::Display for CameraNewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectionZero => write!(f, "Camera direction must be non-zero"),
        }
    }
}

impl std::error::Error for CameraNewError {}

impl Camera {
    /// Create a new camera.
    ///
    /// # Arguments
    ///
    /// * `width`    - Number of horizontal pixels in the resulting frame
    /// * `height`   - Number of vertical pixels in the resulting frame
    /// * `position` - The position of the camera
    /// * `view_dir` - The direction that the camera looks in
    /// * `fov`      - Field of view in degrees [0, 180)
    pub fn new(
        width: u32,
        height: u32,
        position: Vec3,
        view_dir: Vec3,
        fov: f64,
    ) -> Result<Self, CameraNewError> {
        let fov_rad = (fov / 2.0) * std::f64::consts::PI / 180.0;

        if view_dir.length_squared() == 0.0 {
            return Err(CameraNewError::DirectionZero);
        }

        Ok(Self {
            position,
            rotation: view_dir.into(),
            viewport: Viewport::new(width, height, fov_rad),
            fov: fov_rad,
            distance: 1.0 / (fov_rad / 2.0).tan(),
        })
    }

    pub fn set_width(&mut self, width: u32) {
        self.viewport = Viewport::new(width, self.viewport.pixels_y, self.fov);
    }

    pub fn set_height(&mut self, height: u32) {
        self.viewport = Viewport::new(self.viewport.pixels_x, height, self.fov);
    }

    /// Returns a ray with origin from the cameras position
    /// and in the direction of the pixel.
    pub fn ray_from_pixel(&self, pixel_x: f64, pixel_y: f64) -> Ray {
        // Map x to range [-aspect_ratio, aspect_ratio]
        let x = (pixel_x + 0.5) * self.viewport.pixel_width - self.viewport.aspect_ratio;
        // Map y to range [-1, 1]
        let y = (pixel_y + 0.5) * self.viewport.pixel_height - 1.0;

        let direction = Vec3::new(x, y, self.distance).rotate(&self.rotation);

        let origin = self.position;
        Ray::new(origin, direction)
    }

    /// Returns the number of pixels in the resulting image.
    /// (width, height)
    pub fn pixels(&self) -> (u32, u32) {
        (self.viewport.pixels_x, self.viewport.pixels_y)
    }
}

/// A plane in front of the camera.
///
/// The plane has dimensions:
/// Top left: (-aspect_ratio,-1), Bottom right: (aspect_ratio,1)
#[derive(Debug)]
struct Viewport {
    /// `width / height`
    aspect_ratio: f64,
    /// Number of horizontal pixels.
    pixels_x: u32,
    /// Number of vertical pixels.
    pixels_y: u32,
    /// The distance between two pixels in the x-direction.
    pixel_width: f64,
    /// The distance between two pixels in the y-direction.
    pixel_height: f64,
}

impl Viewport {
    /// Width and height are the number of pixels in
    /// the image which is used to calculate aspect ratio.
    pub fn new(width: u32, height: u32, fov: f64) -> Self {
        let w = width as f64;
        let h = height as f64;
        let aspect_ratio = w / h;

        Self {
            pixels_x: width,
            pixels_y: height,
            aspect_ratio,
            // The grid should be `aspect_ratio` wide.
            pixel_width: 2.0 * aspect_ratio / w,
            // The grid sohuld be 2 m tall
            pixel_height: 2.0 / h,
        }
    }
}
