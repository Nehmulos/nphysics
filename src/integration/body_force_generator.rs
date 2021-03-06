use std::ptr;
use ncollide::util::hash_map::HashMap;
use ncollide::util::hash::UintTWHash;
use object::{Body, RB, SB};
use integration::Integrator;
use signal::signal::{SignalEmiter, BodyActivationSignalHandler};
use aliases::traits::{NPhysicsScalar, NPhysicsDirection, NPhysicsOrientation, NPhysicsTransform, NPhysicsInertia};

// FIXME: split this on `RigidBodyForceGenerator` and `SoftBodyForceGenerator` ?
pub struct BodyForceGenerator<N, LV, AV, M, II> {
    priv objects: HashMap<uint, @mut Body<N, LV, AV, M, II>, UintTWHash>,
    priv lin_acc: LV,
    priv ang_acc: AV
}

impl<N:  'static + Clone + NPhysicsScalar,
     LV: 'static + Clone + NPhysicsDirection<N, AV>,
     AV: 'static + Clone + NPhysicsOrientation<N>,
     M:  'static + Clone + NPhysicsTransform<LV, AV>,
     II: 'static + Clone + NPhysicsInertia<N, LV, AV, M>>
BodyForceGenerator<N, LV, AV, M, II> {
    pub fn new<C>(events:  &mut SignalEmiter<N, Body<N, LV, AV, M, II>, C>,
                  lin_acc: LV,
                  ang_acc: AV)
                  -> @mut BodyForceGenerator<N, LV, AV, M, II> {
        let res = @mut BodyForceGenerator {
            objects: HashMap::new(UintTWHash::new()),
            lin_acc: lin_acc,
            ang_acc: ang_acc
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
BodyForceGenerator<N, LV, AV, M, II> {
    #[inline]
    pub fn lin_acc(&self) -> LV {
        self.lin_acc.clone()
    }

    #[inline]
    pub fn set_lin_acc(&mut self, lin_acc: LV) {
        self.lin_acc = lin_acc;

        for o in self.objects.elements().iter() {
            self.write_accs_to(o.value)
        }
    }

    #[inline]
    pub fn ang_acc(&self) -> AV {
        self.ang_acc.clone()
    }

    #[inline]
    pub fn set_ang_acc(&mut self, ang_acc: AV) {
        self.ang_acc = ang_acc;

        for o in self.objects.elements().iter() {
            self.write_accs_to(o.value)
        }
    }

    #[inline]
    fn write_accs_to(&self, o: &mut Body<N, LV, AV, M, II>) {
        match *o {
            RB(ref mut rb) => {
                rb.set_lin_acc(self.lin_acc.clone());
                rb.set_ang_acc(self.ang_acc.clone());
            },
            SB(ref mut sb) => sb.acc = self.lin_acc.clone()
        }
    }
}

impl<N:  Clone + NPhysicsScalar,
     LV: Clone + NPhysicsDirection<N, AV>,
     AV: Clone + NPhysicsOrientation<N>,
     M:  Clone + NPhysicsTransform<LV, AV>,
     II: Clone + NPhysicsInertia<N, LV, AV, M>>
Integrator<N, Body<N, LV, AV, M, II>> for BodyForceGenerator<N, LV, AV, M, II> {
    #[inline]
    fn add(&mut self, o: @mut Body<N, LV, AV, M, II>) {
        self.objects.insert(ptr::to_mut_unsafe_ptr(o) as uint, o);

        self.write_accs_to(o)
    }

    #[inline]
    fn remove(&mut self, o: @mut Body<N, LV, AV, M, II>) {
        self.objects.remove(&(ptr::to_mut_unsafe_ptr(o) as uint));
    }

    #[inline]
    fn update(&mut self, _: N) { }

    #[inline]
    fn priority(&self) -> f64 { 0.0 }
}

impl<N:  Clone + NPhysicsScalar,
     LV: Clone + NPhysicsDirection<N, AV>,
     AV: Clone + NPhysicsOrientation<N>,
     M:  Clone + NPhysicsTransform<LV, AV>,
     II: Clone + NPhysicsInertia<N, LV, AV, M>,
     C>
BodyActivationSignalHandler<Body<N, LV, AV, M, II>, C> for BodyForceGenerator<N, LV, AV, M, II> {
    fn handle_body_activated_signal(&mut self, b: @mut Body<N, LV, AV, M, II>, _: &mut ~[C]) {
        self.add(b)
    }

    fn handle_body_deactivated_signal(&mut self, b: @mut Body<N, LV, AV, M, II>) {
        self.remove(b)
    }
}
