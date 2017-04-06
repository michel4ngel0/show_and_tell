use cgmath::*;
use std::f64::consts::E;

const INITIAL_DIRECTION: Vector3<f32> = Vector3 {
        x:  0.0,
        y:  0.0,
        z: -1.0,
    };
const INITIAL_UP: Vector3<f32> = Vector3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

pub struct Camera {
    position: Point3<f32>,
    rotation: Quaternion<f32>,
}

impl Camera {
    pub fn new(position: Point3<f32>) -> Camera {
        Camera {
            position: position,
            rotation: Quaternion::<f32>::one(),
        }
    }

    pub fn get_matrix(&self) -> Matrix4<f32> {
        let direction = self.rotation * INITIAL_DIRECTION;
        let up = self.rotation * INITIAL_UP;

        Matrix4::<f32>::look_at(
            self.position,
            self.position + direction,
            up
        )
    }

    pub fn step(&mut self, shift: Vector2<f32>) {
        let speed = 0.001 * (self.position.z + 1.0);
        let direction = self.rotation * vec3(shift.x, shift.y, 1.0);

        self.position.x -= speed * direction.x;
        self.position.y -= speed * direction.y;
    }

    pub fn turn_around(&mut self, turn: Vector2<f32>) {
        // let speed = 0.1;
        //
        // let pitch = speed * turn.y;
        // let yaw = speed * turn.x;
        // let roll = 0.0;
        //
        // let rotation = Quaternion::from(Euler {
        //     x: Deg(pitch),
        //     y: Deg(yaw),
        //     z: Deg(roll),
        // });
        //
        // let rotation = Quaternion::<f32>::from(rotation);
        //
        // self.rotation = self.rotation * rotation;
    }

    pub fn zoom(&mut self, delta: f32) {
        let speed = 0.5 * self.get_speed() * delta;
        let forward_vec = self.rotation * INITIAL_DIRECTION;

        self.position += speed * forward_vec;
    }

    fn get_speed(&self) -> f32 {
        let height = self.position.z;
        let mut result = if height <= 10.0 { 1.0 } else { (E as f32).powf((height - 10.0) * 0.02) };
        if result >= 1000.0 { result = 1000.0 }
        result
    }
}
