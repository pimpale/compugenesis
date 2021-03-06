use cgmath::{Matrix4, One, Point3, Quaternion, Rad, Vector3};
use std::time::Duration;
use std::time::Instant;

#[allow(dead_code)]

pub enum CameraMovementDir {
  Forward,
  Backward,
  Upward,
  Downward,
  Left,
  Right,
}

pub enum CameraRotationDir {
  Upward,
  Downward,
  Left,
  Right,
}

#[derive(Clone, Debug)]
pub struct Camera {
  begin_time: Instant,

  screen_x: u32,            //Horizontal size of screen
  screen_y: u32,            //Vertical size of screen
  worldup: Vector3<f32>,    // The normalized vector that the camera percieves to be up
  rotation: Matrix4<f32>,   // The rotation of the camera
  loc: Point3<f32>,         // The camera's location in 3d space
  projection: Matrix4<f32>, // Projection Matrix
  model: Matrix4<f32>,      // Model Matrix
  view: Matrix4<f32>,       // View Matrix
}

impl Camera {
  pub fn new(location: Point3<f32>, screen_x: u32, screen_y: u32) -> Camera {
    let mut cam = Camera {
      begin_time: Instant::now(),
      screen_x,
      screen_y,
      worldup: Vector3::new(0.0, 1.0, 0.0),
      loc: location,
      projection: Matrix4::one(),
      model: Matrix4::one(),
      view: Matrix4::one(),
      rotation: Matrix4::one(),
    };
    cam.genmodel();
    cam.genview();
    cam.genprojection();
    cam
  }

  pub fn mvp(&self) -> Matrix4<f32> {
    self.projection * self.view * self.model
  }

  pub fn translate(&mut self, delta: Vector3<f32>) {
    self.setloc(self.loc + delta);
  }

  pub fn translate_rot(&mut self, delta: Vector3<f32>) {
    self.translate(self.rotate_by_camera(delta));
  }

  pub fn setloc(&mut self, loc: Point3<f32>) {
    self.loc = loc;
    self.genview();
  }

  pub fn dir_move(&mut self, dir: CameraMovementDir) {
    let scale = 0.1;
    self.translate_rot(
      match dir {
        CameraMovementDir::Forward => Vector3::unit_z(),
        CameraMovementDir::Backward => -Vector3::unit_z(),
        CameraMovementDir::Right => -Vector3::unit_x(),
        CameraMovementDir::Left => Vector3::unit_x(),
        CameraMovementDir::Upward => Vector3::unit_y(),
        CameraMovementDir::Downward => -Vector3::unit_y(),
      } * scale,
    );
  }

  pub fn dir_rotate(&mut self, dir: CameraRotationDir) {
    let rotval = 0.05;

    let ret = match dir {
      CameraRotationDir::Right => Matrix4::from_axis_angle(Vector3::unit_y(), Rad(-rotval)),
      CameraRotationDir::Left => Matrix4::from_axis_angle(Vector3::unit_y(), Rad(rotval)),
      CameraRotationDir::Upward => Matrix4::from_axis_angle(Vector3::unit_x(), Rad(rotval)),
      CameraRotationDir::Downward => Matrix4::from_axis_angle(Vector3::unit_x(), Rad(-rotval)),
    };
    self.rotation = self.rotation  * ret;

    self.genview();
  }

  pub fn setscreen(&mut self, screen_x: u32, screen_y: u32) {
    self.screen_x = screen_x;
    self.screen_y = screen_y;
    self.genprojection();
  }

  fn rotate_by_camera(&self, vec: Vector3<f32>) -> Vector3<f32> {
    (self.rotation * vec.extend(1.0)).truncate()
  }

  fn genview(&mut self) {
    // Look at the place in front of us
    self.view = Matrix4::look_at(
      self.loc,
      self.loc + self.rotate_by_camera(Vector3::unit_z()),
      self.worldup,
    );
  }

  fn genmodel(&mut self) {
    self.model = Matrix4::one()
  }

  fn genprojection(&mut self) {
    let aspect_ratio = self.screen_x as f32 / self.screen_y as f32;
    self.projection =
      cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
  }
}
