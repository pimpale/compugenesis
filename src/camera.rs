use cgmath::{Angle, Deg, InnerSpace, Matrix4, One, Point3, Rad, Vector3, Vector4};
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
    Clockwise,
    Counterclockwise,
}

#[derive(Copy, Clone, Debug)]
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
            screen_x: screen_x,
            screen_y: screen_y,
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

    pub fn translate(&mut self, delta: Vector3<f32>) -> () {
        self.setloc(self.loc + delta);
    }

    pub fn translate_rot(&mut self, delta: Vector3<f32>) -> () {
        self.translate(self.rotate_by_camera(delta));
    }

    pub fn rotate(&mut self, rot: Matrix4<f32>) -> () {
        self.setrot(self.rotation * rot);
    }

    pub fn setloc(&mut self, loc: Point3<f32>) {
        self.loc = loc;
        self.genview();
    }

    pub fn setrot(&mut self, rot: Matrix4<f32>) -> () {
        self.rotation = rot;
        self.genview();
    }

    pub fn dir_move(&mut self, dir: CameraMovementDir) -> () {
        let scale = 0.01;
        match dir {
            CameraMovementDir::Forward => self.translate_rot(Vector3::unit_z() * scale),
            CameraMovementDir::Backward => self.translate_rot(-Vector3::unit_z() * scale),
            CameraMovementDir::Right => self.translate_rot(-Vector3::unit_x() * scale),
            CameraMovementDir::Left => self.translate_rot(Vector3::unit_x() * scale),
            CameraMovementDir::Upward => self.translate_rot(Vector3::unit_y() * scale),
            CameraMovementDir::Downward => self.translate_rot(-Vector3::unit_y() * scale),
        }
    }

    pub fn dir_rotate(&mut self, dir: CameraRotationDir) -> () {
        let scale = 0.05;
        match dir {
            CameraRotationDir::Right => self.rotate(Matrix4::from_angle_y(Rad(-scale))),
            CameraRotationDir::Left => self.rotate(Matrix4::from_angle_y(Rad(scale))),
            CameraRotationDir::Upward => self.rotate(Matrix4::from_angle_x(Rad(scale))),
            CameraRotationDir::Downward => self.rotate(Matrix4::from_angle_x(Rad(-scale))),
            CameraRotationDir::Clockwise => self.rotate(Matrix4::from_angle_z(Rad(scale))),
            CameraRotationDir::Counterclockwise => self.rotate(Matrix4::from_angle_z(Rad(-scale))),
        }
    }

    pub fn setscreen(&mut self, screen_x: u32, screen_y: u32) -> () {
        self.screen_x = screen_x;
        self.screen_y = screen_y;
        self.genprojection();
    }

    fn rotate_by_camera(&self, vec: Vector3<f32>) -> Vector3<f32> {
        let newvec = self.rotation * Vector4::new(vec.x, vec.y, vec.z, 1.0);
        Vector3::new(newvec.x, newvec.y, newvec.z)
    }

    fn genview(&mut self) -> () {
        // Look at the place in front of us
        self.view = Matrix4::look_at(
            self.loc,
            self.loc + self.rotate_by_camera(Vector3::unit_z()),
            self.worldup,
        );
    }

    fn genmodel(&mut self) -> () {
        self.model = Matrix4::one()
    }

    fn genprojection(&mut self) -> () {
        let aspect_ratio = self.screen_x as f32 / self.screen_y as f32;
        self.projection =
            cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
    }
}
