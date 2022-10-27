//! This create provides the [`variants!`] macro, which can be used to create _variants_ of
//! code snippets.
//!
//! All the documentation of how to use that macro are found on the [macro itself](variants!).

/// Creates variants of the same code
///
/// # Syntax
///
/// The syntax is similar to the one of a function or of a macro 2.0:
/// ```plain
/// variants!(
///     #[dollar($)]
///     #[variant(<substitution>, ...)]
///     macro <macro>(<param>, ...) { <code> }
/// )
/// ```
///
/// For each `#[variant]` a whole copy of the `<code>` will be created  where each occurrence of `$param`,
/// is replaced with the contents of the respective `<substitution>`.
/// `<macro>` will be the name of the generated macro, not really useful but it allows nesting calls to [`variants!`].
///
/// If `N` parameters are declared, there can't be less then `N` substitutions for each variant
///
/// An error will be generated when there are not enough substitutions as in the following snippet:
/// ```compile_fail
/// # use variants::variants;
/// variants!(
///     #[dollar($)]
///     #[variant(c)]
///     macro test(a, b) {}
/// );
/// ```
/// This will produce an error similar to this one:
/// ```plain
/// error: unexpected end of macro invocation
///    --> src\lib.rs:286:16
///     |
/// 234 |         macro_rules! $macro {
///     |         ------------------- when calling this macro
/// ...
/// 286 |     #[variant(c)]
///     |                ^ missing tokens in macro arguments
/// ```
///
/// When, instead, there are more substitutions than needed, no error will be generated and the extra ones
/// will just be ignored, as demostrated in the following example:
/// ```
/// # use variants::variants;
/// variants!(
///     #[dollar($)]
///     #[variant(c, d, e)]
///     macro test(a, b) {}
/// );
/// ```
///
/// The macro defined with the given name can be used to choose which code to execute for which variant
///
/// The syntax is `<macro>!(<sub> | <sub> | ... : { <code> }, <sub> : { <code> }, ...)`
///
/// Alternatively `<macro>!(<sub> | <sub> | ... : <code>)` or simply `<macro>!(<sub> : <code>)`
///
/// Where `<sub>` a substitution to match, and `<code>` is some sequence of token that will remain
/// after the macro expansion for the copy with the selected substitution
///
/// Not every substitution must be present, in variants that don't match anything it expands to nothing,
/// and the same substitution can appear more than once (only the first appearance will be evaluated).
/// The special substitution `_` can be used to match any variant.
/// If a substitution that doesn't exist in any variant is added (that is not `_`), the macro will
/// generate a compile time error.
///
/// ## What's the deal with the dollar?
///
/// The generated code will be expanded inside a macro definition, so that the `<param>`s can be
/// _macro meta-variables_.
///
/// The `#[dollar($)]` at the start is needed because of a restriction on macros for which
/// inside a macro body the literal token `$` can't appear, but using a dollar sign is required
/// to declar a macro inside a macro.
/// It won't be needed anymore once [RFC 3086](https://rust-lang.github.io/rfcs/3086-macro-metavar-expr.html)
/// becomes stable.
///
/// If there is the need to use the dollar sign (for example when nesting calls to [`variants!`])
/// `#[dollar($ as <name>)]` can be used.
/// By doing so, every occurrence of `$name` will be replaced with the dollar sign.
/// An example of how to use it it's shown later.
///
/// # Example
///
/// Let's say you have a type that wraps a number and you have to overload the
/// main mathematical operators so that they can be used directly with your type,
/// this can be done easily by using substitution groups of two elements:
/// ```
/// # use variants::variants;
/// # use std::ops::*;
/// # #[derive(PartialEq, Debug)]
/// struct Usize(usize);
///
/// variants!(
///     #[dollar($)]
///     #[variant(Add, add)]
///     #[variant(Sub, sub)]
///     #[variant(Mul, mul)]
///     #[variant(Div, div)]
///     macro operators(op_trait, op_fn) {
///         impl $op_trait for Usize {
///             type Output = Self;
///
///             #[inline]
///             fn $op_fn(self, rhs: Self) -> Self::Output {
///                 Self(usize::$op_fn(self.0, rhs.0))
///             }
///         }
///     }
/// );
///
/// assert_eq!(Usize(1) + Usize(1), Usize(2));
/// assert_eq!(Usize(1) - Usize(1), Usize(0));
/// assert_eq!(Usize(2) * Usize(3), Usize(6));
/// assert_eq!(Usize(6) / Usize(2), Usize(3));
/// ```
///
/// # Using the generated macro
///
/// The following example shows a solution to something that isn't possible in Rust (yet):
/// being generic over the mutability of a reference.
/// Just being generic over that won't help much, be by using the generated macro it's possible
/// to change the behaviour based on the mutability of the reference.
/// ```
/// # use variants::variants;
/// variants!(
///     #[dollar($)]
///     #[variant(add_one_ref, (&usize))]
///     #[variant(add_one_mut, (&mut usize))]
///     macro refmut(name, ty) {
///         fn $name(param: $ty) -> usize {
///             let out = *param + 1;
///             // do something with param ...
///             refmut!{add_one_mut: *param += 1};
///             out
///         }
///     }
/// );
///
/// let mut test = 0;
/// assert_eq!(add_one_ref(&test), 1);
/// assert_eq!(test, 0);
/// assert_eq!(add_one_mut(&mut test), 1);
/// assert_eq!(test, 1);
/// ```
/// This will expand to semthing like this:
/// ```ignore
/// fn add_one_ref<T>(param: &T) {
///     let out = *param + 1;
///     out
/// }
///
/// fn add_one_mut<T>(param: &mut T) {
///     let out = *param + 1;
///     *param += 1;
///     out
/// }
/// ```
///
/// ### Note
///
/// Wrapping the type in parenthesis is needed as a substitution must be a single token-tree
/// but this will produce the warning "unnecessary parentheses around type" so if this
/// is not wanted two solutions can be used:
/// 1) Add `#[allow(unused_parens)]` before the `fn $name`
/// 2) Use a `remove_parens!` macro, which simply removes the parenthesis;\
///    Such a macro can be written like this:
///    ```ignore
///    macro_rules! remove_parens { (($($t:tt)*)) => {$($t)*} }
///    ```
#[macro_export]
macro_rules! variants {
    // NOTE: what is $d?
    // $d must be the dollar sign (`$`) and it's needed to generate macros that take parameters
    // because the dollar sign cannot be used inside a macro definition
    (
        #[dollar($d:tt $(as $dollar:ident)?)]
        #[variant($($sub:tt),+)]
        $(#[variant($($other_sub:tt),+)])*
        macro $macro:ident($($param:ident),+)
        {$($i:tt)*}
    ) => {
        // NOTE: here $d is the same as $
        macro_rules! $macro {
            // Same as: $sub $(| $_:tt)* : { $($t:tt)* } $(, $($__:tt)|+ : { $($___:tt)* })* $(,)?
            $(($sub $d (| $d _:tt)* : { $d ($d t:tt)* } $d (, $d ($d __:tt)|+ : { $d ($d ___:tt)* })* $d (,)?) => {
                // Same as: $($t)*
                $d ($d t)*
            };)+
            // Same as: _ $(| $_:tt)* : { $($t:tt)* } $(, $($__:tt)|+ : { $($___:tt)* })* $(,)?
            (_ $d (| $d _:tt)* : { $d ($d t:tt)* } $d (, $d ($d __:tt)|+ : { $d ($d ___:tt)* })* $d (,)?) => {
                // Same as: $($t)*
                $d ($d t)*
            };
            // Same as: $sv:tt : { $($__:tt)* } $(, $($v:tt)|+ : { $($t:tt)* })* $(,)?
            ($d sv:tt : { $d ($d __:tt)* } $d (, $d ($d v:tt)|+ : { $d ($d t:tt)* })* $d (,)?) => {
                // Same as: $macro!{$($v $(| $va)* : { $($t)* }),*}
                $macro!{$d ($d ($d v)|+ : { $d ($d t)* }),*}
            };
            // Same as: $_:tt $(| $v:tt)+ : { $($t:tt)* } $(, $($ov:tt)|+ : { $($ot:tt)* })* $(,)?
            ($d _:tt $d (| $d v:tt)+ : { $d ($d t:tt)* } $d (, $d ($d ov:tt)|+ : { $d ($d ot:tt)* })* $d (,)?) => {
                // Same as: $macro!{$($v)|+ : { $($t)* }, $($($ov)|+ : { $($ot)* }),*}
                $macro!{$d ($d v)|+ : { $d ($d t)* }, $d ($d ($d ov)|+ : { $d ($d ot)* }),*}
            };
            // Same as: $($v)|+ : $($t:tt)*
            ($d ($d v:tt)|+ : $d ($d t:tt)*) => {
                // Same as: $macro!{$($v)|+ : { $($t)* }}
                $macro!{$d ($d v)|+ : { $d ($d t)* }}
            };
            () => {};
            // NOTE: why not doing everything inside the first matcher?
            // That's because if that was done an extra variable called $d will be avaiable inside the given code
            // which is not wanted, as the only "doller-meta-variable" should be $dollar
            (@$([$d $dollar:tt])?expand $($d $param:tt)+ $d ($d _:tt)*) => { $($i)* };
            // NOTE: this catches the cases when $dollar is not defined, as the [$] is always set
            (@[$d _:tt]expand $($d $param:tt)+ $d ($d __:tt)*) => { $($i)* };
        }
        $macro!{@[$d]expand $($sub)+}

        $crate::variants!{
            #[dollar($d $(as $dollar)?)]
            $(#[variant($($other_sub),+)])*
            macro $macro($($param),+)
            {$($i)*}
        }
    };
    (
        #[dollar($d:tt $(as $dollar:ident)?)]
        macro $macro:ident($($param:ident),+)
        {$($i:tt)*}
    ) => {};
}
