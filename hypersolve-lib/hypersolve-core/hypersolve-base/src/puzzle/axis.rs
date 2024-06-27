/// Axes `X`, `Y`, `Z`, and `W`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, const_gen::CompileConst)]
#[repr(u8)]
pub enum Axis {
    #[default]
    X = 0,
    Y = 1,
    Z = 2,
    W = 3,
}
