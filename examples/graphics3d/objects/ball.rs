use kiss3d::window::Window;
use kiss3d::object::Object;
use nalgebra::na::{Vec3, Transformation};
use nalgebra::na;
use nphysics::aliases::dim3;
use engine::SceneNode;

pub struct Ball {
    priv color:      Vec3<f32>,
    priv base_color: Vec3<f32>,
    priv delta:      dim3::Transform3d<f32>,
    priv gfx:        Object,
    priv body:       @mut dim3::Body3d<f32>
}

impl Ball {
    pub fn new(body:   @mut dim3::Body3d<f32>,
               delta:  dim3::Transform3d<f32>,
               radius: f32,
               color:  Vec3<f32>,
               window: &mut Window) -> Ball {
        let mut res = Ball {
            color:      color,
            base_color: color,
            delta:      delta,
            gfx:        window.add_sphere(radius as f32),
            body:       body
        };

        res.gfx.set_color(color.x, color.y, color.z);
        res.update();

        res
    }
}

impl SceneNode for Ball {
    fn select(&mut self) {
        self.color = Vec3::x();
    }

    fn unselect(&mut self) {
        self.color = self.base_color;
    }

    fn update(&mut self) {
        let rb = self.body.to_rigid_body_or_fail();
        if rb.is_active() {
            {
                self.gfx.set_transformation(na::transformation(rb) * self.delta);
            }

            self.gfx.set_color(self.color.x, self.color.y, self.color.z);
        }
        else {
            self.gfx.set_color(self.color.x * 0.25, self.color.y * 0.25, self.color.z * 0.25);
        }
    }

    fn object<'r>(&'r self) -> &'r Object {
        &'r self.gfx
    }
}
