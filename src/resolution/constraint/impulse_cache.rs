use std::util;
use std::rand::{IsaacRng, Rng};
use std::num::Zero;
use std::vec;
use nalgebra::na::Vec;
use nalgebra::na;
use std::hashmap::HashMap;

#[deriving(Eq)]
struct ContactIdentifier<V> {
    obj1:    uint,
    obj2:    uint,
    ccenter: V
}

pub type Cb<'a> = 'a |buf: &[u8]| -> bool;

impl<V: IterBytes> IterBytes for ContactIdentifier<V> {
    #[inline]
    fn iter_bytes(&self, _lsb0: bool, f: Cb) -> bool {
        self.ccenter.iter_bytes(_lsb0, f)
    }
}

impl<V: Round + Vec<N>, N> ContactIdentifier<V> {
    pub fn new(obj1: uint, obj2: uint, center: V, step: &N) -> ContactIdentifier<V> {
        ContactIdentifier {
            obj1:    obj1,
            obj2:    obj2,
            ccenter: (center / *step).trunc()
        }
    }
}

// FIXME: make the fields priv
pub struct ImpulseCache<N, V> {
    priv hash_prev:           HashMap<ContactIdentifier<V>, (uint, uint)>,
    priv cache_prev:          ~[N],
    priv hash_next:           HashMap<ContactIdentifier<V>, (uint, uint)>,
    priv cache_next:          ~[N],
    priv step:                N,
    priv impulse_per_contact: uint
}

impl<N: Zero + Clone,
     V: Vec<N> + Round + IterBytes>
ImpulseCache<N, V> {
    pub fn new(step: N, impulse_per_contact: uint) -> ImpulseCache<N, V> {
        let mut rng = IsaacRng::new_unseeded();

        ImpulseCache {
            hash_prev:           HashMap::with_capacity_and_keys(rng.gen(), rng.gen(), 32),
            hash_next:           HashMap::with_capacity_and_keys(rng.gen(), rng.gen(), 32),
            cache_prev:          vec::from_elem(impulse_per_contact, na::zero()),
            cache_next:          vec::from_elem(impulse_per_contact, na::zero()),
            step:                step,
            impulse_per_contact: impulse_per_contact
        }
    }

    pub fn insert<'a>(&'a mut self, cid: uint, obj1: uint, obj2: uint, center: V) {
        let id = ContactIdentifier::new(obj1, obj2, center, &self.step);
        let imp =
            match self.hash_prev.find_copy(&id) {
                Some((_, i)) => i,
                None         => 0
            };

        self.hash_next.insert(id, (cid, imp));
    }

    pub fn hash<'a>(&'a self) -> &'a HashMap<ContactIdentifier<V>, (uint, uint)> {
        &'a self.hash_next
    }

    pub fn hash_mut<'a>(&'a mut self) -> &'a mut HashMap<ContactIdentifier<V>, (uint, uint)> {
        &'a mut self.hash_next
    }

    pub fn push_impulsions<'a>(&'a mut self) -> &'a mut [N] {
        let begin = self.cache_next.len();

        self.impulse_per_contact.times(|| {
            self.cache_next.push(na::zero());
        });

        let end = self.cache_next.len();

        self.cache_next.mut_slice(begin, end)
    }

    pub fn reserved_impulse_offset(&self) -> uint {
        self.impulse_per_contact
    }

    pub fn impulsions_at<'a>(&'a self, at: uint) -> &'a [N] {
        self.cache_prev.slice(at, at + self.impulse_per_contact)
    }

    pub fn len(&self) -> uint {
        self.hash_next.len()
    }

    pub fn clear(&mut self) {
        self.cache_prev.clear();
        self.hash_prev.clear();
        self.cache_next.clear();
        self.hash_next.clear();

        self.cache_prev.grow_set(self.impulse_per_contact, &na::zero(), na::zero());
        self.cache_next.grow_set(self.impulse_per_contact, &na::zero(), na::zero());
    }

    pub fn swap(&mut self) {
        util::swap(&mut self.hash_prev, &mut self.hash_next);
        util::swap(&mut self.cache_prev,&mut self.cache_next);
        self.hash_next.clear();
        self.cache_next.truncate(self.impulse_per_contact);
    }
}
