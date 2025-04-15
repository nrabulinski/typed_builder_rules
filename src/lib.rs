#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

#[macro_export]
#[doc(hidden)]
macro_rules! __typed_builder_setter_impl {
    (
        $($field:ident $(! $(@ $if_bang:tt)?)? : $ty:ty),* $(,)?
    ) => {
        $crate::__typed_builder_setter_impl!(@
            $($field),* =>
            $($field $(! $($if_bang)?)? : $ty),* =>
        );
    };

    // helper arms to replace Into<T> with T, if the user types field!: T.
    (@@ $ty:ty) => { impl ::std::convert::Into<$ty> };
    (@@! $ty:ty) => { $ty };

    (
        @ $($field:ident),* =>
        =>
        $($rest:ident),*
    ) => {}; // end of the loop

    // Setter generator implementation
    // The code to generate the setter must have a way of replacing the generic argument Tn,
    // which corresponds to the field we're generating the setter for.
    // To my knowledge there's no easy way of doing that with macro_rules currently,
    // so I came up with a workaround:
    // The setter macro holds 3 pieces of "state":
    //   - list of all required fields
    //   - list of required fields we need to generate a setter for
    //   - list of fields we already generated a setter for
    // This is all we need to replace the given type. First, we generate a type alias, which has all the fields in order,
    // *except* for the one we're currently focused on, which ends up first in the list.
    // So we end up with something like
    //   pub type bar<T2, T1, T3, T4> = super::Builder<T1, T2, T3, T4>;
    // Then, because our generic arguments have the same names as the required fields, we can refer to
    // bar<$current_field, ...$already_processed, ...$to_be_processed>,
    // which ends up being substituted with super::Builder<T1, $current_field, T3, T4>.
    (
        @ $($field:ident),* =>
        $head_name:ident $(! $(@ $if_bang:tt)?)? : $head_ty:ty $(, $tail_name:ident $(! $(@ $tail_bang:tt)?)? : $tail_ty:ty)* =>
        $($rest:ident),*
    ) => {
        const _: () = {
            #[allow(dead_code, non_camel_case_types)]
            mod __helper_type {
                pub type $head_name<$head_name $(,$rest)* $(,$tail_name)*> = super::Builder<$($field),+>;
            }

            #[allow(dead_code, non_camel_case_types)]
            impl<$($field),+> Builder<$($field),+> {
                    pub fn $head_name(self, value: $crate::__typed_builder_setter_impl!(@@ $(! $($if_bang)?)? $head_ty)) -> __helper_type::$head_name<$head_ty $(,$rest)* $(,$tail_name)*> {
                    #[allow(unused_variables)]
                    let Builder {
                        $($field),+
                    } = self;
                    Builder {
                        $head_name: (Some(::std::convert::Into::into(value)), ::std::marker::PhantomData),
                        $($rest,)*
                        $($tail_name,)*
                    }
                }
            }
        };

        $crate::__typed_builder_setter_impl!(@ $($field),* => $($tail_name$(! $($tail_bang)?)?: $tail_ty),* => $($rest,)* $head_name);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __typed_builder_impl {
    (
        struct $name:ident {
        // There are 2 main groups of properties
        $(
            @@ // field delimiter, to allow different kinds of fields to match
            $(; $req_field:ident $(! $(@ $req_bang:tt)?)?: $req_ty:ty $(= $default_val:expr)?)? // required fields
            $(; @$priv_field:ident: $priv_ty:ty = $priv_val:expr)? // private fields, always computer at build time
            // TODO: Optional fields?
        )*
        }
    ) => {
        #[allow(dead_code)]
        pub enum Empty {}

        pub struct Builder<$($($req_field = $crate::__typed_builder_impl!(@ $req_ty; $($default_val)?),)?)*> {
            $(
                $($req_field: (Option<$req_ty>, ::std::marker::PhantomData<$req_field>),)?
            )*
        }

        impl Builder {
            pub(super) fn empty() -> Self {
                Self {
                    $(
                        $($req_field: (None, ::std::marker::PhantomData),)?
                    )*
                }
            }
        }

        $crate::__typed_builder_setter_impl!(
            $(
                $($req_field $(! $($req_bang)?)? : $req_ty,)?
            )*
        );

        impl Builder<$($($req_ty,)?)*> {
            pub fn build(self) -> super::$name {
                $(
                    $(let $req_field = self.$req_field.0 $(.or_else(|| Some($default_val)))? .unwrap();)?
                    $(let $priv_field = $priv_val;)?
                )*
                super::$name {
                    $(
                        $($req_field,)?
                        $($priv_field,)?
                    )*
                }
            }
        }
    };

    (@ $ty:ty ; $default_val:expr) => { $ty };
    (@ $ty:ty ;) => { Empty };
}

#[macro_export]
macro_rules! typed_builder {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {$(
            $(#[$field_meta:meta])*
            $(@ $(@ $is_priv:tt)?)? $field_vis:vis $field_name:ident $(! $(@ $if_bang:tt)?)? : $field_ty:ty $(= $field_default:expr)?
        ),* $(,)?}
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name: $field_ty
            ),*
        }

        #[allow(non_camel_case_types)]
        const _: () = {
            mod builder_impl {
                $crate::__typed_builder_impl!(
                    struct $name {$(
                        @@
                        ; $(@ $($is_priv)?)? $field_name$(! $($if_bang)?)?: $field_ty $(= $field_default)?
                    )*}
                );
            }

            impl $name {
                pub fn builder() -> builder_impl::Builder {
                    builder_impl::Builder::empty()
                }
            }
        };
    };
}

#[cfg(test)]
mod tests {
    use crate::typed_builder;

    typed_builder!(
        #[derive(Debug, PartialEq)]
        pub struct Foo {
            hi: String,
            bye: String = String::new(),
            @private: String = String::new(),
        }
    );

    #[test]
    fn it_works() {
        let foo = Foo::builder().hi("wowie").build();
        assert_eq!(
            foo,
            Foo {
                hi: "wowie".to_string(),
                bye: "".to_string(),
                private: "".to_string()
            }
        );
    }
}
