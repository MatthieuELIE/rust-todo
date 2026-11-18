/// A normal-mode key that waits for a motion, an object or a priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operator {
    /// `d`: deletes what the motion covers.
    Delete,
    /// `c`: deletes what the motion covers, then inserts.
    Change,
    /// `y`: copies what the motion covers.
    Yank,
    /// `p`: writes the priority typed next.
    Priority,
}

impl Operator {
    /// The operator typed with `key`, if there is one.
    pub(super) const fn from_key(key: char) -> Option<Self> {
        match key {
            'd' => Some(Self::Delete),
            'c' => Some(Self::Change),
            'y' => Some(Self::Yank),
            'p' => Some(Self::Priority),
            _ => None,
        }
    }

    /// The key that types the operator.
    pub(super) const fn key(self) -> char {
        match self {
            Self::Delete => 'd',
            Self::Change => 'c',
            Self::Yank => 'y',
            Self::Priority => 'p',
        }
    }
}
