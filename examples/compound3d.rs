#[crate_type = "bin"];
#[warn(non_camel_case_types)];
#[feature(managed_boxes)];

extern mod std;
extern mod extra;
extern mod kiss3d;
extern mod graphics3d;
extern mod nphysics;
extern mod nalgebra;
extern mod ncollide;

use std::rc::Rc;
use kiss3d::window::Window;
use nalgebra::na::{Vec3, Iso3, Translation};
use nalgebra::na;
use ncollide::geom::{Plane, Box, Compound};
use nphysics::world::BodyWorld;
use nphysics::aliases::dim3;
use nphysics::object::{RigidBody, Static, Dynamic, RB};
use graphics3d::engine::GraphicsManager;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    GraphicsManager::simulate(compound_3d)
}

pub fn compound_3d(window: &mut Window, graphics: &mut GraphicsManager) -> dim3::BodyWorld3d<f32> {
    /*
     * World
     */
    let mut world = BodyWorld::new();
    world.set_gravity(Vec3::new(0.0f32, -9.81, 0.0));

    /*
     * Planes
     */
    let rb   = RigidBody::new(Plane::new(Vec3::new(0.0f32, 1.0, 0.0)), 0.0, Static, 0.3, 0.6);
    let body = @mut RB(rb);

    world.add_body(body);
    graphics.add(window, body);

    /*
     * Cross shaped geometry
     */

    let delta1 = Iso3::new(Vec3::new(0.0f32, -5.0, 0.0), na::zero());
    let delta2 = Iso3::new(Vec3::new(-5.0f32, 0.0, 0.0), na::zero());
    let delta3 = Iso3::new(Vec3::new(5.0f32, 0.0, 0.0), na::zero());

    let mut cross_geoms = ~[];
    cross_geoms.push((delta1, ~Box::new(Vec3::new(5.0f32, 0.25, 0.25)) as dim3::Geom3d<f32>));
    cross_geoms.push((delta2, ~Box::new(Vec3::new(0.25f32, 5.0, 0.25)) as dim3::Geom3d<f32>));
    cross_geoms.push((delta3, ~Box::new(Vec3::new(0.25f32, 5.0, 0.25)) as dim3::Geom3d<f32>));

    let cross = Rc::from_send(~Compound::new(cross_geoms) as dim3::Geom3d<f32>);

    /*
     * Create the crosses 
     */
    let num     = 6;
    let rad     = 5.0;
    let shift   = rad * 2.0;
    let centerx = shift * (num as f32) / 2.0;
    let centery = 30.0 + shift / 2.0;
    let centerz = shift * (num as f32) / 2.0;

    for i in range(0u, num) {
        for j in range(0u, num) {
            for k in range(0u, num) {
                let x = i as f32 * shift - centerx;
                let y = j as f32 * shift + centery;
                let z = k as f32 * shift - centerz;

                let mut rb = RigidBody::new_with_shared_geom(cross.clone(), 1.0f32, Dynamic, 0.3, 0.5);

                rb.append_translation(&Vec3::new(x, y, z));

                let body = @mut RB(rb);

                world.add_body(body);
                graphics.add(window, body);
            }
        }
    }

    /*
     * Set up the camera and that is it!
     */
    graphics.look_at(Vec3::new(-30.0, 30.0, -30.0), Vec3::new(0.0, 0.0, 0.0));

    world
}
