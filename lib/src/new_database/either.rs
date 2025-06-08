use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

// impl<L, R> Copy for Either<L, R>
// where
//     L: Copy,
//     R: Copy,
// {
// }

// impl<L, R> Clone for Either<L, R>
// where
//     L: Clone,
//     R: Clone,
// {
//     fn clone(&self) -> Self {
//         match self {
//             Self::Left(arg0) => Self::Left(arg0.clone()),
//             Self::Right(arg0) => Self::Right(arg0.clone()),
//         }
//     }
// }

// impl<L, R> Debug for Either<L, R>
// where
//     L: Debug,
//     R: Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Left(arg0) => f.debug_tuple("Left").field(arg0).finish(),
//             Self::Right(arg0) => f.debug_tuple("Right").field(arg0).finish(),
//         }
//     }
// }
