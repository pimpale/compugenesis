use cgmath::{Angle, Deg, InnerSpace, Matrix4, Point3, Rad, Vector3};

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

// vectors giving the current perception of the camera
#[derive(Clone, Debug)]
struct DirVecs {
    // NOTE: front is actually backwards
    front: Vector3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,
}

#[derive(Clone, Debug)]
pub struct Camera {
    // screen dimensions
    screen_x: u32,
    screen_y: u32,

    // global camera position
    loc: Point3<f32>,
    // the global up
    worldup: Vector3<f32>,

    // pitch and yaw values
    pitch: Rad<f32>,
    yaw: Rad<f32>,

    // relative directions
    dirs: DirVecs,

    // Projection Matrix
    projection: Matrix4<f32>,
}

impl DirVecs {
    fn new(worldup: Vector3<f32>, pitch: Rad<f32>, yaw: Rad<f32>) -> DirVecs {
        let front = Vector3 {
            x: yaw.cos() * pitch.cos(),
            y: pitch.sin(),
            z: yaw.sin() * pitch.cos(),
        }
        .normalize();
        // get other vectors
        let right = front.cross(worldup).normalize();
        let up = right.cross(front).normalize();
        // return values
        DirVecs { front, right, up }
    }
}

impl Camera {
    pub fn new(location: Point3<f32>, screen_x: u32, screen_y: u32) -> Camera {
        let pitch = Rad::from(Deg(0.0));
        let yaw = Rad::from(Deg(-90.0));

        let worldup = Vector3::unit_y();

        Camera {
            screen_x,
            screen_y,
            loc: location,
            worldup,
            pitch,
            yaw,
            dirs: DirVecs::new(worldup, pitch, yaw),
            projection: Camera::gen_projection(screen_x, screen_y),
        }
    }

    pub fn mvp(&self) -> Matrix4<f32> {

        let view = Matrix4::look_at_rh(self.loc, self.loc - self.dirs.front, self.worldup);
        self.projection * view
    }

    pub fn translate(&mut self, delta: Vector3<f32>) {
        self.loc = self.loc + delta;
    }

    pub fn dir_move(&mut self, dir: CameraMovementDir) {
        let scale = 0.1;
        self.translate(match dir {
            CameraMovementDir::Forward => -self.dirs.front * scale,
            CameraMovementDir::Backward => self.dirs.front * scale,
            CameraMovementDir::Right => -self.dirs.right * scale,
            CameraMovementDir::Left => self.dirs.right * scale,
            CameraMovementDir::Upward => self.dirs.up * scale,
            CameraMovementDir::Downward => -self.dirs.up * scale,
        });
    }

    pub fn dir_rotate(&mut self, dir: CameraRotationDir) {
        let rotval = Rad(0.05);

        match dir {
            CameraRotationDir::Left => self.yaw -= rotval,
            CameraRotationDir::Right => self.yaw += rotval,
            CameraRotationDir::Upward => self.pitch += rotval,
            CameraRotationDir::Downward => self.pitch -= rotval,
        }

        if self.pitch > Deg(89.0).into() {
            self.pitch = Deg(89.0).into();
        } else if self.pitch < Deg(-89.0).into() {
            self.pitch = Deg(-89.0).into();
        }
        // recalculate camera directions
        self.dirs = DirVecs::new(self.worldup, self.pitch, self.yaw);
    }

    pub fn set_screen(&mut self, screen_x: u32, screen_y: u32) {
        self.screen_x = screen_x;
        self.screen_y = screen_y;
        self.projection = Camera::gen_projection(screen_x, screen_y);
    }

    fn gen_projection(screen_x: u32, screen_y: u32) -> Matrix4<f32> {
        let aspect_ratio = screen_x as f32 / screen_y as f32;
        cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0)
    }
}
