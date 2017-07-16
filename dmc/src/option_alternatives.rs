//! Alternative `Option` types, which carry a more specific meaning.

macro_rules! option_alternative {
    (
        $(#[$attr:meta])*
        pub enum $name:ident<T> {
            $(#[$with_attr:meta])*
            $with:ident(T),
            $(#[$without_attr:meta])*
            $without:ident,
        }
        $is_with:ident 
        $is_with_some:ident 
        $is_with_none:ident 
        $is_without:ident
    ) => {
        $(#[$attr])*
        #[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
        pub enum $name<T> {
            $(#[$with_attr])*
            $with(T),
            $(#[$without_attr])*
            $without,
        }
        impl<T> Default for $name<T> {
            fn default() -> Self {
                $name::$without
            }
        }
        impl<T> From<T> for $name<T> {
            fn from(val: T) -> Self {
                $name::$with(val)
            }
        }
        impl<T> From<Option<T>> for $name<T> {
            fn from(val: Option<T>) -> Self {
                match val {
                    Some(thing) => $name::$with(thing),
                    None => $name::$without,
                }
            }
        }
        impl<T> $name<T> {
            #[allow(missing_docs)]
            pub fn $is_with(&self) -> bool {
                if let &$name::$with(ref _val) = self {
                    true
                } else { 
                    false
                }
            }
            #[allow(missing_docs)]
            pub fn $is_without(&self) -> bool {
                !self.$is_with()
            }
            #[allow(missing_docs)]
            pub fn into_option(self) -> Option<T> {
                match self {
                    $name::$with(val) => Some(val),
                    $name::$without => None,
                }
            }
            #[allow(missing_docs)]
            pub fn as_ref(&self) -> $name<&T> {
                match *self {
                    $name::$with(ref x) => $name::$with(x),
                    $name::$without => $name::$without,
                }
            }
            #[allow(missing_docs)]
            pub fn as_mut(&mut self) -> $name<&mut T> {
                match *self {
                    $name::$with(ref mut x) => $name::$with(x),
                    $name::$without => $name::$without,
                }
            }
            #[allow(missing_docs)]
            pub fn as_option(&self) -> Option<&T> {
                match *self {
                    $name::$with(ref x) => Some(x),
                    $name::$without => None,
                }
            }
            #[allow(missing_docs)]
            pub fn as_mut_option(&mut self) -> Option<&mut T> {
                match *self {
                    $name::$with(ref mut x) => Some(x),
                    $name::$without => None,
                }
            }
        }
        impl<T> $name<Option<T>> {
            /// Unwrap both this and the underlying `Option`.
            pub fn unwrap_all(self) -> T {
                self.into_option().unwrap().unwrap()
            }
            pub fn unwrap_all_or(self, alt: T) -> T {
                if self.$is_with_some() {
                    self.unwrap_all()
                } else {
                    alt
                }
            }
            pub fn $is_with_some(&self) -> bool {
                match self {
                    &$name::$with(ref opt) => opt.is_some(),
                    &$name::$without => false,
                }
            }
            pub fn $is_with_none(&self) -> bool {
                match self {
                    &$name::$with(ref opt) => opt.is_none(),
                    &$name::$without => false,
                }
            }
        }
        impl<T: Default> $name<Option<T>> {
            pub fn unwrap_all_or_default(self) -> T {
                self.unwrap_all_or(T::default())
            }
        }
    }
}

option_alternative!{
    /// A value which is either known or unknown.
    /// 
    /// APIs should use this instead
    /// of Option<T> when the conveyed meaning is more appropriate.
    #[allow(missing_docs)]
    pub enum Knowledge<T> {
        Known(T),
        Unknown,
    }
    is_known
    is_known_some
    is_known_none
    is_unknown
}


option_alternative!{
    /// Type for values which may be either set manually or left for the
    /// implementation to decide.
    /// 
    /// The actual meaning of `Auto` is often
    /// "best setting", but this is not required as it depends too much
    /// on context.
    /// 
    /// APIs should use this instead
    /// of Option<T> when the conveyed meaning is more appropriate.
    /// 
    /// For a rationale, see [https://english.stackexchange.com/a/203664](https://english.stackexchange.com/a/203664)
    #[allow(missing_docs)]
    pub enum Decision<T> {
        Manual(T),
        Auto,
    }
    is_manual
    is_manual_some
    is_manual_none
    is_auto
}
