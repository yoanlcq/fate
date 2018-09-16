use std::ops::{Index, IndexMut, Deref};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
struct Info {
    pub index: u32, // Index into items
    pub generation: u32,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
pub struct Key {
    index: u32, // Index into infos
    generation: u32,
}

impl Key {
    pub fn index(&self) -> usize { self.index as _ }
    pub fn generation(&self) -> u32 { self.generation }
}


#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct DMap<V> {
    items: Vec<V>,
    backs: Vec<u32>, // Index into infos. Aligned with `items`, and maps back to an info index.
    infos: Vec<Info>, // Index into items. NOTE: This vec is sparse, and Infos themselves don't know if they point to something valid. Use `frees` or `backs`!
    frees: Vec<u32>, // Free list of indices into `info`
}

impl<V> DMap<V> {
    pub fn remove(&mut self, k: Key) -> Option<V> {
        let i = {
            let info = self.infos.get_mut(k.index as usize)?;
            if info.generation != k.generation {
                return None;
            }
            info.generation = info.generation.wrapping_add(1);
            info.index as usize
        };
        let item = self.items.swap_remove(i);
        let _bck = self.backs.swap_remove(i);
        if i < self.items.len() {
            let back = self.backs[i] as usize;
            self.infos[back].index = i as _;
        }
        self.frees.push(k.index);
        Some(item)
    }
    pub fn push(&mut self, item: V) -> Key {
        self.items.push(item);
        self.infos.push(Info { index: (self.items.len() - 1) as _, generation: 0, });
        let k = Key { index: (self.infos.len() - 1) as _, generation: 0, };
        self.backs.push(k.index);
        k
    }
    pub fn insert(&mut self, item: V) -> Key {
        match self.frees.is_empty() {
            true => self.push(item),
            false => {
                let i = self.frees.swap_remove(0);
                self.items.push(item);
                let info = &mut self.infos[i as usize];
                info.index = (self.items.len() - 1) as _;
                self.backs.push(i);
                Key { index: i as _, generation: info.generation, }
            },
        }
    }

    pub fn contains_key(&self, k: Key) -> bool {
        match self.infos.get(k.index as usize) {
            None => false,
            Some(info) => info.generation == k.generation,
        }
    }
    #[inline]
    pub unsafe fn get_unchecked(&self, k: Key) -> &V {
        self.items.get_unchecked(self.infos.get_unchecked(k.index as usize).index as usize)
    }
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, k: Key) -> &mut V {
        self.items.get_unchecked_mut(self.infos.get_unchecked(k.index as usize).index as usize)
    }
    pub fn get(&self, k: Key) -> Option<&V> {
        if self.contains_key(k) {
            Some(unsafe { self.get_unchecked(k) })
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, k: Key) -> Option<&mut V> {
        if self.contains_key(k) {
            Some(unsafe { self.get_unchecked_mut(k) })
        } else {
            None
        }
    }


    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_and_free_list_capacity(cap, 0)
    }
    pub fn with_capacity_and_free_list_capacity(cap: usize, fcap: usize) -> Self {
        Self { 
            items: Vec::with_capacity(cap),
            backs: Vec::with_capacity(cap),
            infos: Vec::with_capacity(cap),
            frees: Vec::with_capacity(fcap),
        }
    }
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }
    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
        self.infos.reserve(additional);
    }
    pub fn reserve_free_list(&mut self, additional: usize) {
        self.frees.reserve(additional);
    }
    pub fn clear(&mut self) {
        // NOTE: Perform "mass removal" to make sure that keys created before "clear" are
        // invalidated.
        self.items.clear();
        for (i, info) in self.infos.iter_mut().enumerate() {
            info.generation = info.generation.wrapping_add(1);
            self.frees.push(i as _);
        }
    }
    pub fn as_slice(&self) -> &[V] { &self.items }

    // NOTE: private!!!!!!
    // Otherwise this could allow breaking referential integrity by sorting.
    #[allow(dead_code)]
    fn as_mut_slice(&mut self) -> &[V] { &mut self.items }

    pub fn keys(&self) -> Keys<V> {
        Keys::new(self)
    }
    pub fn values(&self) -> Values<V> {
        Values::new(self)
    }
    pub fn values_mut(&mut self) -> ValuesMut<V> {
        ValuesMut::new(self)
    }
    pub fn iter(&self) -> Iter<V> {
        Iter::new(self)
    }
    pub fn iter_mut(&mut self) -> IterMut<V> {
        IterMut::new(self)
    }

}

impl<V> Index<Key> for DMap<V> {
    type Output = V;
    #[inline]
    fn index(&self, k: Key) -> &V {
        self.get(k).unwrap()
    }
}

impl<V> IndexMut<Key> for DMap<V> {
    #[inline]
    fn index_mut(&mut self, k: Key) -> &mut V {
        self.get_mut(k).unwrap()
    }
}

impl<V> Index<usize> for DMap<V> {
    type Output = V;
    #[inline]
    fn index(&self, i: usize) -> &V {
        &self.items[i]
    }
}

impl<V> IndexMut<usize> for DMap<V> {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut V {
        &mut self.items[i]
    }
}

impl<V> Deref for DMap<V> {
    type Target = [V];
    fn deref(&self) -> &[V] {
        self.as_slice()
    }
}



#[derive(Debug)] pub struct Keys     <'a, V: 'a> { c: &'a     DMap<V>, i: usize, }
#[derive(Debug)] pub struct Values   <'a, V: 'a> { items: &'a     [V], i: usize, }
#[derive(Debug)] pub struct ValuesMut<'a, V: 'a> { items: &'a mut [V], i: usize, }
#[derive(Debug)] pub struct Iter     <'a, V: 'a> { c: &'a     DMap<V>, i: usize, }
#[derive(Debug)] pub struct IterMut  <'a, V: 'a> { c: &'a mut DMap<V>, i: usize, }

impl<'a, V> Keys     <'a, V> { pub fn new(c: &'a     DMap<V>) -> Self { Self { c, i: 0, } } }
impl<'a, V> Values   <'a, V> { pub fn new(c: &'a     DMap<V>) -> Self { Self { items: &c.items, i: 0, } } }
impl<'a, V> ValuesMut<'a, V> { pub fn new(c: &'a mut DMap<V>) -> Self { Self { items: &mut c.items, i: 0, } } }
impl<'a, V> Iter     <'a, V> { pub fn new(c: &'a     DMap<V>) -> Self { Self { c, i: 0, } } }
impl<'a, V> IterMut  <'a, V> { pub fn new(c: &'a mut DMap<V>) -> Self { Self { c, i: 0, } } }

impl<'a, V> Iterator for Keys<'a, V> {
    type Item = Key;
    fn next(&mut self) -> Option<Key> {
        let i = self.i;
        self.i += 1;
        self.c.backs.get(i).map(|back| unsafe {
            let info = *self.c.infos.get_unchecked(*back as usize);
            Key { index: *back, generation: info.generation }
        })
    }
}
impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> {
        let i = self.i;
        self.i += 1;
        self.items.get(i)
    }
}
impl<'a, V> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<&'a mut V> {
        let i = self.i;
        self.i += 1;
        self.items.get_mut(i).map(|v| unsafe {(v as *mut V).as_mut().unwrap() })
    }
}
impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (Key, &'a V);
    fn next(&mut self) -> Option<(Key, &'a V)> {
        let i = self.i;
        self.i += 1;
        if i >= self.c.items.len() {
            return None;
        }
        unsafe {
            let back = *self.c.backs.get_unchecked(i);
            let info = *self.c.infos.get_unchecked(back as usize);
            Some((Key { index: back, generation: info.generation }, self.c.items.get_unchecked(i)))
        }
    }
}
impl<'a, V> Iterator for IterMut<'a, V> {
    type Item = (Key, &'a mut V);
    fn next(&mut self) -> Option<(Key, &'a mut V)> {
        let i = self.i;
        self.i += 1;
        if i >= self.c.items.len() {
            return None;
        }
        unsafe {
            let back = *self.c.backs.get_unchecked(i);
            let info = *self.c.infos.get_unchecked(back as usize);
            Some((Key { index: back, generation: info.generation }, (self.c.items.get_unchecked_mut(i) as *mut V).as_mut().unwrap()))
        }
    }
}

impl<'a, V> IntoIterator for &'a DMap<V> {
    type Item = (Key, &'a V);
    type IntoIter = Iter<'a, V>;
    fn into_iter(self) -> Iter<'a, V> { self.iter() }
}
impl<'a, V> IntoIterator for &'a mut DMap<V> {
    type Item = (Key, &'a mut V);
    type IntoIter = IterMut<'a, V>;
    fn into_iter(self) -> IterMut<'a, V> { self.iter_mut() }
}
