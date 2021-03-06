use std::ptr;
// use std::rand::RngUtil;
use nalgebra::na::{Transformation, RotationWithTranslation, CrossMatrix, Row};
use nalgebra::na;
use detection::constraint::{Constraint, RBRB, BallInSocket, Fixed};
use object::Body;
use resolution::constraint::velocity_constraint::VelocityConstraint;
use resolution::constraint::contact_equation;
use resolution::constraint::contact_equation::{CorrectionMode, CorrectionParameters};
use resolution::constraint::ball_in_socket_equation;
use resolution::constraint::fixed_equation;
use resolution::solver::Solver;
use pgs = resolution::constraint::projected_gauss_seidel_solver;
use resolution::constraint::projected_gauss_seidel_solver::Velocities;
use resolution::constraint::impulse_cache::ImpulseCache;
use aliases::traits::{NPhysicsScalar, NPhysicsDirection, NPhysicsOrientation, NPhysicsTransform,
                      NPhysicsInertia};


pub struct AccumulatedImpulseSolver<N, LV, AV, M, II, M2> {
    priv correction:              CorrectionParameters<N>,
    priv cache:                   ImpulseCache<N, LV>,
    priv num_first_order_iter:    uint,
    priv num_second_order_iter:   uint,
    priv restitution_constraints: ~[VelocityConstraint<LV, AV, N>],
    priv friction_constraints:    ~[VelocityConstraint<LV, AV, N>],
    priv MJLambda:                ~[Velocities<LV, AV>]
}

impl<N:  'static + Clone + NPhysicsScalar,
     LV: 'static + Clone + NPhysicsDirection<N, AV> + CrossMatrix<M2>,
     AV: 'static + Clone + NPhysicsOrientation<N>,
     M:  'static + Clone + NPhysicsTransform<LV, AV>,
     II: 'static + Clone + NPhysicsInertia<N, LV, AV, M>,
     M2:  Row<AV>>
AccumulatedImpulseSolver<N, LV, AV, M, II, M2> {
    pub fn new(step:                  N,
               correction_mode:       CorrectionMode<N>,
               joint_corr_factor:     N,
               rest_eps:              N,
               num_first_order_iter:  uint,
               num_second_order_iter: uint)
               -> AccumulatedImpulseSolver<N, LV, AV, M, II, M2> {
        AccumulatedImpulseSolver {
            num_first_order_iter:    num_first_order_iter,
            num_second_order_iter:   num_second_order_iter,
            restitution_constraints: ~[],
            friction_constraints:    ~[],
            MJLambda:                ~[],
            cache:                   ImpulseCache::new(step, na::dim::<LV>()),

            correction: CorrectionParameters {
                corr_mode:  correction_mode,
                joint_corr: joint_corr_factor,
                rest_eps:   rest_eps
            }
        }
    }

    fn resize_buffers(&mut self, num_restitution_equations: uint, num_friction_equations: uint) {
        resize_buffer(&mut self.restitution_constraints,
                      num_restitution_equations,
                      VelocityConstraint::new());

        resize_buffer(&mut self.friction_constraints,
                      num_friction_equations,
                      VelocityConstraint::new());
    }

    fn do_solve(&mut self,
                dt:          N,
                constraints: &[Constraint<N, LV, AV, M, II>],
                joints:      &[uint],
                bodies:      &[@mut Body<N, LV, AV, M, II>]) {
        let num_friction_equations    = (na::dim::<LV>() - 1) * self.cache.len();
        let num_restitution_equations = self.cache.len();
        let mut num_joint_equations = 0;

        for i in joints.iter() {
            match constraints[*i] {
                BallInSocket(_) => {
                    num_joint_equations = num_joint_equations + na::dim::<LV>()
                },
                Fixed(_) => {
                    num_joint_equations = num_joint_equations + na::dim::<LV>() + na::dim::<AV>()
                },
                RBRB(_, _, _) => { }
            }
        }

        self.resize_buffers(num_restitution_equations + num_joint_equations, num_friction_equations);

        let mut friction_offset = 0;

        for (i, (_, &(ci, imp))) in self.cache.hash().iter().enumerate() {
            match constraints[ci] {
                RBRB(rb1, rb2, ref c) => {
                    contact_equation::fill_second_order_equation(
                        dt.clone(),
                        c,
                        rb1.to_rigid_body_or_fail(), rb2.to_rigid_body_or_fail(),
                        &mut self.restitution_constraints[i],
                        i,
                        self.friction_constraints,
                        friction_offset,
                        self.cache.impulsions_at(imp),
                        &self.correction);
                },
                _ => { }
            }

            friction_offset = friction_offset + na::dim::<LV>() - 1;
        }

        let mut joint_offset = num_restitution_equations;
        for i in joints.iter() {
            match constraints[*i] {
                BallInSocket(bis) => {
                    ball_in_socket_equation::fill_second_order_equation(
                        dt.clone(),
                        bis,
                        self.restitution_constraints.mut_slice_from(joint_offset), // XXX
                        &self.correction
                    );

                    joint_offset = joint_offset + na::dim::<LV>();
                },
                Fixed(f) => {
                    fixed_equation::fill_second_order_equation(
                        dt.clone(),
                        f,
                        self.restitution_constraints.mut_slice_from(joint_offset), // XXX
                        &self.correction
                    );

                    joint_offset = joint_offset + na::dim::<LV>() + na::dim::<AV>();
                },
                RBRB(_, _, _) => { }
            }
        }

        resize_buffer(&mut self.MJLambda, bodies.len(), Velocities::new());

        // FIXME: parametrize by the resolution algorithm?
        pgs::projected_gauss_seidel_solve(
            self.restitution_constraints,
            self.friction_constraints,
            self.MJLambda,
            bodies.len(),
            self.num_second_order_iter,
            false);

        // FIXME: this is _so_ ugly!
        self.resize_buffers(num_restitution_equations, num_friction_equations);

        for b in bodies.iter() {
            let rb = b.to_mut_rigid_body_or_fail();
            let i  = rb.index();

            let curr_lin_vel = rb.lin_vel();
            let curr_ang_vel = rb.ang_vel();

            rb.set_lin_vel(curr_lin_vel + self.MJLambda[i].lv);
            rb.set_ang_vel(curr_ang_vel + self.MJLambda[i].av);
        }

        for (i, dv) in self.restitution_constraints.iter().enumerate() {
            let imps = self.cache.push_impulsions();
            imps[0]  = dv.impulse * na::cast(0.85);

            for j in range(0u, na::dim::<LV>() - 1) {
                let fc = &self.friction_constraints[i * (na::dim::<LV>() - 1) + j];
                imps[1 + j] = fc.impulse * na::cast(0.85);
            }
        }

        let offset = self.cache.reserved_impulse_offset();
        for (i, (_, kv)) in self.cache.hash_mut().mut_iter().enumerate() {
            *kv = (kv.first(), offset + i * na::dim::<LV>());
        }

        /*
         * first order resolution
         */
        let needs_correction = !self.correction.corr_mode.pos_corr_factor().is_zero() &&
            constraints.iter().any(|constraint| {
            match *constraint {
                RBRB(_, _, ref c) => c.depth >= self.correction.corr_mode.min_depth_for_pos_corr(),
                _ => false // no first order resolution for joints
            }
        });

        if needs_correction {
            self.resize_buffers(num_restitution_equations, num_friction_equations);

            for (i, (_, &(ci, _))) in self.cache.hash().iter().enumerate() {
                match constraints[ci] {
                    RBRB(_, _, ref c) => {
                        contact_equation::reinit_to_first_order_equation(
                            dt.clone(),
                            c,
                            &mut self.restitution_constraints[i],
                            &self.correction);
                    },
                    _ => { }
                }
            }

            // FIXME: parametrize by the resolution algorithm?
            pgs::projected_gauss_seidel_solve(
                self.restitution_constraints,
                [],
                self.MJLambda,
                bodies.len(),
                self.num_first_order_iter,
                true);

            for b in bodies.iter() {
                let rb = b.to_mut_rigid_body_or_fail();
                let i  = rb.index();

                let translation = self.MJLambda[i].lv * dt;
                let rotation    = self.MJLambda[i].av * dt;

                let center = &rb.center_of_mass().clone();

                let mut delta: M = na::one();
                delta.append_rotation_wrt_point(&rotation, center);
                delta.append_translation(&translation);

                rb.append_transformation(&delta);
            }
        }
    }
}

impl<N:  'static + Clone + NPhysicsScalar,
     LV: 'static + Clone + NPhysicsDirection<N, AV> + CrossMatrix<M2>,
     AV: 'static + Clone + NPhysicsOrientation<N>,
     M:  'static + Clone + NPhysicsTransform<LV, AV>,
     II: 'static + Clone + NPhysicsInertia<N, LV, AV, M>,
     M2: Row<AV>>
Solver<N, Constraint<N, LV, AV, M, II>> for
AccumulatedImpulseSolver<N, LV, AV, M, II, M2> {
    fn solve(&mut self, dt: N, constraints: &[Constraint<N, LV, AV, M, II>]) {
        // FIXME: bodies index assignment is very ugly
        let mut bodies = ~[];

        if constraints.len() != 0 {
            /*
             * Associate the constraints with the cached impulse.
             */
            for (i, cstr) in constraints.iter().enumerate() {
                match *cstr {
                    RBRB(a, b, ref c) => {
                        self.cache.insert(i,
                                          ptr::to_mut_unsafe_ptr(a) as uint,
                                          ptr::to_mut_unsafe_ptr(b) as uint,
                                          (c.world1 + c.world2) / na::cast(2.0));
                    },
                    BallInSocket(_) => {
                        // XXX: cache for ball in socket?
                    },
                    Fixed(_) => {
                        // XXX: cache for fixed?
                    }
                }
            }

            /*
             * Assign an index to each body.
             */
            // This is a two-passes assignation of index to the rigid bodies.
            // This is not very good, but is the only way to do that without having a separate list
            // of all rigid bodies.
            for c in constraints.iter() {
                match *c {
                    RBRB(a, b, _) => {
                        a.set_index(-2);
                        b.set_index(-2)
                    },
                    BallInSocket(bis) => {
                        match bis.anchor1().body {
                            Some(b) => b.set_index(-2),
                            None    => { }
                        };

                        match bis.anchor2().body {
                            Some(b) => b.set_index(-2),
                            None    => { }
                        }
                    }
                    Fixed(f) => { // FIXME: code duplication from BallInSocket
                        match f.anchor1().body {
                            Some(b) => b.set_index(-2),
                            None    => { }
                        };

                        match f.anchor2().body {
                            Some(b) => b.set_index(-2),
                            None    => { }
                        }
                    }
                }
            }

            let mut id = 0;

            fn set_body_index<N:  Clone + NPhysicsScalar,
                              LV: Clone + NPhysicsDirection<N, AV>,
                              AV: Clone + NPhysicsOrientation<N>,
                              M:  NPhysicsTransform<LV, AV>,
                              II: Clone + NPhysicsInertia<N, LV, AV, M>>(
                              a:      @mut Body<N, LV, AV, M, II>,
                              bodies: &mut ~[@mut Body<N, LV, AV, M, II>],
                              id:     &mut int) {
                if a.index() == -2 {
                    if a.can_move() {
                        a.set_index(*id);
                        bodies.push(a);
                        *id = *id + 1;
                    }
                    else {
                        a.set_index(-1)
                    }
                }
            }

            // FIXME: avoid allocation
            let mut joints = ~[];
            for (i, c) in constraints.iter().enumerate() {
                match *c {
                    RBRB(a, b, _) => {
                        set_body_index(a, &mut bodies, &mut id);
                        set_body_index(b, &mut bodies, &mut id);
                    },
                    BallInSocket(bis) => {
                        joints.push(i);
                        match bis.anchor1().body {
                            Some(b) => set_body_index(b, &mut bodies, &mut id),
                            None => { }
                        }

                        match bis.anchor2().body {
                            Some(b) => set_body_index(b, &mut bodies, &mut id),
                            None => { }
                        }
                    },
                    Fixed(f) => { // FIXME: code duplication from BallInSocket
                        joints.push(i);
                        match f.anchor1().body {
                            Some(b) => set_body_index(b, &mut bodies, &mut id),
                            None => { }
                        }

                        match f.anchor2().body {
                            Some(b) => set_body_index(b, &mut bodies, &mut id),
                            None => { }
                        }
                    }
                }
            }

            self.do_solve(dt.clone(), constraints, joints, bodies);
            self.cache.swap();
        }
    }

    #[inline]
    fn priority(&self) -> f64 { 0.0 }
}

fn resize_buffer<A: Clone>(buff: &mut ~[A], size: uint, val: A) {
    if buff.len() < size {
        buff.grow_set(size - 1, &val, val.clone());
    }
    else {
        buff.truncate(size)
    }
}
