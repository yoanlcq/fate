//! The Semver (Semantic versioning) struct.

/// Semantic versioning structure ([semver.org](http://semver.org)): 
/// that is, a version number in the `major.minor.patch` format.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Semver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Default for Semver {
    fn default() -> Self {
        Self::new(0,1,0)
    }
}

impl Semver {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}
impl From<(u32,u32,u32)> for Semver {
    fn from(tuple: (u32,u32,u32)) -> Self {
        let (major, minor, patch) = tuple;
        Self { major, minor, patch }
    }
}
impl From<(u32,u32)> for Semver {
    fn from(tuple: (u32,u32)) -> Self {
        let ((major, minor), patch) = (tuple, 0);
        Self { major, minor, patch }
    }
}


#[test]
fn test_semver_cmp() {
    assert!(Semver::new(3,2,1) > Semver::new(2,2,1));
    assert!(Semver::new(3,2,1) >= Semver::new(2,2,1));
    assert!(Semver::new(3,2,1) >= Semver::new(3,2,1));
    assert!(Semver::new(3,2,1) == Semver::new(3,2,1));
    assert!(Semver::new(3,2,1) < Semver::new(4,2,1));
    assert!(Semver::new(3,2,1) <= Semver::new(4,2,1));
    assert!(Semver::new(3,2,1) <= Semver::new(3,2,1));
    assert!(Semver::new(3,2,1) > Semver::new(2,7,8));
    assert!(Semver::new(2,7,8) < Semver::new(3,2,1));
}
