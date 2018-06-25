use std::fmt::{self, Display, Formatter};
use gl::types::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GLVariant {
    Desktop,
    ES,
}

impl GLVariant {
    pub fn is_desktop(&self) -> bool { *self == GLVariant::Desktop }
    pub fn is_es(&self) -> bool { *self == GLVariant::ES }
}

impl Display for GLVariant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            GLVariant::Desktop => "Desktop",
            GLVariant::ES => "ES",
        })
    }
}


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GLVersion {
    pub variant: GLVariant,
    pub major: GLuint,
    pub minor: GLuint,
}

impl GLVersion {
    pub fn new(variant: GLVariant, major: GLuint, minor: GLuint) -> Self {
        Self { variant, major, minor }
    }
    pub fn current() -> Self {
        Self::from_gl_version_string(&::gl_version_string())
    }
    pub fn from_gl_version_string(mut version_string: &str) -> Self {
        let opengl_es = "OpenGL ES ";
        let variant = if version_string.starts_with(opengl_es) {
            version_string = &version_string[opengl_es.len() ..];
            GLVariant::ES
        } else {
            GLVariant::Desktop
        };
        let mut tokens = version_string.split('.');
        let token = tokens.next().unwrap();
        let major = token.parse().expect("Could not parse GL major version");
        let token = tokens.next().unwrap().split(' ').next().unwrap();
        let minor = token.parse().expect("Could not parse GL minor version");
        Self::new(variant, major, minor)
    }
    fn at_least(&self, major: GLuint, minor: GLuint) -> bool {
        self.major > major || (self.major == self.major && self.minor >= minor)
    }
    pub fn gl(&self, major: GLuint, minor: GLuint) -> bool {
        self.variant == GLVariant::Desktop && self.at_least(major, minor)
    }
    pub fn gles(&self, major: GLuint, minor: GLuint) -> bool {
        self.variant == GLVariant::ES && self.at_least(major, minor)
    }
    pub fn is_desktop(&self) -> bool { self.variant.is_desktop() }
    pub fn is_es(&self) -> bool { self.variant.is_es() }
}

impl Display for GLVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let &Self { variant, major, minor } = self;
        write!(f, "{}.{} {}", major, minor, variant)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gl_version() {
        let versions = [
            ("4.3", GLVariant::Desktop, 4, 3),
            ("4.3 Foo", GLVariant::Desktop, 4, 3),
            ("4.3.523", GLVariant::Desktop, 4, 3),
            ("4.3.523 Foo", GLVariant::Desktop, 4, 3),
            ("OpenGL ES 2.0", GLVariant::ES, 2, 0),
            ("OpenGL ES 2.0 Foo", GLVariant::ES, 2, 0),
        ];
        for (version_string, variant, major, minor) in versions {
            let from_string = GLVersion::from_gl_version_string(version_string);
            let expected = GLVersion::new(variant, major, minor);
            assert_eq!(from_string, expected, "'{}' was parsed as '{}', but '{}' was expected", version_string, from_string, expected);
        }
    }
}


