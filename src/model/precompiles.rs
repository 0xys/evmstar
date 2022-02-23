use super::revision::Revision;

pub const NUM_OF_FRONTIER_CONTRACTS: usize = 4;
pub const NUM_OF_BYZANTIUM_CONTRACTS: usize = 8;
pub const NUM_OF_ISTANBUL_CONTRACTS: usize = 9;

pub fn num_of_precompiles(revision: Revision) -> u8 {
    match revision {
        Revision::Frontier | Revision::Homestead | Revision::Tangerine | Revision::Spurious => {
            NUM_OF_FRONTIER_CONTRACTS as u8
        }
        Revision::Byzantium | Revision::Constantinople | Revision::Petersburg => {
            NUM_OF_BYZANTIUM_CONTRACTS as u8
        }
        Revision::Istanbul | Revision::Berlin | Revision::London | Revision::Shanghai => {
            NUM_OF_ISTANBUL_CONTRACTS as u8
        }
    }
}