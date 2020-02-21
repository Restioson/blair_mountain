//! A crate that exports one macro (`union`) to create types that are enums with checks in debug
//! mode, but unions in release mode. In debug mode, invalid access will panic, while in release
//! **they will not.**  It is therefore unsafe. Fields should be treated as fields of unions (i.e
//! no non-`Copy` types allowed, etc). **Please test all code generated with `blair_mountain` in
//! both release and debug modes.**

/// Define a union.
///
/// **Note: fields must be `Copy`.**
///
/// # Example
///
/// ```rust
/// pub union Example {
///     pub one: &'static str,
///     pub two: u32,
/// }
/// ```

pub use paste::item as paste_item;

#[macro_export]
macro_rules! union {
    {
        $(
            $union_vis:vis union $name:ident {
                $($member_vis:vis $member:ident: $member_type:ty,)*
            }
        )*
    } => {
        $(
            #[cfg(debug_assertions)]
            $crate::paste_item! {
                #[allow(non_camel_case_types)]
                $union_vis enum [<$name Inner>] {
                    $($member($member_type),)*
                }

                #[allow(dead_code)]
                impl $name {
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
                $union_vis union [<$name Inner>] {
                    $($member: $member_type,)*
                }

                #[allow(dead_code)]
                impl $name {
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
                $union_vis struct $name([<$name Inner>]);
            }
        )*
    };
}


#[cfg(test)]
mod tests {
    union! {
        pub union Example {
            pub one: &'static str,
            pub two: u32,
        }
    }

    #[test]
    fn accessors() {
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
    #[should_panic(expected = "unexpected union member")]
    fn invalid_accessors() {
        let eg = Example::new_one("asdfs");
        unsafe { eg.get_two(); }
    }
}
