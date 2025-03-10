/// Utility for implementing a trait for many sizes of tuple using declarative macros.
macro_rules! implement_for_tuples (
    ($m:ident) => {
        paste::paste! {
            [<$m>]!(0 . 1 );
            [<$m>]!(0 1 . 2 );
            [<$m>]!(0 1 2 . 3 );
            [<$m>]!(0 1 2 3 . 4 );
            [<$m>]!(0 1 2 3 4 . 5 );
            [<$m>]!(0 1 2 3 4 5 . 6 );
            [<$m>]!(0 1 2 3 4 5 6 . 7 );
            [<$m>]!(0 1 2 3 4 5 6 7 . 8 );
            [<$m>]!(0 1 2 3 4 5 6 7 8 . 9 );
            [<$m>]!(0 1 2 3 4 5 6 7 8 9 . 10 );
            [<$m>]!(0 1 2 3 4 5 6 7 8 9 10 . 11 );
            [<$m>]!(0 1 2 3 4 5 6 7 8 9 10 11 . 12 );
            [<$m>]!(0 1 2 3 4 5 6 7 8 9 10 11 12 . 13 );
            [<$m>]!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 . 14 );
            [<$m>]!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 . 15 );
        }
    }
);

pub(crate) use implement_for_tuples;

/*
 * Child macro template:
 *
macro_rules! foo (
    ($first: literal $($mid: literal)* . $last: literal) => {
        paste::paste! {
            // Your code here.
        }
    }
);

implement_for_tuples!(foo);
*/
