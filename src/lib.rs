//! A crate that exports one macro (`union`) to create types that are enums with checks in debug
//! mode, but unions in release mode. In debug mode, invalid access will panic, while in release
//! **they will not.**  It is therefore unsafe. Fields should be treated as fields of unions (i.e
//! no non-`Copy` types allowed, etc). **Please test all code generated with `blair_mountain` in
//! both release and debug modes.**

#[doc(hidden)]
pub use paste::item as paste_item;

/// Define a union. Variants must have trailing commas. Variants must be `Copy`
/// (without [this feature gate](https://github.com/rust-lang/rust/issues/55149)).
/// `Hash`, `Default`, `Debug` (and some other traits) can't be derived for unions either, so do not
/// add `#[derive]` invocations of those above a union.
///
/// This macro creates a struct with the name given and the following methods per variant:
/// - `new_<variant>(val)` - creates a new union with the given variant
/// - `get_<variant>` - gets the union's value as the given variant. Unsound if the union is not
///   of that variant.
/// - `get_<variant>_mut` - gets the union's value as the given variant mutable. For soundness, refer
///   to [this page](https://doc.rust-lang.org/reference/items/unions.html)
/// - `set_<variant>(val)` - sets the union to the given value. The previous variant should be
///   dropped if need be (`ManuallyDrop`)
/// - `into_<variant>` - moves the variant out of the union, consuming the union. Unsound if the
///   union is not of that variant.
///
/// **Note: fields must be `Copy`.**
///
/// # Example
///
/// ```rust,ignore
/// use blair_mountain::union;
///
/// union! {
///     pub union Example {
///         pub one: &'static str,
///         pub two: u32,
///         private: f32,
///     }
///
///     pub union GenericExample<T: Copy, U>
///        where U: Copy + Clone
///     {
///        pub one: T,
///        pub two: U,
///    }
/// }
/// ```
#[macro_export]
macro_rules! union {
    {
        $(
            $(#[$union_meta:meta])*
            $union_vis:vis union $name:ident$(<$($generic:ident $(: $generic_trait:ty)?$(,)?)*>)?
            $(
                where $($where_generic:ident: $($where_bound:ty)+)*
            )?
            {
                $($member_vis:vis $member:ident: $member_type:ty,)*
            }
        )*
    } => {
        $(
            #[cfg(debug_assertions)]
            $crate::paste_item! {
                #[allow(non_camel_case_types)]
                enum [<$name Inner>]$(<$($generic,)*>)?
                $(
                    where $($where_generic: $($where_bound)*)*
                )?
                {
                    $(
                        $member($member_type),
                    )*
                }

                #[allow(dead_code)]
                impl$(<$($generic$(: $generic_trait)?,)*>)? $name$(<$($generic,)*>)?
                $(
                    where $($where_generic: $($where_bound)*)*
                )?
                {
                    $(
                        $member_vis fn [<new_ $member>](val: $member_type) -> Self {
                            Self([<$name Inner>]::$member(val))
                        }

                        $member_vis unsafe fn [<get_ $member>](&self) -> &$member_type {
                            match &self.0 {
                                [<$name Inner>]::$member(val) => val,
                                _ => panic!("unexpected union member")
                            }
                        }

                        $member_vis unsafe fn [<get_ $member _mut>](&mut self) -> &mut $member_type {
                            match &mut self.0 {
                                [<$name Inner>]::$member(val) => val,
                                _ => panic!("unexpected union member")
                            }
                        }

                        $member_vis unsafe fn [<set_ $member>](&mut self, new: $member_type) {
                            self.0 = [<$name Inner>]::$member(new);
                        }

                        $member_vis unsafe fn [<into_ $member>](self) -> $member_type {
                            match self.0 {
                                [<$name Inner>]::$member(val) => val,
                                _ => panic!("unexpected union member")
                            }
                        }
                    )*
                }
            }

            #[cfg(not(debug_assertions))]
            $crate::paste_item! {
                union [<$name Inner>]$(<$($generic$(: $generic_trait)?,)*>)?
                $(
                    where $($where_generic: $($where_bound)*)*
                )?
                {
                    $($member: $member_type,)*
                }

                #[allow(dead_code)]
                impl$(<$($generic$(: $generic_trait)?,)*>)? $name$(<$($generic,)*>)?
                $(
                    where $($where_generic: $($where_bound)*)*
                )?
                {
                    $(
                        $member_vis fn [<new_ $member>](val: $member_type) -> Self {
                            Self([<$name Inner>] {
                                $member: val,
                            })
                        }

                        $member_vis unsafe fn [<get_ $member>](&self) -> &$member_type {
                            &(self.0).$member
                        }

                        $member_vis unsafe fn [<get_ $member _mut>](&mut self) -> &mut $member_type {
                            &mut (self.0).$member
                        }

                        $member_vis unsafe fn [<set_ $member>](&mut self, new: $member_type) {
                            (self.0).$member = new;
                        }

                        $member_vis unsafe fn [<into_ $member>](self) -> $member_type {
                            (self.0).$member
                        }
                    )*
                }
            }

            $crate::paste_item! {
                #[repr(transparent)]
                $(#[$union_meta])*
                $union_vis struct $name$(<$($generic$(: $generic_trait)?,)*>)?(
                        [<$name Inner>]$(<$($generic,)*>)?
                )
                $(
                    where $($where_generic: $($where_bound)*)*
                )?;
            }
        )*
    };
}

/// An example union.
pub mod example {
    union! {
        /// An example union.
        pub union Example {
            pub one: &'static str,
            pub two: u32,
            private: f32,
        }

        /// An example union with generics
        pub union GenericExample<T: Copy, U>
            where U: Copy + Clone
        {
            pub one: T,
            pub two: U,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::example::{GenericExample, Example};

    #[test]
    fn accessors_simple() {
        let mut eg_1 = Example::new_one("asdfs");
        unsafe {
            assert_eq!(*eg_1.get_one(), "asdfs");

            eg_1.set_two(10);
            assert_eq!(*eg_1.get_two(), 10);
        }

        let mut eg_2 = Example::new_two(1234);

        unsafe {
            assert_eq!(*eg_2.get_two(), 1234);

            eg_2.set_two(102);
            assert_eq!(*eg_2.get_two(), 102);

            *eg_2.get_two_mut() = 101;
            assert_eq!(*eg_2.get_two(), 101);

            assert_eq!(eg_2.into_two(), 101);
        }
    }

    #[test]
    fn accessors_generics() {
        let mut eg_1: GenericExample<&'static str, u32> = GenericExample::new_one("asdfs");
        unsafe {
            assert_eq!(*eg_1.get_one(), "asdfs");

            eg_1.set_two(10);
            assert_eq!(*eg_1.get_two(), 10);
        }

        let mut eg_2: GenericExample<&'static str, u32> = GenericExample::new_two(1234);

        unsafe {
            assert_eq!(*eg_2.get_two(), 1234);

            eg_2.set_two(102);
            assert_eq!(*eg_2.get_two(), 102);

            *eg_2.get_two_mut() = 101;
            assert_eq!(*eg_2.get_two(), 101);

            assert_eq!(eg_2.into_two(), 101);
        }
    }

    #[test]
    #[should_panic(expected = "unexpected union member")]
    fn invalid_accessor_get() {
        let eg = Example::new_one("asdfs");
        unsafe { eg.get_two(); }
    }

    #[test]
    #[should_panic(expected = "unexpected union member")]
    fn invalid_accessor_get_mut() {
        let mut eg = Example::new_one("asdfs");
        unsafe { eg.get_two_mut(); }
    }

    #[test]
    #[should_panic(expected = "unexpected union member")]
    fn invalid_accessor_into() {
        let eg = Example::new_one("asdfs");
        unsafe { eg.into_two(); }
    }
}
