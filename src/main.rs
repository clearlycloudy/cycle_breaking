#![feature(weak_into_raw)] 
#![feature(weak_ptr_eq)]

#[macro_use]
extern crate lazy_static;

use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::borrow::BorrowMut;
use std::ops::Deref;
use std::collections::HashSet;
use std::cmp::{PartialEq,Eq};
use std::hash::{Hasher,Hash};
use std::sync::{Arc,Mutex};

#[derive(Clone,Debug)]
enum Link {
    S(Rc<RefCell<Inner>>),
    W(Weak<RefCell<Inner>>),
}

impl Default for Link {
    fn default() -> Link {
        Link::W(Weak::new())
    }
}

#[derive(Clone,Default,Debug)]
struct InnerWrap (pub Link);

#[derive(Clone,Default,Debug)]
struct Inner {
    pub val: u32,
    pub next: Option<InnerWrap>,
}

impl Drop for Inner {
    fn drop(& mut self){
        println!("drop inner: val: {}",self.val);
    }
}

impl InnerWrap {
    pub fn drop_helper(& mut self, q : & mut HashSet<InnerWrap> ) {
        
        use std::mem;
        
        let mut a = Link::default();
        mem::swap( & mut self.0, & mut a );
        
        match & mut a {
            Link::S(ref mut x) => {
                match x.deref().deref().try_borrow_mut() {
                    Ok(ref mut xx) => {
                        xx.drop_collect( q );
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }
}

impl Drop for InnerWrap {
    fn drop(& mut self){

        use std::mem;

        let mut q = HashSet::new();
        
        let mut a = Link::default();
        mem::swap( & mut self.0, & mut a );
        
        match & mut a {
            Link::S(ref mut x) => {
                match x.deref().deref().try_borrow_mut() {
                    Ok(ref mut xx) => {
                        xx.drop_collect( & mut q );
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }
}

impl Inner {
    pub fn drop_collect(& mut self, q: & mut HashSet<InnerWrap> ){
        if self.next.is_some() {
            use std::mem;
            let mut a = None;
            mem::swap( & mut a, & mut self.next );
            q.insert(a.as_ref().unwrap().clone());
            a.unwrap().drop_helper( q );
        }
    }
}

impl PartialEq for InnerWrap {
    fn eq(&self, other:&Self) -> bool {
        
        //decay both to Weak and compare
        
        let a = match self.0 {
            Link::S(ref x) => {Rc::downgrade(x)},
            Link::W(ref x) => {x.clone()},
        };

        let b = match other.0 {
            Link::S(ref x) => {Rc::downgrade(x)},
            Link::W(ref x) => {x.clone()},
        };

        Weak::ptr_eq(&a, &b)
    }
}

impl Hash for InnerWrap {
    fn hash<H:Hasher>(&self, state: &mut H){
        match self.0 {
            Link::S(ref x) => {
                let p = Rc::downgrade(x);
                (Weak::as_raw(&p) as usize).hash(state);
            },
            Link::W(ref x) => {
                (Weak::as_raw(x) as usize).hash(state);
            },
        }

    }
}

impl Eq for InnerWrap{}

fn main(){

    let mut in4 = InnerWrap(Link::S(Rc::new(RefCell::new(Inner::default()))));
    let mut in3 = InnerWrap(Link::S(Rc::new(RefCell::new(Inner::default()))));
    let mut in1 = InnerWrap(Link::S(Rc::new(RefCell::new(Inner::default()))));
    let mut in2 = InnerWrap(Link::S(Rc::new(RefCell::new(Inner::default()))));


    match & mut in1.0 {
        Link::S(ref mut x) =>{
            x.deref().deref().borrow_mut().val = 0;
            x.deref().deref().borrow_mut().next = Some(in4.clone());
        },
        _ => {panic!();},
    }

    match & mut in2.0 {
        Link::S(ref mut x) =>{
            x.deref().deref().borrow_mut().val = 1;
            x.deref().deref().borrow_mut().next = Some(in1.clone());
        },
        _ => {panic!();},
    }

    match & mut in3.0 {
        Link::S(ref mut x) =>{
            x.deref().deref().borrow_mut().val = 2;
            x.deref().deref().borrow_mut().next = Some(in1.clone());
        },
        _ => {panic!();},
    }

    let l = in4.clone();
    match & mut in4.0 {
        Link::S(ref mut x) =>{
            x.deref().deref().borrow_mut().val = 3;
            x.deref().deref().borrow_mut().next = Some(l);
        },
        _ => {panic!();},
    }
}
