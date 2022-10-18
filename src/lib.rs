//! This create provides the [`duplicate!`] macro, which can be used to create _duplicate_ of
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
//! The [`duplicate!`] marco can be used for many such cases with the advantage that the code can be seen directly
//! by the linter and there won't be any macro expansion error (so long that the caller follows the syntax
//! described)

/// Creates duplicate of the same code
///
/// The syntax is: `[$] <substituition variables>; <substitution group>; ...  { <code> }`
/// Where:
/// - `<substituition variables>` = `[<var #0>] [<var #1>] ...`
/// - `<substitution group>` = `[sub. for var #0] [sub. for var #1] ...`
///
/// **Note** that the `[$]` at the start is only needed because of a restriction on macros.
/// It won't be needed anymore once [RFC 3086](https://rust-lang.github.io/rfcs/3086-macro-metavar-expr.html)
/// becomes stable.
///
/// For each `<substitution group>` a whole copy of the `<code>` will be created  where each occurrence of `$($var)*`,
/// where `var` is one of the is `<substituition variables>`, is replaced with the contents of the respective
/// `<substitution>`.
///
/// **Note** that the generated code will be expanded inside a macro definition,
/// so the code can't contain dollar signes that are not meta-variables.
/// In fact the `<var>`s are actually _macro meta-variables_
///
/// # `select!`
///
/// Select is a utility macro which can be used to chose which code to execute for which variant
///
/// The syntax is `[<sub>] | [<sub>] | ... : { <code> }, [<sub>] : { <code> }, ...`
///
/// Alternatively `[<sub>] | [<sub>] | ... : <code>` or simply `[<sub>] : <code>`
///
/// Where `<sub>` a substitution to match, and `<code>` is some sequence of token that will remain
/// after the macro expansion for the copy with the _selected_ substitution
///
/// **Note** that not every substitution must be present, and the same substitution can
/// appear more than once (only the first appearance will be evaluated)
///
/// The special substitution `_` can be used to match any variant
///
/// # Example
///
/// To give a basic example we can think of two function that do somewhat the same thing but one
/// takes a shared reference and the other a mutable reference.
/// Being generic over the mutability of a reference is not possible yet so this crate provides
/// a simple solution:
/// ```
/// # use duplicate_decl::duplicate;
/// duplicate!([$] [name]; [fn_ref]; [fn_mut]; {
///     fn $($name)* <T>(param: select!([fn_ref]: {&T}, [fn_mut]: {&mut T})) {
///         // do something with param ...
///
///         select!{[fn_mut]: /* mutate param */};
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
/// ## Using _or_ in `select!`
///
/// As explained in the syntax refernece, with `select!` more than one
/// substitution can be mathed for the same piece of code.
/// The following example uses this feature by adding a `fn_own` substitution
/// to the previous example:
/// ```
/// # use duplicate_decl::duplicate;
/// duplicate!([$] [name]; [fn_own]; [fn_ref]; [fn_mut]; {
///     fn $($name)*<T>(param: select!([fn_own]: {T}, [fn_ref]: {&T}, [fn_mut]: {&mut T})) {
///         // do something with param ...
///
///         select!(
///             [fn_mut] | [fn_ref]: { /* use the reference */ },
///             [fn_own]: { /* use the value */ },
///         );
///     }
/// });
/// ```
///
/// ## Aliases
///
/// Let's say you have a type that wraps a number and you have to overload the main
/// arithmetical operators so that they can be used directly with your type,
/// this can be done easily by using substitution groups of two elements:
/// ```
/// # use duplicate_decl::duplicate;
/// # use std::ops::*;
/// struct Usize(usize);
///
/// duplicate!([$] [tr][op]; [Add][add]; [Sub][sub]; [Mul][mul]; [Div][div]; {
///     impl $($tr)* for Usize {
///         type Output = Self;
///
///         #[inline]
///         fn $($op)*(self, rhs: Self) -> Self::Output {
///             Self(usize::$($op)*(self.0, rhs.0))
///         }
///     }
/// });
/// ```
/// **Note** that if `N` variables are declared, there can't be less then `N` substitutions for each group
///
/// An error will be generated when there are not enough substitutions as in the following snippet:
/// ```compile_fail
/// # use duplicate_decl::duplicate;
/// duplicate!([$] [a][b]; [c]; {});
/// ```
///
/// When, instead, there are more substitutions than needed, no error will be generated, as demostrated in the following example:
/// ```
/// # use duplicate_decl::duplicate;
/// duplicate!([$] [a][b]; [c][d][e]; {});
/// ```
/// **Note** that the `[e]` won't be considered
/// 
/// ## Using arbitrary tokens
/// 
/// The first example could be written like this:
/// ```
/// # use duplicate_decl::duplicate;
/// duplicate!([$] [name][ref]; [fn_ref][&]; [fn_mut][&mut]; {
///     fn $($name)* <T>(param: $($ref)* T) {
///         // do something with param ...
///
///         select!{[fn_mut]: /* mutate param */};
///     }
/// });
/// ```
#[macro_export]
macro_rules! duplicate {
    // NOTE: what is $d?
    // $d must be the dollar sign (`$`) and it's needed to generate macros that take parameters
    // because the dollar sign cannot be used inside a macro definition
    ([$d:tt] $([$name:ident])+; $( $([$($sub:tt)*])+; )* {$($i:tt)*}) => {
        // NOTE: here $d is the same as $
        macro_rules! variant {
            // NOTE: what is $d d?
            // $d d, like for $d, is the dollar sign.
            // This has to be done to achive two-level-deep nesting of macros with parameters:
            // like for the main macro a dollar sign has to be passed in order to make sub-macros
            // that take parameters, but in defining this dollar-sign-meta-var the main
            // one ($d) as to be used, this $d d is the inner macro `$`
            $(([$d d:tt] $([$($sub)*])+) => {
                // NOTE: here $d d is the same as $
                macro_rules! select {
                    // Same as:
                    // [$($sub)*] $(| [$($_:tt)*])* : { $($t:tt)* }
                    // $(, $([$($__:tt)*])|+ : { $($___:tt)* })* $(,)?
                    $((
                        [$($sub)*] $d d (| [$d d ($d d _:tt)*])* : { $d d ($d d t:tt)* }
                        $d d (, $d d ([$d d ($d d __:tt)*])|+ : { $d d ($d d ___:tt)* })* $d d (,)?
                    ) => {
                        // Same as: $($t)*
                        $d d ($d d t)*
                    };)+
                    // Same as: _ $(| [$($_:tt)*])* : { $($t:tt)* } $(, $([$($__:tt)*])|+ : { $($___:tt)* })* $(,)?
                    (_ $d d (| [$d d ($d d _:tt)*])* : { $d d ($d d t:tt)* } $d d (, $d d ([$d d ($d d __:tt)*])|+ : { $d d ($d d ___:tt)* })* $d d (,)?) => {
                        // Same as: $($t)*
                        $d d ($d d t)*
                    };
                    // Same as: [$($_:tt)*] : { $($__:tt)* } $(, $([$($v:tt)*])|+ : { $($t:tt)* })* $(,)?
                    ([$d d ($d d _:tt)*] : { $d d ($d d __:tt)* } $d d (, $d d ([$d d ($d d v:tt)*])|+ : { $d d ($d d t:tt)* })* $d d (,)?) => {
                        // Same as: select!{$($([$($v)*])|+ : { $($t)* }),*}
                        select!{$d d ($d d ([$d d ($d d v)*])|+ : { $d d ($d d t)* }),*}
                    };
                    // Same as: [$($_:tt)*] $(| [$($v:tt)*])+ : { $($t:tt)* } $(, $([$($ov:tt)*])|+ : { $($ot:tt)* })* $(,)?
                    ([$d d ($d d _:tt)*] $d d (| [$d d ($d d v:tt)*])+ : { $d d ($d d t:tt)* } $d d (, $d d ([$d d ($d d ov:tt)*])|+ : { $d d ($d d ot:tt)* })* $d d (,)?) => {
                        // Same as: select!{$([$($v)*])|+ : { $($t)* }, $($([$($ov)*])|+ : { $($ot)* }),*}
                        select!{$d d ([$d d ($d d v)*])|+ : { $d d ($d d t)* }, $d d ($d d ([$d d ($d d ov)*])|+ : { $d d ($d d ot)* }),*}
                    };
                    // Same as: $([$($v:tt)*])|+ : $($t:tt)*
                    ($d d ([$d d ($d d v:tt)*])|+ : $d d ($d d t:tt)*) => {
                        // Same as: select!{$([$($v)*])|+ : { $($t)* }}
                        select!{$d d ([$d d ($d d v)*])|+ : { $d d ($d d t)* }}
                    };
                    () => {};
                }
                variant!{$([$($sub)*])+}
            };)*
            ($([$d ($d $name:tt)*])+ $d ($d _:tt)*) => { $($i)* };
        }
        $(variant!{[$d] $([$($sub)*])+})*
    };
}
