#![doc(html_root_url = "https://docs.rs/many-derive/0.1.0")]
#![feature(associated_type_defaults)]
#![feature(inclusive_range, inclusive_range_syntax)]

mod many;
mod wizards;
use wizards::*;

fn main() {
    let _ = Wizards::new();
    let mut wizards = Wizards::with_capacity(4);
    // Init from Aos. Probably not the most efficient cache-wise, but :
    // - Safer;
    // - Never needs refactoring;
    // - Only done once at initialization (normally);
    wizards.alive_mut().extend_with_aos(vec![
        Wizard { life: 42, mana: 13, team: Team::ChaosBlue, color: BLUE },
        Wizard { life: 42, mana: 13, team: Team::ChaosBlue, color: BLUE },
        Wizard { life: 42, mana: 13, team: Team::ChaosBlue, color: BLUE },
    ].into_iter());
    // This one asserts that each row has the same length.
    // Needs refactoring when the data layout changes, but best insertion perf.
    wizards.alive_mut().extend(WizardRows {
        len: 1,
        life_and_mana: vec![WizardLifeMana { life: 42, mana: 13 }].into_iter(),
        team: vec![ Team::OrderRed ].into_iter(),
        color: vec![ BLUE ].into_iter(),
    });
    // Ths one does not perform any length consistency check. Use at your own risk.
    unsafe {
        wizards.alive_mut().extend_unchecked(WizardRows {
            len: 1,
            life_and_mana: vec![WizardLifeMana { life: 42, mana: 13 }].into_iter(),
            team: vec![ Team::ChaosBlue ].into_iter(),
            color: vec![ BLUE ].into_iter()
        });
    }
    

    // Declare an index value for this type
    let _: WizardIndex = 0;

    // --- Iterate mutably over the data
    for w in &mut wizards {
        *w.life = 1000;
        *w.mana = 1000;
        *w.color = BLUE;
        *w.color = RED;
        *w.team = Team::ChaosBlue;
        println!("{:?}", w);
    }
    // Destructure for convenience
    for WizardRefMut { life, .. } in &mut wizards {
        *life = 1000;
        println!("{}", life);
    }
    // Does the same as the above, with an explicit iterator...
    for w in wizards.iter_mut() {
        *w.life = 1000;
        println!("{}", w.life);
    }
    // Rust iterators are powerful !
    for (i, w) in wizards.iter_mut().enumerate() {
        *w.life += (i as i32 + 1)*1000;
        println!("{}", w.life);
    }

    // --- Iterate immutably over the data
    for w in &wizards {
        println!("{}", w.life);
    }
    for WizardRef { life, .. } in &wizards {
        println!("{}", life);
    }
    for w in wizards.iter() {
        println!("{}", w.life);
    }
    for (i, w) in wizards.iter().enumerate() {
        println!("{:?}", (i, w.life));
    }

    // --- Iterate by consuming the data
    // This enforces iteration over an on-the-fly AoS layout.
    // Creating a closure to re-create one every time here 
    // because the for loop takes ownership.
    {
        let wizards = || {
            let mut wizards = Wizards::with_capacity(4);
            wizards.alive_mut().extend_with_aos(vec![
                Wizard { life: 42, mana: 13, team: Team::ChaosBlue, color: BLUE },
                Wizard { life: 42, mana: 13, team: Team::ChaosBlue, color: BLUE },
                Wizard { life: 42, mana: 13, team: Team::ChaosBlue, color: BLUE },
            ].into_iter());
            wizards
        };
        for w in wizards() {
            println!("{}", w.life);
        }
        for Wizard { life, .. } in wizards() {
            println!("{}", life);
        }
        for w in wizards() {
            println!("{}", w.life);
        }
        for (i, w) in wizards().iter().enumerate() {
            println!("{:?}", (i, w.life));
        }
    }
    
    // --- Convenience shortcut methods
    for w in wizards.alive() {
        println!("{}", w.life);
    }
    for (_, w) in wizards.alive().into_iter().enumerate() {
        println!("{}", w.life);
    }
    for w in wizards.alive_mut() {
        *w.life = 1000;
        println!("{}", w.life);
    }
    for (_, w) in wizards.alive_mut().into_iter().enumerate() {
        *w.life = 1000;
        println!("{}", w.life);
    }

    // --- Iterate over multiple sub-sections
    println!("{}", line!());
    for w in wizards.alive().into_iter().chain(wizards.dead().into_iter()) {
        println!("{}", w.life);
    }
    
    // --- Over arbitrary ranges
    for w in wizards.range(0..2) {
        println!("{}", w.life);
    }
    for w in wizards.range(1..) {
        println!("{}", w.life);
    }
    for w in wizards.range(..2) {
        println!("{}", w.life);
    }
    for w in wizards.range(..) {
        println!("{}", w.life);
    }
    for w in wizards.range(...1) {
        println!("{}", w.life);
    }
    for w in wizards.range(0...1) {
        println!("{}", w.life);
    }
    for w in wizards.range_mut(0..2) {
        *w.life = 1000;
        println!("{}", w.life);
    }

    println!("{:?}", wizards);
}
