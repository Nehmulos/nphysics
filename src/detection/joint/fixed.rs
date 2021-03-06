use detection::joint::anchor::Anchor;
use object::{RB, SB};
use aliases::traits::{NPhysicsScalar, NPhysicsDirection, NPhysicsOrientation, NPhysicsTransform,
                      NPhysicsInertia};

pub struct Fixed<N, LV, AV, M, II> {
    priv up_to_date: bool,
    priv anchor1:    Anchor<N, LV, AV, M, II, M>,
    priv anchor2:    Anchor<N, LV, AV, M, II, M>,
}

impl<N:  NPhysicsScalar,
     LV: Clone + NPhysicsDirection<N, AV>,
     AV: Clone + NPhysicsOrientation<N>,
     M:  Clone + NPhysicsTransform<LV, AV>,
     II: Clone + NPhysicsInertia<N, LV, AV, M>>
Fixed<N, LV, AV, M, II> {
    pub fn new(anchor1: Anchor<N, LV, AV, M, II, M>,
               anchor2: Anchor<N, LV, AV, M, II, M>)
               -> Fixed<N, LV, AV, M, II> {
        Fixed {
            up_to_date: false,
            anchor1:    anchor1,
            anchor2:    anchor2
        }
    }

    pub fn up_to_date(&self) -> bool {
        self.up_to_date
    }

    pub fn update(&mut self) {
        self.up_to_date = true
    }

    pub fn anchor1<'r>(&'r self) -> &'r Anchor<N, LV, AV, M, II, M> {
        &self.anchor1
    }

    pub fn anchor2<'r>(&'r self) -> &'r Anchor<N, LV, AV, M, II, M> {
        &self.anchor2
    }

    pub fn set_local1(&mut self, local1: M) {
        if local1 != self.anchor1.position {
            self.up_to_date = false;
            self.anchor1.position = local1
        }
    }

    pub fn set_local2(&mut self, local2: M) {
        if local2 != self.anchor2.position {
            self.up_to_date = false;
            self.anchor2.position = local2
        }
    }

    pub fn anchor1_pos(&self) -> M {
        match self.anchor1.body {
            Some(b) => {
                match *b {
                    RB(ref rb) => rb.transform_ref() * self.anchor1.position,
                    SB(_)      => fail!("Not yet implemented.")
                }
            },
            None => self.anchor1.position.clone()
        }
    }

    pub fn anchor2_pos(&self) -> M {
        match self.anchor2.body {
            Some(b) => {
                match *b {
                    RB(ref rb) => rb.transform_ref() * self.anchor2.position,
                    SB(_)      => fail!("Not yet implemented.")
                }
            },
            None => self.anchor2.position.clone()
        }
    }
}
