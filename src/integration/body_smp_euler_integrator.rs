use std::ptr;
use nalgebra::na::Transformation;
use ncollide::util::hash_map::HashMap;
use ncollide::util::hash::UintTWHash;
use object::{RB, SB};
use object::Body;
use integration::Integrator;
use integration::euler;
use signal::signal::{SignalEmiter, BodyActivationSignalHandler};
use aliases::traits::{NPhysicsScalar, NPhysicsDirection, NPhysicsOrientation, NPhysicsTransform, NPhysicsInertia};

pub struct BodySmpEulerIntegrator<N, LV, AV, M, II> {
    priv objects: HashMap<uint, @mut Body<N, LV, AV, M, II>, UintTWHash>,
}

impl<N:  'static + Clone + NPhysicsScalar,
     LV: 'static + Clone + NPhysicsDirection<N, AV>,
     AV: 'static + Clone + NPhysicsOrientation<N>,
     M:  'static + Clone + NPhysicsTransform<LV, AV>,
     II: 'static + Clone + NPhysicsInertia<N, LV, AV, M>>
BodySmpEulerIntegrator<N, LV, AV, M, II> {
    #[inline]
    pub fn new<C>(events: &mut SignalEmiter<N, Body<N, LV, AV, M, II>, C>)
                  -> @mut BodySmpEulerIntegrator<N, LV, AV, M, II> {
        let res = @mut BodySmpEulerIntegrator {
            objects: HashMap::new(UintTWHash::new())
        };

        events.add_body_activation_handler(
            ptr::to_mut_unsafe_ptr(res) as uint,
            res as @mut BodyActivationSignalHandler<Body<N, LV, AV, M, II>, C>
        );

        res
    }
}

impl<N:  Clone + NPhysicsScalar,
     LV: Clone + NPhysicsDirection<N, AV>,
     AV: Clone + NPhysicsOrientation<N>,
     M:  Clone + NPhysicsTransform<LV, AV>,
     II: Clone + NPhysicsInertia<N, LV, AV, M>>
Integrator<N, Body<N, LV, AV, M, II>> for BodySmpEulerIntegrator<N, LV, AV, M, II> {
    #[inline]
    fn add(&mut self, o: @mut Body<N, LV, AV, M, II>) {
        self.objects.insert(ptr::to_mut_unsafe_ptr(o) as uint, o);
    }

    #[inline]
    fn remove(&mut self, o: @mut Body<N, LV, AV, M, II>) {
        self.objects.remove(&(ptr::to_mut_unsafe_ptr(o) as uint));
    }

    #[inline]
    fn update(&mut self, dt: N) {
        for o in self.objects.elements().iter() {
            match *o.value {
                RB(ref mut rb) => {
                    if rb.can_move() {
                        let (t, lv, av) = euler::semi_implicit_integrate(
                            dt.clone(),
                            rb.transform_ref(),
                            rb.center_of_mass(),
                            &rb.lin_vel(),
                            &rb.ang_vel(),
                            &rb.lin_acc(),
                            &rb.ang_acc());

                        rb.append_transformation(&t);
                        rb.set_lin_vel(lv);
                        rb.set_ang_vel(av);
                    }
                },
                SB(_) => fail!("Not yet implemented.")
            }
        }
    }

    #[inline]
    fn priority(&self) -> f64 { 50.0 }
}

impl<N:  Clone + NPhysicsScalar,
     LV: Clone + NPhysicsDirection<N, AV>,
     AV: Clone + NPhysicsOrientation<N>,
     M:  Clone + NPhysicsTransform<LV, AV>,
     II: Clone + NPhysicsInertia<N, LV, AV, M>,
     C>
BodyActivationSignalHandler<Body<N, LV, AV, M, II>, C> for BodySmpEulerIntegrator<N, LV, AV, M, II> {
    fn handle_body_activated_signal(&mut self, b: @mut Body<N, LV, AV, M, II>, _: &mut ~[C]) {
        self.add(b)
    }

    fn handle_body_deactivated_signal(&mut self, b: @mut Body<N, LV, AV, M, II>) {
        self.remove(b)
    }
}
