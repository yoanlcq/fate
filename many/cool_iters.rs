// !!! It's a dump for ideas. It won't compile!

#[derive(Debug, Default, Copy, Clone, PartialEq)]
#[repr(C, packed)]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

pub type Rgba32 = Rgba<u8>;
const RED: Rgba32  = Rgba32 { x: 255, g: 0, b: 0, a: 255 };
const BLUE: Rgba32 = Rgba32 { x: 0, g: 0, b: 255, a: 255 };
pub type Life = i32;
pub type Mana = i32;
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Team {
    OrderRed,
    ChaosBlue,
}

// TODO change this
pub type Row<T> = Vec<T>;





use std::ops::Range;

type WizardSectionRange = Range<WizardIdx>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WizardSectionRanges {
    pub alive: WizardSectionRange,
    pub alive_players: WizardSectionRange,
    pub alive_ai: WizardSectionRange,
    pub dead: WizardSectionRange,
}
#[derive(Debug, Clone)]
pub struct WizardSections<'a> {
    table: &'a Wizards,
}
#[derive(Debug, Clone, Hash)]
pub struct WizardSection<'a> {
    pub life_and_mana: &'a [WizardLifeAndMana],
    pub color: &'a [Rgba32],
    pub team: &'a [Team],
}
impl<'a> WizardSection<'a> {
    pub fn from_range(table: &'a Wizards, range: &WizardSectionRange) -> Self {
        let r = range.start as usize .. range.end as usize;
        WizardSection {
            life_and_mana: &table.life_and_mana[r.clone()],
            color: &table.color[r.clone()],
            team: &table.team[r.clone()],
        }
    }
}
impl WizardSections {
    pub fn alive(&self)   -> WizardSection { WizardSection::from_range(self.table, &self.table.ranges.alive) }
    pub fn alive_players(&self)   -> WizardSection { WizardSection::from_range(self.table, &self.table.ranges.alive_players) }
    pub fn alive_ai(&self)   -> WizardSection { WizardSection::from_range(self.table, &self.table.ranges.alive_ai) }
    pub fn dead(&self)   -> WizardSection { WizardSection::from_range(self.table, &self.table.ranges.dead) }
}

#[derive(Debug, Clone)]
pub struct WizardMutSections<'a> {
    table: &'a mut Wizards,
}
#[derive(Debug, Clone, Hash)]
pub struct WizardMutSection<'a> {
    pub life_and_mana: &'a mut [WizardLifeAndMana],
    pub color: &'a mut [Rgba32],
    pub team: &'a mut [Team],
}
impl<'a> WizardMutSection<'a> {
    pub fn from_range(table: &'a mut Wizards, range: &WizardSectionRange) -> Self {
        let r = range.start as usize .. range.end as usize;
        WizardSection {
            life_and_mana: &mut table.life_and_mana[r.clone()],
            color: &mut table.color[r.clone()],
            team: &mut table.team[r.clone()],
        }
    }
}
impl WizardMutSections {
    pub fn alive(&self)   -> WizardMutSection { WizardMutSection::from_range(self.table, &self.table.ranges.alive) }
    pub fn alive_players(&self)   -> WizardMutSection { WizardMutSection::from_range(self.table, &self.table.ranges.alive_players) }
    pub fn alive_ai(&self)   -> WizardMutSection { WizardMutSection::from_range(self.table, &self.table.ranges.alive_ai) }
    pub fn dead(&self)   -> WizardMutSection { WizardMutSection::from_range(self.table, &self.table.ranges.dead) }
}

pub type WizardIdx = u16;
// Private on purpose, might disappear with the layout
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct WizardLifeAndMana {
    pub life: Life,
    pub mana: Mana,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Wizards {
    pub len: WizardIdx,
    pub ranges: WizardSectionRanges,
    pub life_and_mana: Row<WizardLifeAndMana>,
    pub color: Row<Rgba32>,
    pub team: Row<Team>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wizard {
    pub life: Life,
    pub mana: Mana,
    pub color: Rgba32,
    pub team: Team,
}
#[derive(Debug, Clone, PartialEq)]
pub struct WizardRef<'a> {
    pub life: &'a Life,
    pub mana: &'a Mana,
    pub color: &'a Rgba32,
    pub team: &'a Team,
}
#[derive(Debug, Clone, PartialEq)]
pub struct WizardRefMut<'a> {
    pub life: &'a mut Life,
    pub mana: &'a mut Mana,
    pub color: &'a mut Rgba32,
    pub team: &'a mut Team,
}

use std::iter::{Zip, Map};
use std::vec::IntoIter;
use std::cmp::min;

impl From<(((Life, Mana), Rgba32), Team)> for Wizard {
    fn from(tuple: (((Life, Mana), Rgba32), Team)) -> Self {
        let (((life, mana), color), team) = tuple;
        Self { life, mana, color, team }
    }
}

impl IntoIterator for Wizards {
    type Item = Wizard;
    type IntoIter = Map<
        Zip<
            Zip<
                Zip<
                    IntoIter<Life>, 
                    IntoIter<Mana>
                >,
                IntoIter<Rgba32>
            >,
            IntoIter<Team>
        >, 
        fn((Life, Mana, Rgba32, Team)) -> Wizard
    >;
    fn into_iter(self) -> Self::IntoIter {
             self.life.into_iter()
        .zip(self.mana.into_iter())
        .zip(self.color.into_iter())
        .zip(self.team.into_iter())
        .map(Wizard::from)
    }
}


pub struct WizardIter<'a> {
    section: &'a WizardSection,
    i: usize,
}

impl<'a> IntoIterator for &'a Wizards {
    type Item = WizardRef<'a>;
    type IntoIter = WizardIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WizardIter { section: self, i: 0 }
    }
}

impl<'a> Iterator for SoaIter<'a> {
    type Item = WizardRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.len {
            return None;
        }
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


pub struct WizardIterMut<'a> {
    section: &'a mut WizardMutSection,
    i: usize,
}

impl<'a> Iterator for WizardIterMut<'a> {
    type Item = WizardRefMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.len {
            return None;
        }
        unsafe {
            // Ugly hack to bypass lifetimes issues, but of course it's safe.
            let out = WizardRefMut {
                life:  (&mut self.section.life_and_mana.get_unchecked(self.i).life as *mut Life).as_mut().unwrap(),
                mana:  (&mut self.section.life_and_mana.get_unchecked(self.i).mana as *mut Mana).as_mut().unwrap(),
                color: (&mut self.section.color.get_unchecked(self.i) as *mut Color).as_mut().unwrap(),
                team:  (&mut self.section.team.get_unchecked(self.i) as *mut Team).as_mut().unwrap(),
            };
            self.i += 1;
            Some(out)
        }
    }
}

impl<'a> IntoIterator for &'a mut Wizards {
    type Item = WizardRefMut<'a>;
    type IntoIter = WizardIterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WizardIterMut { section: self, i: 0 }
    }
}

extern crate num_traits;
use num_traits::sign::Unsigned;
use num_traits::Zero;

pub trait Many<'a> {
    type Idx: Unsigned;
    type One;
    type Ref;
    type RefMut;
    fn with_capacity(cap: Self::Idx) -> Self where Self: Sized;
    fn new() -> Self where Self: Sized {
        Self::with_capacity(Self::Idx::zero())
    }
    fn iter(&'a self) -> <&'a Self as IntoIterator>::IntoIter where &'a Self: IntoIterator {
        self.into_iter()
    }
    fn iter_mut(&'a mut self) -> <&'a mut Self as IntoIterator>::IntoIter where &'a mut Self: IntoIterator {
        self.into_iter()
    }
}

impl<'a> Many<'a> for Wizards {
    type Idx = WizardIdx;
    type One = Wizard;
    type Ref = WizardRef<'a>;
    type RefMut = WizardRefMut<'a>;
    fn with_capacity(cap: Self::Idx) -> Self {
        Self {
            life_and_mana: Row::with_capacity(cap as usize),
            color: Row::with_capacity(cap as usize),
            team: Row::with_capacity(cap as usize),
        }
    }
}

fn main() {
    // TODO: Make it impossible to have vectors that differ in length
    let mut wizards = Wizards {
        life_and_mana: vec![(42, 13), (45, 14), (46, 15)],
        color: vec![BLUE, RED, RED],
        team: vec![Team::ChaosBlue, Team::OrderRed, Team::OrderRed],
    };

    // Declare an index value for this type
    let _: WizardIdx = 0;

    // --- Iterate mutably over the data
    for w in &mut wizards {
        *w.life = 1000;
    }
    // Deconstruct for convenience
    for WizardRefMut { life, .. } in &mut wizards {
        *life = 1000;
    }
    // Does the same as the above, with an explicit iterator...
    for w in wizards.iter_mut() {
        *w.life = 1000;
    }
    // Rust iterators are powerful !
    for (i, w) in wizards.iter_mut().enumerate() {
        *w.life += (i as i32 + 1)*1000;
    }

    // -- Iterate immutably over the data
    for w in &wizards {
        w.life;
    }
    for WizardRef { life, .. } in &wizards {
        let _ = life;
    }
    for w in wizards.iter() {
        w.life;
    }
    for (i, w) in wizards.iter().enumerate() {
        (i, w.life);
    }

    // --- Iterate by consuming the data
    // This enforces iteration over an on-the-fly AoS layout.
    // I clone() it here for the sake of the example.
    for w in wizards.clone() {
        w.life;
    }
    for Wizard { life, .. } in wizards.clone() {
        let _ = life;
    }
    for w in wizards.clone().iter() {
        w.life;
    }
    for (i, w) in wizards.clone().iter().enumerate() {
        (i, w.life);
    }
    
    // --- Iterate over a sub-section
    for w in wizards.sections().alive() {
        w.life;
    }
    for w in wizards.sections_mut().alive() {
        w.life;
    }
            
    // --- Iterate over multiple sub-sections
    for w in wizards.sections().zip(|s| (s.alive(), s.dead())) {
        w.life;
    }
    for w in wizards.sections_mut().zip(|s| (s.alive(), s.dead())) {
        w.life;
    }
    
    // --- Purposefully iterate over one row, forces refactoring
    for life in wizards.sections().alive().life.iter_mut() {
        life;
    }
    for life in &mut wizards.sections().alive().life {
        life;
    }

    println!("{:?}", soa);
}
