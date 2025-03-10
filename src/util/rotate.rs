/*!

*/

pub trait Rotate {
    type Other;
    fn rot(self) -> Self::Other;
}
impl<T, U> Rotate for (T, U) {
    type Other = (U, T);
    fn rot(self) -> Self::Other {
        let (a, b) = self;
        (b, a)
    }
}

impl<T, U, E> Rotate for Result<(T, U), E> {
    type Other = Result<(U, T), E>;

    fn rot(self) -> Self::Other {
        self.map(|(a, b)| (b, a))
    }
}

impl<T, U> Rotate for Option<(T, U)> {
    type Other = Option<(U, T)>;

    fn rot(self) -> Self::Other {
        self.map(|(a, b)| (b, a))
    }
}

pub trait RotateFn<O, E> {
    fn rot(self) -> impl Fn() -> Result<O, E>;
}

impl<T, U, E, F: Fn() -> Result<(T, U), E>> RotateFn<(U, T), E> for F {
    fn rot(self) -> impl Fn() -> Result<(U, T), E> {
        move || self().map(|(a, b)| (b, a))
    }
}
