use std::ops::*;
use std::ptr;
use many::*;

pub use self::base::*;

mod base {
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    #[repr(C, packed)]
    pub struct Rgba<T> {
        pub r: T,
        pub g: T,
        pub b: T,
        pub a: T,
    }

    pub type Rgba32 = Rgba<u8>;
    pub const RED: Rgba32 = Rgba32 { r: 255, g:0 , b: 0, a: 0 };
    pub const BLUE: Rgba32 = Rgba32 { r: 0, g:0 , b: 255, a: 0 };
    pub type Life = i32;
    pub type Mana = i32;
    // XXX derive Copy and Clone
    #[derive(Debug, Hash, PartialEq, Eq)]
    pub enum Team {
        OrderRed,
        ChaosBlue,
    }
}











// XXX Does stuff work with zero-length ? (extend(), etc)

pub type WizardIndex = u16;

/// A Table carries the implicit guarantee that each
/// of its rows is of the same length.
#[derive(Debug, PartialEq)]
pub struct Wizards {
    ranges: WizardSectionRanges,
    life_and_mana: Row<WizardLifeMana>,
    color: Row<Rgba32>,
    team: Row<Team>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct WizardSectionRanges {
    alive: Range<WizardIndex>,
    alive_players: Range<WizardIndex>,
    alive_ai: Range<WizardIndex>,
    dead: Range<WizardIndex>,
}

// XXX Copy and Clone
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct WizardLifeMana {
    pub life: Life,
    pub mana: Mana,
}

/// Available to the user, since it purposefully can't
/// do anything except be consumed by `Wizards::extend()`.
/// 
/// This isn't directly generic over `IntoIterator`s for `ExactSizeIterators`,
/// because of ownership issues when checking the consistency of each
/// row's length.
#[derive(Debug, PartialEq)]
pub struct WizardRows<
    I0: ExactSizeIterator<Item=WizardLifeMana>,
    I1: ExactSizeIterator<Item=Rgba32>,
    I2: ExactSizeIterator<Item=Team>,
> {
    pub len: WizardIndex,
    pub life_and_mana: I0,
    pub color: I1,
    pub team: I2,
}

#[derive(Debug, PartialEq)]
pub struct Wizard {
    pub life: Life,
    pub mana: Mana,
    pub color: Rgba32,
    pub team: Team,
}

// Here instead of getters because of deconstructions.
// TODO: test vs getters
#[derive(Debug, PartialEq)]
pub struct WizardRef<'a> {
    pub life: &'a Life,
    pub mana: &'a Mana,
    pub color: &'a Rgba32,
    pub team: &'a Team,
}

#[derive(Debug, PartialEq)]
pub struct WizardRefMut<'a> {
    pub life: &'a mut Life,
    pub mana: &'a mut Mana,
    pub color: &'a mut Rgba32,
    pub team: &'a mut Team,
}

// Here because we need the range for inserting.
#[derive(Debug, Clone, PartialEq)]
pub struct WizardSectionDescriptor<'a> {
    table: &'a Wizards,
    range: Range<WizardIndex>,
}

#[derive(Debug, PartialEq)]
pub struct WizardMutSectionDescriptor<'a> {
    table: &'a mut Wizards,
    range: Range<WizardIndex>,
}

/// A Section carries the implicit guarantee that each
/// of its rows is of the same length.
// Here because we only need the slices for iterating.
#[derive(Debug, Clone, PartialEq)]
pub struct WizardSection<'a> {
    pub life_and_mana: &'a [WizardLifeMana],
    pub color: &'a [Rgba32],
    pub team: &'a [Team],
}

#[derive(Debug, PartialEq)]
pub struct WizardMutSection<'a> {
    pub life_and_mana: &'a mut [WizardLifeMana],
    pub color: &'a mut [Rgba32],
    pub team: &'a mut [Team],
}

#[derive(Debug, PartialEq)]
pub struct WizardIntoIter {
    table: Wizards,
    i: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WizardIter<'a> {
    section: WizardSection<'a>,
    i: usize,
}

#[derive(Debug, PartialEq)]
pub struct WizardIterMut<'a> {
    section: WizardMutSection<'a>,
    i: usize,
}





// IMPLS



impl Default for Wizards {
    fn default() -> Self {
        Self::new()
    }
}


impl<'a> Wizards {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    pub fn with_capacity(cap: WizardIndex) -> Self {
        Self {
            ranges: WizardSectionRanges {
                alive: 0..0,
                alive_players: 0..0,
                alive_ai: 0..0,
                dead: 0..0,
            },
            life_and_mana: Row::with_capacity(cap as usize),
            color: Row::with_capacity(cap as usize),
            team: Row::with_capacity(cap as usize),
        }
    }
    pub fn iter(&self) -> WizardIter {
        WizardSectionDescriptor::from(self).into_iter()
    }
    pub fn iter_mut(&mut self) -> WizardIterMut {
        WizardMutSectionDescriptor::from(self).into_iter()
    }

    pub fn len(&self) -> WizardIndex {
        self.life_and_mana.len() as WizardIndex
    }


    pub fn alive(&'a self) -> WizardSectionDescriptor<'a> {
        WizardSectionDescriptor {
            table: self, range: self.ranges.alive.clone()
        }
    }
    pub fn alive_mut(&'a mut self) -> WizardMutSectionDescriptor<'a> {
        let range = self.ranges.alive.clone();
        WizardMutSectionDescriptor {
            table: self, range
        }
    }
    pub fn dead(&'a self) -> WizardSectionDescriptor<'a> {
        WizardSectionDescriptor {
            table: self, range: self.ranges.dead.clone()
        }

    }
    pub fn dead_mut(&'a mut self) -> WizardMutSectionDescriptor<'a> {
        let range = self.ranges.dead.clone();
        WizardMutSectionDescriptor {
            table: self, range
        }
    }


    pub fn range<R: IntoIndexRange<WizardIndex>>(&'a self, index: R) -> WizardSectionDescriptor<'a> {
        let range = index.into_index_range(self.len());
        WizardSectionDescriptor {
            table: self, range
        }
    }
    pub fn range_mut<R: IntoIndexRange<WizardIndex>>(&'a mut self, index: R) -> WizardMutSectionDescriptor<'a> {
        let range = index.into_index_range(self.len());
        WizardMutSectionDescriptor {
            table: self, range
        }
    }
}


impl<'a> WizardSection<'a> {
    pub fn len(&self) -> WizardIndex {
        // TODO: Fix this with the Row type
        self.life_and_mana.len() as WizardIndex
    }
}

impl<'a> WizardMutSection<'a> {
    pub fn len(&self) -> WizardIndex {
        // TODO: Fix this with the Row type
        self.life_and_mana.len() as WizardIndex
    }
}


impl<I0,I1,I2> WizardRows<I0,I1,I2>
where
    I0: ExactSizeIterator<Item=WizardLifeMana>,
    I1: ExactSizeIterator<Item=Rgba32>,
    I2: ExactSizeIterator<Item=Team>,
{
    pub fn check_row_lengths(&self) -> Result<(), String> {
        let len = self.len as usize;

        let life_and_mana_len = self.life_and_mana.len();
        let color_len = self.color.len();
        let team_len = self.team.len();
        let cmps = &[
            life_and_mana_len == len,
            color_len == len,
            team_len == len,
        ];
        if cmps.into_iter().fold(true, |acc, &x| acc && x) {
            return Ok(());
        }
            
        let names_and_lens = &[
            ("life_and_mana", life_and_mana_len),
            ("color", color_len),
            ("team", team_len),
        ];
        let mut e = format!("These rows aren't of the expected length (which is {}) :\n", len);
        for (is_correct, &(row_name, row_len)) in cmps.into_iter().zip(names_and_lens.into_iter()) {
            if !is_correct {
                e += format!("- {} ({} elements);\n", row_name, row_len).as_str();
            }
        }
        Err(e)
    }

    pub fn try_into_wizards(self) -> Result<Wizards, String> {
        self.check_row_lengths()?;
        Ok(unsafe { self.into_wizards_unchecked() })
    }
    pub unsafe fn into_wizards_unchecked(self) -> Wizards {
        Wizards {
            ranges: WizardSectionRanges {
                alive: 0..0,
                alive_players: 0..0,
                alive_ai: 0..0,
                dead: 0..0,
            },
            life_and_mana: self.life_and_mana.collect(),
            color: self.color.collect(),
            team: self.team.collect(),
        }
    }
}


impl<'a> WizardMutSectionDescriptor<'a> {
    #[inline]
    pub fn extend<I0,I1,I2>(&mut self, ext: WizardRows<I0,I1,I2>) 
    where
        I0: ExactSizeIterator<Item=WizardLifeMana>,
        I1: ExactSizeIterator<Item=Rgba32>,
        I2: ExactSizeIterator<Item=Team>,
    {
        let i = self.range.end - self.range.start;
        self.insert_trusted_i(i, ext)
    }

    #[inline]
    pub fn insert<I0,I1,I2>(&mut self, index: WizardIndex, ext: WizardRows<I0,I1,I2>) 
    where
        I0: ExactSizeIterator<Item=WizardLifeMana>,
        I1: ExactSizeIterator<Item=Rgba32>,
        I2: ExactSizeIterator<Item=Team>,
    {
        if self.range.start + index > self.range.end {
            panic!("Index is {} but slice ends at {}", index, self.range.end - self.range.start);
        }
        self.insert_trusted_i(index, ext)
    }

    fn insert_trusted_i<I0,I1,I2>(&mut self, index: WizardIndex, ext: WizardRows<I0,I1,I2>) 
        where
        I0: ExactSizeIterator<Item=WizardLifeMana>,
        I1: ExactSizeIterator<Item=Rgba32>,
        I2: ExactSizeIterator<Item=Team>,
    {
        ext.check_row_lengths().unwrap();
        unsafe { self.insert_unchecked(index, ext) }
    }

    pub unsafe fn extend_unchecked<I0,I1,I2>(&mut self, s: WizardRows<I0,I1,I2>)
    where
        I0: ExactSizeIterator<Item=WizardLifeMana>,
        I1: ExactSizeIterator<Item=Rgba32>,
        I2: ExactSizeIterator<Item=Team>,
    {
        let i = self.range.end - self.range.start;
        self.insert_unchecked(i, s)
    }

    pub unsafe fn insert_unchecked<I0,I1,I2>(&mut self, index: WizardIndex, s: WizardRows<I0,I1,I2>) 
    where
        I0: ExactSizeIterator<Item=WizardLifeMana>,
        I1: ExactSizeIterator<Item=Rgba32>,
        I2: ExactSizeIterator<Item=Team>,
    {
        let i = (self.range.start + index) as usize;
        self.table.life_and_mana.insert_intoiter_exactsize(i, s.life_and_mana);
        self.table.color.insert_intoiter_exactsize(i, s.color);
        self.table.team.insert_intoiter_exactsize(i, s.team);
    }



    // There's no "unchecked" counterpart for "*_aos()" : you can't go wrong with Aos layouts.
    // It's supposedly less cache-friendly though.
    pub fn extend_with_aos<I>(&mut self, iter: I) where I: IntoIterator<Item=Wizard> {
        let index = self.range.end - self.range.start;
        unsafe {
            self.insert_aos_unchecked(index, iter)
        }
    }

    pub fn insert_aos<I>(&mut self, index: WizardIndex, iter: I) where I: IntoIterator<Item=Wizard> {
        assert!(self.range.start + index <= self.range.end);
        unsafe {
            self.insert_aos_unchecked(index, iter)
        }
    }

    pub unsafe fn insert_aos_unchecked<I>(&mut self, index: WizardIndex, iter: I) where I: IntoIterator<Item=Wizard> {
        let i = (self.range.start + index) as usize;
        for w in iter {
            self.table.life_and_mana.insert_unchecked(i, WizardLifeMana { life: w.life, mana: w.mana });
            self.table.color.insert_unchecked(i, w.color);
            self.table.team.insert_unchecked(i, w.team);
        }
    }
}





// ITERS





impl IntoIterator for Wizards {
    type Item = Wizard;
    type IntoIter = WizardIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        WizardIntoIter { table: self, i: 0 }
    }
}

// The following two impls are the minimum required -
// the others are implemented on top using From conversions.

impl<'a> IntoIterator for WizardSection<'a> {
    type Item = WizardRef<'a>;
    type IntoIter = WizardIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WizardIter { section: self, i: 0 }
    }
}

impl<'a> IntoIterator for WizardMutSection<'a> {
    type Item = WizardRefMut<'a>;
    type IntoIter = WizardIterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WizardIterMut { section: self, i: 0 }
    }
}

impl<'a> IntoIterator for WizardSectionDescriptor<'a> {
    type Item = WizardRef<'a>;
    type IntoIter = WizardIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WizardSection::from(self).into_iter()
    }
}
impl<'a> IntoIterator for WizardMutSectionDescriptor<'a> {
    type Item = WizardRefMut<'a>;
    type IntoIter = WizardIterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WizardMutSection::from(self).into_iter()
    }
}

impl<'a> IntoIterator for &'a Wizards {
    type Item = WizardRef<'a>;
    type IntoIter = WizardIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WizardSectionDescriptor::from(self).into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Wizards {
    type Item = WizardRefMut<'a>;
    type IntoIter = WizardIterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WizardMutSectionDescriptor::from(self).into_iter()
    }
}



impl Iterator for WizardIntoIter {
    type Item = Wizard;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i as WizardIndex >= self.table.len() {
            return None;
        }
        unsafe {
            // std::vec::IntoIter roughly does the same thing to take ownership
            let out = Wizard {
                life:  ptr::read(self.table.life_and_mana.get_unchecked_mut(self.i)).life,
                mana:  ptr::read(self.table.life_and_mana.get_unchecked_mut(self.i)).mana,
                color: ptr::read(self.table.color.get_unchecked_mut(self.i)),
                team:  ptr::read(self.table.team.get_unchecked_mut(self.i)),
            };
            self.i += 1;
            Some(out)
        }
    }
}

// NOTE: borrow is *actually* needed.
#[cfg_attr(feature = "cargo-clippy", allow(needless_borrow))]
impl<'a> Iterator for WizardIter<'a> {
    type Item = WizardRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i as WizardIndex >= self.section.len() {
            return None;
        }
        unsafe {
            let out = WizardRef { 
                life:  &self.section.life_and_mana.get_unchecked(self.i).life, 
                mana:  &self.section.life_and_mana.get_unchecked(self.i).mana,
                color: &self.section.color.get_unchecked(self.i),
                team:  &self.section.team.get_unchecked(self.i),
            };
            self.i += 1;
            Some(out)
        }
    }
}

impl<'a> Iterator for WizardIterMut<'a> {
    type Item = WizardRefMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i as WizardIndex >= self.section.len() {
            return None;
        }
        unsafe {
            // This is the closest to safety I could come up with to solve lifetime issues.
            // Casting to a pointer does stuff ._.
            // If you know of an idiomatic way, please please tell me about it!
            let self_ = (self as *mut Self).as_mut().unwrap();
            let life_and_mana = self_.section.life_and_mana.get_unchecked_mut(self.i);
            let out = WizardRefMut { 
                life : &mut life_and_mana.life,
                mana : &mut life_and_mana.mana,
                color: self_.section.color.get_unchecked_mut(self.i),
                team : self_.section.team.get_unchecked_mut(self.i),
            };
            self.i += 1;
            Some(out)
        }
    }
}






// CONVERSIONS





impl<'a> From<WizardSectionDescriptor<'a>> for WizardSection<'a> {
    fn from(d: WizardSectionDescriptor<'a>) -> Self {
        let r = d.range.start as usize .. d.range.end as usize;
        WizardSection {
            life_and_mana: &d.table.life_and_mana[r.clone()],
            color: &d.table.color[r.clone()],
            team: &d.table.team[r.clone()],
        }
    }
}

impl<'a> From<WizardMutSectionDescriptor<'a>> for WizardMutSection<'a> {
    fn from(d: WizardMutSectionDescriptor<'a>) -> Self {
        let r = d.range.start as usize .. d.range.end as usize;
        WizardMutSection {
            life_and_mana: &mut d.table.life_and_mana[r.clone()],
            color: &mut d.table.color[r.clone()],
            team: &mut d.table.team[r.clone()],
        }
    }
}


impl<'a> From<&'a Wizards> for WizardSectionDescriptor<'a> {
    fn from(table: &'a Wizards) -> Self {
        WizardSectionDescriptor { table, range: 0..table.len() }
    }
}

impl<'a> From<&'a mut Wizards> for WizardMutSectionDescriptor<'a> {
    fn from(table: &'a mut Wizards) -> Self {
        let range = 0 .. table.len();
        WizardMutSectionDescriptor { table, range }
    }
}

impl<'a> From<WizardMutSectionDescriptor<'a>> for WizardSectionDescriptor<'a> {
    fn from(d: WizardMutSectionDescriptor<'a>) -> Self {
        WizardSectionDescriptor { table: d.table, range: d.range }
    }
}

impl<'a> From<WizardMutSection<'a>> for WizardSection<'a> {
    fn from(s: WizardMutSection<'a>) -> Self {
        WizardSection {
            life_and_mana: s.life_and_mana,
            color: s.color,
            team: s.team,
        }
    }
}

// XXX Soon won't match the Row type
impl From<Wizards> for WizardRows<
    ::std::vec::IntoIter<WizardLifeMana>,
    ::std::vec::IntoIter<Rgba32>,
    ::std::vec::IntoIter<Team>,
> {
    fn from(table: Wizards) -> Self {
        WizardRows {
            len: table.len(),
            life_and_mana: table.life_and_mana.into_iter(),
            color: table.color.into_iter(),
            team: table.team.into_iter(),
        }
    }
}

