use ink::primitives::AccountId;

use crate::voting::{TypeVote, Error};

#[ink::trait_definition]
pub trait Votingtraits {
    #[ink(message)]
    fn vote(&mut self, voter_id: AccountId, value: TypeVote) -> Result<(), Error>;

    #[ink(message)]
    fn get_reputation(&self, voter_id: AccountId) -> Result<i32, Error>;
}