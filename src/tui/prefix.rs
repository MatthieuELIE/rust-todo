/// The first key of a two-key command of the list or the help, waiting for its second key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Prefix {
    /// `dd`.
    D,
    /// `gg` and `gx`.
    G,
    /// `p` and a letter.
    P,
    /// `yy`.
    Y,
    /// `zM`, `zR` and `za`.
    Z,
}

impl Prefix {
    /// The key typed.
    pub(super) const fn key(self) -> char {
        match self {
            Prefix::D => 'd',
            Prefix::G => 'g',
            Prefix::P => 'p',
            Prefix::Y => 'y',
            Prefix::Z => 'z',
        }
    }
}

#[cfg(test)]
mod tests;
