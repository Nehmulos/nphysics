use std::num::One;
use nalgebra::na::{Vec, Translation, Rotation, RotationWithTranslation};
use nalgebra::na;

pub fn explicit_integrate<M:  Translation<LV> + Rotation<AV> + One,
                          LV: Vec<N>,
                          AV: Vec<N>,
                          N:  Clone>(
                          dt: N,
                          p:  &M,
                          c:  &LV,
                          lv: &LV,
                          av: &AV,
                          lf: &LV,
                          af: &AV)
                          -> (M, LV, AV) {
    (
        displacement(dt.clone(), p, c, lv, av), 
        integrate(dt.clone(), lv, lf),
        integrate(dt, av, af)
    )
}

pub fn explicit_integrate_wo_rotation<V: Vec<N>,
                                      N: Clone>(
                                      dt: N,
                                      p:  &V,
                                      lv: &V,
                                      lf: &V)
                                      -> (V, V) {
    (
        integrate(dt.clone(), p, lv), 
        integrate(dt, lv, lf)
    )
}

pub fn semi_implicit_integrate<M:  Translation<LV> + Rotation<AV> + One,
                               LV: Vec<N>,
                               AV: Vec<N>,
                               N:  Clone>(
                               dt: N,
                               p:  &M,
                               c:  &LV,
                               lv: &LV,
                               av: &AV,
                               lf: &LV,
                               af: &AV)
                               -> (M, LV, AV) {
    let nlv = integrate(dt.clone(), lv, lf);
    let nav = integrate(dt.clone(), av, af);

    (
        displacement(dt.clone(), p, c, &nlv, &nav),
        nlv,
        nav
    )
}

pub fn semi_implicit_integrate_wo_rotation<V: Vec<N>,
                                           N: Clone>(
                                           dt: N,
                                           p:  &V,
                                           lv: &V,
                                           lf: &V)
                                           -> (V, V) {
    let nlv = integrate(dt.clone(), lv, lf);

    (
        integrate(dt.clone(), p, &nlv), 
        nlv
    )
}

// fn implicit_integrate<>()
// {
//    FIXME
// }

pub fn displacement<M: RotationWithTranslation<LV, AV> + One,
                    LV: Vec<N>,
                    AV: Vec<N>,
                    N>(
                    dt:             N,
                    _:              &M,
                    center_of_mass: &LV,
                    lin_vel:        &LV,
                    ang_vel:        &AV)
                    -> M {
    let mut res: M = na::one();
    res.append_rotation_wrt_point(&(ang_vel * dt), center_of_mass);

    res.append_translation(&(lin_vel * dt));

    res
}

#[inline]
fn integrate<N, V: Vec<N>>(dt: N, v: &V, f: &V) -> V {
    v + f * dt
}
