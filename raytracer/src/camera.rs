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
    /// * `fov`      - Field of view in degrees [0, 180]
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
        })
    }

    pub fn set_width(&mut self, width: u32) {
        self.viewport.set_width(width);
    }

    pub fn set_height(&mut self, height: u32) {
        self.viewport.set_height(height);
    }

    /// Returns a ray with origin from the cameras position
    /// and in the direction of the pixel.
    pub fn ray_from_pixel(&self, pixel_x: usize, pixel_y: usize) -> Ray {
        // Map pixels to range [-1, 1]
        let x = pixel_x as f64 * self.viewport.pixel_width - 0.5;
        let y = pixel_y as f64 * self.viewport.pixel_height - 0.5;

        let direction = Vec3::new(x, y, VIEWPORT_DISTANCE).rotate(&self.rotation);

        let origin = self.position;
        Ray::new(origin, direction)
    }

    /// Returns the number of pixels in the resulting image.
    /// (pixels x, pixels y)
    pub fn pixels(&self) -> (usize, usize) {
        (self.viewport.pixels_x, self.viewport.pixels_y)
    }
}

/// A grid in front of the camera.
///
/// The grid is 2 by 2 meter.
/// Top left: (-1,-1), Bottom right: (1,1)
#[derive(Debug)]
struct Viewport {
    /// `width / height`
    aspect_ratio: f64,
    /// Number of horizontal pixels.
    pixels_x: usize,
    /// Number of vertical pixels.
    pixels_y: usize,
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
            pixels_x: width as usize,
            pixels_y: height as usize,
            aspect_ratio,
            // The grid is 1 m by 1 m.
            // So we divide the grid evenly by the number of pixels.
            pixel_width: 1.0 / w,
            pixel_height: 1.0 / h / aspect_ratio,
        }
    }

    pub fn set_width(&mut self, width: u32) {
        let w = width as f64;
        self.pixels_x = width as usize;
        self.pixel_width = 1.0 / w;
        self.aspect_ratio = w / self.pixels_y as f64;
    }

    pub fn set_height(&mut self, height: u32) {
        let h = height as f64;
        self.pixels_y = height as usize;
        self.pixel_width = 1.0 / h;
        self.aspect_ratio = self.pixels_x as f64 / h;
    }
}
