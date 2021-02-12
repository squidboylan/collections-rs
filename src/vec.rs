use std::alloc;
use std::marker::PhantomData;
use std::ops::Index;
use std::ops::IndexMut;
use std::ptr;

struct MyVec<T> {
    data: *mut T,
    length: usize,
    cap: usize,
    layout: alloc::Layout,
    pd: PhantomData<T>,
}

#[allow(dead_code)]
impl<T> MyVec<T> {
    pub fn new() -> Self {
        let cap = if std::mem::size_of::<T>() == 0 { !0 } else { 0 };
        let data = std::mem::align_of::<T>() as *mut _;
        Self {
            data,
            length: 0,
            cap,
            layout: alloc::Layout::new::<T>(),
            pd: PhantomData,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let layout = alloc::Layout::array::<T>(cap).unwrap();
        let data = unsafe { alloc::alloc(layout) as *mut _ };
        Self {
            data,
            length: 0,
            cap,
            layout,
            pd: PhantomData,
        }
    }

    pub fn push(&mut self, v: T) {
        if self.length == self.cap {
            self.grow();
        }
        println!("{:?}", self.length);
        unsafe {
            ptr::write(self.data.offset(self.length as isize), v);
        }
        self.length += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            Some(unsafe { ptr::read(self.data.offset(self.length as isize)) })
        }
    }

    fn grow(&mut self) {
        if self.cap != 0 {
            let new_size = self.cap * 2;
            let new_layout = alloc::Layout::array::<T>(new_size).unwrap();
            let ptr =
                unsafe { alloc::realloc(self.data as *mut _, self.layout, new_layout.size()) };
            println!("{:?}", ptr);
            if ptr.is_null() {
                panic!("realloc failed");
            }
            self.layout = new_layout;
            self.cap = new_size;
            self.data = ptr as *mut _;
        } else {
            let new_layout = alloc::Layout::array::<T>(1).unwrap();
            unsafe {
                self.data = alloc::alloc(new_layout) as *mut _;
            }
            self.layout = new_layout;
            self.cap = 1;
        }
    }

    pub fn get<'a>(&'a self, i: usize) -> Option<&'a T> {
        if i >= self.length {
            None
        } else {
            unsafe { Some(&*self.data.offset(i as isize)) }
        }
    }

    pub fn get_mut<'a>(&'a mut self, i: usize) -> Option<&'a mut T> {
        if i >= self.length {
            None
        } else {
            unsafe { Some(&mut *self.data.offset(i as isize)) }
        }
    }
}

impl<T> Index<usize> for MyVec<T> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        match self.get(i) {
            Some(x) => x,
            None => panic!("index {} is out of bounds", i),
        }
    }
}

impl<T> IndexMut<usize> for MyVec<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match self.get_mut(i) {
            Some(x) => x,
            None => panic!("index {} is out of bounds", i),
        }
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
        if std::mem::size_of::<T>() != 0 {
            if self.cap > 0 {
                unsafe {
                    alloc::dealloc(self.data as *mut _, self.layout);
                }
            }
        }
    }
}

#[test]
fn simple_vec() {
    let mut v = MyVec::<u32>::new();
    assert_eq!(0, v.cap);
    assert_eq!(0, v.length);
    v.push(1);
    assert_eq!(1, v.cap);
    assert_eq!(1, v.length);
    v.push(2);
    assert_eq!(2, v.cap);
    assert_eq!(2, v.length);
    v.push(3);
    assert_eq!(4, v.cap);
    assert_eq!(3, v.length);
    v.push(4);
    assert_eq!(4, v.cap);
    assert_eq!(4, v.length);

    assert_eq!(Some(&1), v.get(0));
    assert_eq!(Some(&2), v.get(1));
    assert_eq!(Some(&3), v.get(2));
    assert_eq!(Some(&4), v.get(3));
    assert_eq!(None, v.get(10));

    assert_eq!(Some(&mut 1), v.get_mut(0));
    assert_eq!(Some(&mut 2), v.get_mut(1));
    assert_eq!(Some(&mut 3), v.get_mut(2));
    assert_eq!(Some(&mut 4), v.get_mut(3));
    assert_eq!(None, v.get_mut(10));

    assert_eq!(Some(4), v.pop());
    assert_eq!(Some(3), v.pop());
    assert_eq!(Some(2), v.pop());
    assert_eq!(Some(1), v.pop());
    assert_eq!(None, v.pop());
    assert_eq!(0, v.length);
    assert_eq!(4, v.cap);
}

#[test]
fn vec_index() {
    let mut v = MyVec::<u32>::new();
    assert_eq!(0, v.cap);
    assert_eq!(0, v.length);
    v.push(1);
    assert_eq!(1, v.cap);
    assert_eq!(1, v.length);
    v.push(2);
    assert_eq!(2, v.cap);
    assert_eq!(2, v.length);
    v.push(3);
    assert_eq!(4, v.cap);
    assert_eq!(3, v.length);
    v.push(4);
    assert_eq!(4, v.cap);
    assert_eq!(4, v.length);

    assert_eq!(1, v[0]);
    assert_eq!(2, v[1]);
    assert_eq!(3, v[2]);
    assert_eq!(4, v[3]);

    v[0] = 10;
    assert_eq!(10, v[0]);
}

#[test]
#[should_panic]
fn vec_index_oob() {
    let mut v = MyVec::<u32>::new();
    assert_eq!(0, v.cap);
    assert_eq!(0, v.length);
    v.push(1);
    assert_eq!(1, v.cap);
    assert_eq!(1, v.length);
    v.push(2);
    assert_eq!(2, v.cap);
    assert_eq!(2, v.length);
    v.push(3);
    assert_eq!(4, v.cap);
    assert_eq!(3, v.length);
    v.push(4);
    assert_eq!(4, v.cap);
    assert_eq!(4, v.length);

    assert_eq!(1, v[0]);
    assert_eq!(2, v[1]);
    assert_eq!(3, v[2]);
    assert_eq!(4, v[3]);

    v[10] = 10;
    assert_eq!(10, v[0]);
}

#[test]
fn zst_vec_index() {
    let mut v = MyVec::<()>::new();
    assert_eq!(!0, v.cap);
    assert_eq!(0, v.length);
    v.push(());
    assert_eq!(!0, v.cap);
    assert_eq!(1, v.length);
    v.push(());
    assert_eq!(!0, v.cap);
    assert_eq!(2, v.length);
    v.push(());
    assert_eq!(!0, v.cap);
    assert_eq!(3, v.length);
    v.push(());
    assert_eq!(!0, v.cap);
    assert_eq!(4, v.length);

    assert_eq!((), v[0]);
    assert_eq!((), v[1]);
    assert_eq!((), v[2]);
    assert_eq!((), v[3]);

    v[0] = ();
    assert_eq!((), v[0]);
}

#[test]
#[should_panic]
fn zst_vec_index_oob() {
    let mut v = MyVec::<()>::new();
    assert_eq!(!0, v.cap);
    assert_eq!(0, v.length);
    v.push(());
    assert_eq!(!0, v.cap);
    assert_eq!(1, v.length);
    v.push(());
    assert_eq!(!0, v.cap);
    assert_eq!(2, v.length);
    v.push(());
    assert_eq!(!0, v.cap);
    assert_eq!(3, v.length);
    v.push(());
    assert_eq!(!0, v.cap);
    assert_eq!(4, v.length);

    assert_eq!((), v[0]);
    assert_eq!((), v[1]);
    assert_eq!((), v[2]);
    assert_eq!((), v[3]);

    v[10] = ();
    assert_eq!((), v[0]);
}

#[test]
#[should_panic]
fn vec_empty_index_oob() {
    let v = MyVec::<u32>::new();
    let _ = v[0];
}

#[test]
#[should_panic]
fn vec_empty_index_mut_oob() {
    let mut v = MyVec::<u32>::new();
    v[0] = 1;
}

#[test]
#[should_panic]
fn vec_with_capacity_empty_index_mut_oob() {
    let mut v = MyVec::<u32>::with_capacity(16);
    v[0] = 1;
}
