// #[doc(inline)]
// pub use crate::enabled;

// #[doc(inline)]
// pub use crate::disabled;

#[doc(hidden)]
#[macro_export]
macro_rules! enabled {
    () => { true };
    (if { $($p:tt)* } else { $($n:tt)* }) => { $($p)* };
    ($($p:tt)*) => { $($p)*};
}

#[doc(hidden)]
#[macro_export]
macro_rules! disabled {
    () => { false };
    (if { $($p:tt)* } else { $($n:tt)* }) => { $($n)* };
    ($($p:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! switch {
    ({ $($tt:tt)* }) => {{
        $crate::switch! { $($tt)* }
    }};
    (_ => { $($output:tt)* }) => {
        $($output)*
    };
    (
        $cond:path => $output:tt
        $($( $rest:tt )+)?
    ) => {
        $cond! {
            if {
                $crate::switch! { _ => $output }
            } else {
                $(
                    $crate::switch! { $($rest)+ }
                )?
            }
        }
    };
    (
        #[cfg($cfg:meta)] => $output:tt
        $($( $rest:tt )+)?
    ) => {
        #[cfg($cfg)]
        $crate::switch! { _ => $output }
        $(
            #[cfg(not($cfg))]
            $crate::switch! { $($rest)+ }
        )?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! define_alias {
    (
        #[cfg($meta:meta)] => $p:ident
        $(, $( $rest:tt )+)?
    ) => {
        $crate::define_alias! {
            #[cfg($meta)] => { $p }
            $(
                $($rest)+
            )?
        }
    };
    (
        #[cfg($meta:meta)] => $p:ident,
        $($( $rest:tt )+)?
    ) => {
        $crate::define_alias! {
            #[cfg($meta)] => { $p }
            $(
                $($rest)+
            )?
        }
    };
    (
        #[cfg($meta:meta)] => {
            $(#[$p_meta:meta])*
            $p:ident
        }
        $($( $rest:tt )+)?
    ) => {
        $crate::switch! {
            #[cfg($meta)] => {
                $(#[$p_meta])*
                #[doc(inline)]
                ///
                #[doc = concat!("This macro passes the provided code because `#[cfg(", stringify!($meta), ")]` is currently active.")]
                pub use $crate::enabled as $p;
            }
            _ => {
                $(#[$p_meta])*
                #[doc(inline)]
                ///
                #[doc = concat!("This macro suppresses the provided code because `#[cfg(", stringify!($meta), ")]` is _not_ currently active.")]
                pub use $crate::disabled as $p;
            }
        }

        $(
            $crate::define_alias! {
                $($rest)+
            }
        )?
    }
}

define_alias! {
    #[cfg(feature = "alloc")] => {
        /// Indicates the `alloc` crate is available and can be used
        alloc
    }
        #[cfg(feature = "std")] => {
        /// Indicates the `std` crate is available and can be used.
        std
    }
}
