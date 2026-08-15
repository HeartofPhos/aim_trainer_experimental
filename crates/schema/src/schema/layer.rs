use crate::Team;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Layer {
    None,
    Spawn,
    World,
    Collectable,
    Character(Team),
    Attack(Team),
}

type Bits = u32;

impl Layer {
    pub fn get_bits(&self) -> Bits {
        const TOTAL_BITS: usize = Bits::BITS as usize;
        const RESERVED_COUNT: usize = 3;
        const TEAM_TYPES: usize = 2;
        const TEAM_COUNT: usize = {
            let team_count = Team::TOTAL_BITS;
            let available_team_count = (TOTAL_BITS - RESERVED_COUNT) / TEAM_TYPES;
            assert!(team_count <= available_team_count);

            team_count
        };

        const fn team_type_bit(team_bits: u8, team_type: usize) -> Bits {
            let bit = (team_bits as usize) << (RESERVED_COUNT + TEAM_COUNT * team_type);
            bit as Bits
        }

        #[rustfmt::skip]
            let bits = match self {
                Layer::None => 0,
                Layer::Spawn => 1 << 0,
                Layer::World => 1 << 1,
                Layer::Collectable => 1 << 2,
                Layer::Character(team) => team_type_bit(team.bits(), 0),
                Layer::Attack(team) => team_type_bit(team.bits(), 1),
            };

        bits
    }
}
