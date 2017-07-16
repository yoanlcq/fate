//! Debug breakpoint intrinsics and utilities - currently wrapping LLVM's `debugtrap` intrinsic.
//!
//! The matching Rust intrinsic would be `core::intrinsics::breakpoint`.
//!
//! Since debug breakpoints either allow to step through program execution or abort the program
//! (that is "exit without running destructors"), they match Rust's definition of "safe".
//!
//! [Checking if the app is being run under a debugger](http://stackoverflow.com/a/24969863) is
//! possible, but very OS-specific. This module provides variants of `debugtrap()` which trigger
//! a breakpoint only when the process knows that a debugger is attached : 
//! those have `_checked` in their name.

#![cfg_attr(feature = "cargo-clippy", allow(inline_always))]

/// Possible errors returned by `debugger_is_attached()` and propagated by `debugtrap_checked()`.
#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub enum DebugtrapError {
    /// This target currently provides no way to check if a debugger is attached.
    CannotImplement,
    /// Not implemented yet.
    NotImplementedYet,
    /// Can't prove that a debugger is attached to this process.
    DebuggerMightNotBeAttached,
    /// Returned by `debugbuild_trap_checked()` on non-debug builds.
    NonDebugBuild,
}

use DebugtrapError::*;
use std::fmt::{self, Display, Formatter};
use std::error::Error;

impl Display for DebugtrapError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for DebugtrapError {
    fn description(&self) -> &'static str {
        match *self {
            CannotImplement => "This target currently provides no way to check if a debugger is attached",
            NotImplementedYet => "Not implemented yet! Contributions are welcome",
            DebuggerMightNotBeAttached => "Can't prove that a debugger is attached to this process",
            NonDebugBuild => "Requested to trap only for debug builds",
        }
    }
}

/// Is a debugger attached to the process ? NOTE: very OS-specific.
pub fn debugger_is_attached() -> Result<(), DebugtrapError> {
    Err(NotImplementedYet)
}

/// Triggers a breakpoint only if the process can prove that a debugger is attached to it.
///
/// Here, the purpose of errors is only to explain why the process wasn't trapped -
/// they certainly do not mean that you should `panic!()`, but you may.
///
/// # Example
/// ```rust,no_run
/// # use fate::debugtrap_checked;
/// // Basic use - either safely triggers a breakpoint into your debugger, or does nothing.
/// // Result types can't be ignored, so just bind it to the unused placeholder.
/// let _ = debugtrap_checked();
///
/// // Attempts to trigger a breakpoint, doing something else if the breakpoint wasn't triggered
/// if let Err(_) = debugtrap_checked() {
///     panic!("Don't wanna go further down this code until it's fixed!");
/// }
///
/// // Some kind of party here
/// match debugtrap_checked() {
///     Ok(_) => println!("Now debugging!"),
///     Err(e) => println!("Didn't trigger breakpoint: {}.", e),
/// }
///
/// ```
#[inline(always)]
pub fn debugtrap_checked() -> Result<(), DebugtrapError> {
    if let Err(e) = debugger_is_attached() {
        Err(e)
    } else {
        debugtrap();
        Ok(())    
    }
}

/// Triggers a breakpoint only on debug builds **and** if the process can prove that a debugger is attached to it.
///
/// On debug builds, this resolves to `debugtrap_checked()`.
/// On non-debug builds, this consistently errors with `DebugtrapError::NonDebugBuild`.
#[cfg(debug_assertions)]
#[inline(always)]
pub fn debugbuild_trap_checked() -> Result<(), DebugtrapError> {
    debugtrap_checked()
}

#[cfg(not(debug_assertions))]
#[inline(always)]
pub fn debugbuild_trap_checked() -> Result<(), DebugtrapError> {
    Err(NonDebugBuild)
}

#[cfg(not(debug_assertions))]
#[inline(always)]
pub fn debugbuild_trap() {}

/// Triggers a breakpoint only on debug builds, does nothing otherwise.
#[cfg(debug_assertions)]
#[inline(always)]
pub fn debugbuild_trap() {
    debugtrap()
}

/// Always attempts to trigger a breakpoint.
///
/// Attempting to trigger a breakpoint while your app 
/// is not being run from within a debugger is undefined behaviour, 
/// but will abort the process on most platforms.
#[inline(always)]
pub fn debugtrap() {
    unsafe {
        llvm_debugtrap()
    }
}

/// Triggers a breakpoint only on debug builds, executes a closure otherwise.
///
/// There's no `debugbuild_trap_and<F>(f: F)` function because it's equivalent to putting the code
/// after a regular call to [`debugbuild_trap()`](fn.debugbuild_trap.html).
///
/// # Example
/// ```rust,no_run
/// # use fate::debugbuild_trap_or_else;
/// // On debug builds, this will only trigger a breakpoint.
/// // On release builds, this will execute the given closure, which chooses to panic here.
/// debugbuild_trap_or_else(|| panic!("This ain't supposed to happen!"));
/// 
/// // The following is equivalent in every way to debugbuild_trap().
/// debugbuild_trap_or_else(|| ());
/// ```
#[cfg(debug_assertions)]
#[inline(always)]
pub fn debugbuild_trap_or_else<F: Fn()>(_: F) {
    debugtrap()
}

#[cfg(not(debug_assertions))]
#[inline(always)]
pub fn debugbuild_trap_or_else<F: Fn()>(f: F) {
    f()
}

extern {
    #[inline(always)]
    #[link_name = "llvm.debugtrap"]
    fn llvm_debugtrap();
}
