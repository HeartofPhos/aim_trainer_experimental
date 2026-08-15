use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Team(Bits);

type Bits = u8;

impl Team {
    pub const PLAYER: Team = Team(1 << 0);
    pub const BOT: Team = Team(1 << 1);

    pub const NONE: Team = Team(0);
    pub const ALL: Team = Team(!0);

    pub const TOTAL_BITS: usize = Bits::BITS as usize;

    pub fn bits(&self) -> Bits {
        self.0
    }

    pub fn overlaps(a: Team, b: Team) -> bool {
        a.0 & b.0 != 0
    }

    pub fn complement(self) -> Team {
        Team(!self.0)
    }
}
