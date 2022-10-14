//! This create provides the [`variants!`] macro, which can be used to create _variants_ of
//! code snippets.
//! 
//! All the documentation of how to use that macro are found on the macro itself.
//! 
//! # Why use it?
//! 
//! Sometimes a lot of boilerplate needs to be written, and, be it because of some feature missing
//! in Rust itself or on some project specific restrictions, it cannot be avoided.
//! 
//! Most of the times Rust programmers spend some time to write `macros!` to reduce the amount of duplicate code,
//! but it doing so they expose themseleves to very criptic errors arising from macro expansion and worst of all
//! they lose the help of the linter, because a macro cannot be checked until it's called.
//! 
//! The [`variants!`] marco can be used for many such cases with the advantage that the code can be seen directly
//! by the linter and there won't be any macro expansion error (so long that the caller follows the syntax
//! described)



/// Creates variants of the same code
///
/// The syntax is: `[$] <variable> or <variable> : <variant> or <alias> or <alias>, <variant>, ... => { <code> }`
/// 
/// **Note** that the `[$]` at the start is only needed because of a restriction on macros.
/// It won't be needed anymore once [RFC 3086](https://rust-lang.github.io/rfcs/3086-macro-metavar-expr.html)
/// becomes stable.
///
/// For each `<varaint>` a whole copy of the code will be created.
/// 
/// Each variant can have as many aliases as wanted. Each alias will be matched with the corresponding
/// `<variable>`.
/// 
/// `<variable>` can be accessed inside `<code>` with `$<variable>` and it will expand to the corresponding
/// alias for each variant.
///
/// **Note** that the generated code will be expanded inside a macro definition,
/// so the code can't contain dollar signes that are not meta-variables.
/// In fact the `<variable>`s are actually macro meta-variables
///
/// # `select!`
///
/// Select is a utility macro which can be used to chose which code to execute for which variant
///
/// The syntax is `<name> | <name> | ... : { <code> }, <name> : { <code> }, ...`
///
/// Alternatively `<name> | <name> | ... : <code>` or simply `<name> : <code>`
///
/// Where `<name>` is the name of the variant to match (or of one of it's aliases),
/// and `<code>` is some sequence of token that will remain after the macro expansion
/// for the variants with the given names
///
/// **Note** that not every variant must be present, and the same variant can
/// appear more than once (only the first appearance will be evaluated)
///
/// The special variant `_` can be used to match any variant
///
/// # Example
/// 
/// To give a basic example we can think of two function that do somewhat the same thing but one
/// takes a shared reference and the other a mutable reference.
/// Being generic over the mutability of a reference is not possible yet so this crate provides
/// a simple solution:
/// ```
/// # use variant::variants;
/// 
/// variants!([$] name: fn_ref, fn_mut => {
///     fn $name<T>(param: select!(fn_ref: {&T}, fn_mut: {&mut T})) {
///         // do something with param ...
/// 
///         select!{fn_mut: /* mutate param */};
///     }
/// });
/// ```
/// This will expand to semthing like this:
/// ```ignore
/// fn fn_ref<T>(param: &T) {
///     // do something with param ...
/// }
/// 
/// fn fn_mut<T>(param: &mut T) {
///     // do something with param ...
///     /* mutate param */
/// }
/// ```
///
/// ## Aliases
///
/// Let's say you have a type that wraps a number and you have to overload the main
/// arithmetical operators so that they can be used directly with your type,
/// this can be done easily by using variant aliases
/// ```
/// # use variant::variants;
/// # use std::ops::*;
/// struct Usize(usize);
/// 
/// variants!([$] tr | op: Add | add, Sub | sub, Mul | mul, Div | div => {
///     impl $tr for Usize {
///         type Output = Self;
/// 
///         #[inline]
///         fn $op(self, rhs: Self) -> Self::Output {
///             Self(usize::$op(self.0, rhs.0))
///         }
///     }
/// });
/// ```
/// **Note** that if `N` names are declared, there can't be less then `N` names for each variant
/// 
/// When there are not enough names as in the following snippet:
/// ```compile_fail
/// # use variant::variants;
/// variants!([$] a | b: c => {});
/// ```
/// An error like this will be generated:
/// ```plain
/// error: unexpected end of macro invocation
///  --> src\lib.rs:111:23
///   |
/// 5 | variants!([$] a | b: c => {});
///   |                       ^ missing tokens in macro arguments
/// ```
/// 
/// When, instead, there are more names than needed, no error will be generated, as demostrated in the following example:
/// ```
/// # use variant::variants;
/// variants!([$] a | b: c | d | e => {});
/// ```
#[macro_export]
macro_rules! variants {
    // NOTE: what is $d?
    // $d must be the dollar sign (`$`) and it's needed to generate macros that take parameters
    // because the dollar sign cannot be used inside a macro definition
    ([$d:tt] $($name:tt)|+ : $( $($var:tt)|+ ),* => {$($i:tt)*}) => {
        // NOTE: here $d is the same as $
        macro_rules! variant {
            // NOTE: what is $d d?
            // $d d, like for $d, is the dollar sign.
            // This has to be done to achive two-level-deep nesting of macros with parameters:
            // like for the main macro a dollar sign has to be passed in order to make sub-macros
            // that take parameters, but in defining this dollar-sign-meta-var the main
            // one ($d) as to be used, this $d d is the inner macro `$`
            $(([$d d:tt] $($var)+) => {
                // NOTE: here $d d is the same as $
                macro_rules! select {
                    // Same as: $var : { $($t:tt)* } $(, $($_:tt)|+ : { $($__:tt)* })* $(,)?
                    $(($var : { $d d ($d d t:tt)* } $d d (, $d d ($d d _:tt)|+ : { $d d ($d d __:tt)* })* $d d (,)?) => {
                        // Same as: $($t)*
                        $d d ($d d t)*
                    };)+
                    // Same as: _ : { $($t:tt)* } $(, $($_:tt)|+ : { $($__:tt)* })* $(,)?
                    (_ : { $d d ($d d t:tt)* } $d d (, $d d ($d d _:tt)|+ : { $d d ($d d __:tt)* })* $d d (,)?) => {
                        // Same as: $($t)*
                        $d d ($d d t)*
                    };
                    // Same as: $_:tt : { $($__:tt)* } $(, $($v:tt)|+ : { $($t:tt)* })* $(,)?
                    ($d d _:tt : { $d d ($d d __:tt)* } $d d (, $d d ($d d v:tt)|+ : { $d d ($d d t:tt)* })* $d d (,)?) => {
                        // Same as: select!{$($v $(| $va)* : { $($t)* }),*}
                        select!{$d d ($d d ($d d v)|+ : { $d d ($d d t)* }),*}
                    };
                    // Same as: $($v:tt)|+ : { $($t:tt)* } $(, $($ov:tt)|+ : { $($ot:tt)* })* $(,)?
                    ($d d ($d d v:tt)|+ : { $d d ($d d t:tt)* } $d d (, $d d ($d d ov:tt)|+ : { $d d ($d d ot:tt)* })* $d d (,)?) => {
                        // Same as: select!{$($v : { $($t)* }),+ $(, $($ov)|+ : { $($ot)* }),*}
                        select!{$d d ($d d v : { $d d ($d d t)* }),+ $d d (, $d d ($d d ov)|+ : { $d d ($d d ot)* }),*}
                    };
                    // Same as: $($var)|+ : $($t:tt)*
                    ($d d ($d d v:tt)|+ : $d d ($d d t:tt)*) => {
                        // Same as: select!{$($v)|+ : { $($t)* }}
                        select!{$d d ($d d v)|+ : { $d d ($d d t)* }}
                    };
                    () => {};
                }
                variant!{$($var)|+};
            };)*
            ($($d $name:tt)|+ $d ($d _:tt)*) => { $($i)* };
        }
        $(variant!{[$d] $($var)+})*
    };
}
