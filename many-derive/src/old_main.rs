use std::ops::Range;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Xvec3(pub i64, pub i64, pub i64);
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Xquat(pub f32, pub f32, pub f32, pub f32);

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Xform {
    pub position : Xvec3,
    pub orientation : Xquat,
    pub scale : Xvec3,
}

// Can be usize if you want
pub type XformIdx = u32;

#[derive(Debug, Clone, PartialEq)]
pub struct XformSoa {
    pub active_range : Range<XformIdx>,
    pub parented_range : Range<XformIdx>,
    pub position : Vec<Xvec3>,
    pub orientation : Vec<Xquat>,
    pub scale : Vec<Xvec3>,
}
#[derive(Debug, PartialEq)]
pub struct XformSoaMutSlice<'a> {
    pub position : &'a mut[Xvec3],
    pub orientation : &'a mut[Xquat],
    pub scale : &'a mut[Xvec3],
}
#[derive(Debug, PartialEq)]
pub struct XformSoaMutCol<'a> {
    pub position : &'a mut Xvec3,
    pub orientation : &'a mut Xquat,
    pub scale : &'a mut Xvec3,
}


// XXX Comment je colle position et scale ensemble ?
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct XformWeird_PositionAndScale {
    pub position : Xvec3,
    pub scale : Xvec3,
}
#[derive(Debug, Clone, PartialEq)]
pub struct XformWeird {
    pub active_range : Range<XformIdx>,
    pub parented_range : Range<XformIdx>,
    pub position_and_scale : Vec<XformWeird_PositionAndScale>,
    pub orientation : Vec<Xquat>,
}

impl Default for XformSoa {
    fn default() -> Self {
        Self {
            active_range : 0..0,
            parented_range : 0..0,
            position : Default::default(),
            orientation : Default::default(),
            scale : Default::default(),
        }
    }
}

impl XformSoa {
    pub fn count(&self) -> XformIdx {
        self.position.len() as XformIdx
    }
    pub fn new(cnt: XformIdx) -> Self {
        let mut out = Self::default();
        out.active_range = 0..cnt/4;
        out.parented_range = cnt/4..cnt/2;
        for i in 0..cnt {
            let i = i as i64;
            out.position.push(Xvec3(i,i,i));
        }
        for i in 0..cnt {
            let i = i as f32;
            out.orientation.push(Xquat(i,i,i,i));
        }
        for i in 0..cnt {
            let i = i as i64;
            out.scale.push(Xvec3(i,i,i));
        }
        out
    }
    pub fn all_inrange<'a>(&'a mut self, r : Range<XformIdx>) -> XformSoaMutSlice<'a> {
        let r = r.start as usize .. r.end as usize;
        XformSoaMutSlice {
            position : &mut self.position[r.clone()],
            orientation : &mut self.orientation[r.clone()],
            scale : &mut self.scale[r.clone()],
        }
    }
    pub fn all_active<'a>(&'a mut self) -> XformSoaMutSlice<'a> {
        let r = self.active_range.clone();
        self.all_inrange(r)
    }
    pub fn all_parented<'a>(&'a mut self) -> XformSoaMutSlice<'a> {
        let r = self.parented_range.clone();
        self.all_inrange(r)
    }
}

impl<'a> XformSoaMutSlice<'a> {
    pub fn count(&self) -> XformIdx {
        self.position.len() as XformIdx
    }
    pub fn mut_col(&mut self, i : XformIdx) -> XformSoaMutCol {
        let i = i as usize;
        XformSoaMutCol {
            position : &mut self.position[i],
            orientation : &mut self.orientation[i],
            scale : &mut self.scale[i],
        }
    }
    pub fn position(&mut self, i : XformIdx) -> &mut Xvec3 {
        &mut self.position[i as usize]
    }
    pub fn orientation(&mut self, i : XformIdx) -> &mut Xquat {
        &mut self.orientation[i as usize]
    }
    pub fn scale(&mut self, i : XformIdx) -> &mut Xvec3 {
        &mut self.scale[i as usize]
    }
}

use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref G_XFORMS : Mutex<XformSoa> = Mutex::new(XformSoa::new(9));
}

fn toast() {
    let mut xforms = G_XFORMS.lock().unwrap();
    let mut active = xforms.all_active();
    for i in 0..active.count() {
        let i = i as usize;
        let e = i as i64 + 64i64;
        active.position[i] = Xvec3(e,e,e);
        active.orientation[i] = Default::default();
        active.scale[i] = Xvec3(e,e,e);
    }
}
use std::thread;
use std::sync::mpsc;

fn toast_mt(thread_count : usize) {
    let (tx, rx) = mpsc::channel();
    for i in 0..thread_count {
        let tx = tx.clone();
        thread::Builder::new().name(format!("Toaster {}",i)).spawn(move || {
            toast_part(i, thread_count);
            tx.send(()).unwrap();
        }).unwrap();
    }
    for _ in 0..thread_count {
        rx.recv().unwrap();
    }
    let xforms = G_XFORMS.lock().unwrap();
    println!("{:?}", *xforms);
}
fn toast_part(thread_i : usize, thread_count : usize) {
    let mut xforms = G_XFORMS.lock().unwrap();
    let cnt = xforms.count() as usize;
    let stt;
    let end;
    // TODO use slice.chunks() instead ?
    if cnt/thread_count <= 0 {
        if thread_i >= cnt {
            return;
        }
        stt = thread_i;
        end = stt+1;
    } else {
        stt = thread_i*(cnt/thread_count);
        end = if thread_i==thread_count-1 { cnt } else { stt+cnt/thread_count };
    }
    let mut xforms = xforms.all_inrange(Range { start:stt as XformIdx, end:end as XformIdx });
    for i in 0..xforms.count() {
        let e = thread_i as i64*1000i64 + i as i64 + 64i64;
        let i = i as usize;
        xforms.position[i] = Xvec3(e,e,e);
        xforms.orientation[i] = Default::default();
        xforms.scale[i] = Xvec3(e,e,e);
    }
}

fn toast_demo() {
    let mut xforms = XformSoa::new(8);
    {
        let active = xforms.all_active();
        for p in active.position {
            *p = Xvec3(1,1,1);
        }
    }
    {
        let mut active = xforms.all_active();
        for i in 0..active.count() {
            let e = i as i64 + 64i64;
            let i = i as usize;
            active.position[i] = Xvec3(e,e,e);
            active.orientation[i] = Default::default();
            active.scale[i] = Xvec3(e,e,e);
        }
    }
    {
        let mut active = xforms.all_active();
        for i in 0..active.count() {
            let e = i as i64 + 64i64;
            *active.position(i) = Xvec3(e,e,e);
            *active.orientation(i) = Default::default();
            *active.scale(i) = Xvec3(e,e,e);
        }
    }
    {
        let mut pted = xforms.all_parented();
        for i in 0..pted.count() {
            let e = i as i64 + 52i64;
            let p = pted.mut_col(i);
            *p.position = Xvec3(e,e,e);
            *p.orientation = Default::default();
            *p.scale = Xvec3(e,e,e);
        }
    }
    println!("{:?}", xforms);
}

fn main() {
    toast();
    toast_mt(2);
    toast_mt(6);
    toast_mt(10);
    toast_mt(32);
    toast_demo();
    test_bound_len();
}

#[derive(Debug, Clone, PartialEq)]
struct Vvv<'a, T> {
    v: Vec<T>,
    len: &'a usize,
}

impl<'a, T> Vvv<'a, T> {
    fn enumerate<F>(len: &'a usize, f: F) -> Self where F: FnMut((usize, &mut T)) {
        let mut v = Vec::with_capacity(*len);
        unsafe {
            v.set_len(*len);
        }
        v.iter_mut().enumerate().map(f).count();
        Self { v: v, len: len }
    }
    fn map<F>(len: &'a usize, mut f: F) -> Self where F: FnMut(&mut T) {
        Self::enumerate(len, |(_, x)| f(x))
    }
    fn broadcast(len: &'a usize, val: T) -> Self where T: Clone {
        Self::map(len, |x| *x = val.clone())
    }
    fn default(len: &'a usize) -> Self where T: Default + Clone {
        Self::broadcast(len, T::default())
    }
}
use std::ops::*;
impl<'a, T> Add for Vvv<'a, T> where T: Clone + Add<Output=T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        debug_assert_eq!(self.len, rhs.len);
        Self {
            len: self.len,
            v: self.v.iter().zip(rhs.v.iter()).map(|(s,r)| s.clone()+r.clone()).collect()
        }
    }
}

fn test_bound_len() {
    let len = 8;
    let lll = len; // Set it to something else : still compiles, but panics.
    let t = Vvv::<i32>::default(&len);
    let u = Vvv::broadcast(&lll, 42);
    let v = Vvv::enumerate(&len, |(i, x)| *x = i+i+1);
    let w = Vvv::map(&len, |x| *x = 3);
    println!("");
    println!("");
    println!("{:?}", t);
    println!("{:?}", u);
    println!("{:?}", v);
    println!("{:?}", w);
    println!("{:?}", u+v);
}
