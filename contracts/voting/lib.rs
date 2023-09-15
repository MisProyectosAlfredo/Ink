#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod votingtraits;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
#[ink::contract]
mod voting {

    use ink::prelude::vec::Vec;
    use psp34::psp34::ContractRef;
    use crate::votingtraits::Votingtraits;
    use ink::storage::Mapping;
    use scale::{Decode, Encode};

    #[ink(event)]
    pub struct NewVoter {
        #[ink(topic)]
        voter_id: AccountId,
    }

    #[ink(event)]
    pub struct RemoveVoter {
        #[ink(topic)]
        voter_id: AccountId,
    }

    #[ink(event)]
    pub struct Vote {
        #[ink(topic)]
        voter_id: AccountId,
        #[ink(topic)]
        total_votes: i32,
        #[ink(topic)]
        votation: TypeVote,  
    }

    #[derive(Debug)]
    #[ink::storage_item]
    pub struct Admin {
        address: AccountId,
        modified_date: u64,
    }

    /// Error management.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotIsAdmin,
        MustBeItSelf,
        VoterAlreadyExists,
        VoterNotExist,
        NotVoteItSelf,
        NotIsVoter,
        NftNotMint,
    }

    /// Definition type of vote.
    #[derive(PartialEq, Debug, Eq, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum TypeVote {
        Like,
        Unlike,
    }

    #[ink(storage)]
    pub struct Voting {
        admin: Admin,
        votes: Mapping<AccountId, i32>,
        enabled_voters: Mapping<AccountId, ()>,
        total_votes: i32,
        contract: ContractRef,
    }

    impl Voting {
        #[ink(constructor)]
        pub fn new(admin: AccountId, contract_code_hash: Hash) -> Self {
            let now = Self::env().block_timestamp();
            Self {
                admin: Admin {
                    address: admin,
                    modified_date: now,
                },
                votes: Mapping::default(),
                enabled_voters: Mapping::default(),
                total_votes: 0,
                contract: ContractRef::new()
                    .code_hash(contract_code_hash)
                    .endowment(0)
                    .salt_bytes(Vec::new()) // Sequence of bytes
                    .instantiate(),

            }
        }

        #[ink(message)]
        pub fn add_voter(&mut self, voter_id: AccountId) -> Result<(), Error> {
            if self.env().caller() != self.admin.address {
                return Err(Error::NotIsAdmin);
            }
            if self.enabled_voters.contains(voter_id) {
                return Err(Error::VoterAlreadyExists);
            }

            self.enabled_voters.insert(voter_id, &());
            self.env().emit_event(NewVoter { voter_id });
            Ok(())
        }

        #[ink(message)]
        pub fn remove_voter(&mut self, voter_id: AccountId) -> Result<(), Error> {
            if self.env().caller() != self.admin.address {
                return Err(Error::NotIsAdmin);
            }
            if !self.enabled_voters.contains(voter_id) {
                return Err(Error::VoterNotExist);
            }

            self.enabled_voters.remove(voter_id);
            self.env().emit_event(RemoveVoter { voter_id });
            Ok(())
        }

        #[ink(message)]
        pub fn vote(&mut self, voter_id: AccountId, value: TypeVote) -> Result<(), Error> {
            if !self.enabled_voters.contains(self.env().caller()) {
                return Err(Error::NotIsVoter);
            }
            if !self.enabled_voters.contains(voter_id) {
                return Err(Error::VoterNotExist);
            }
            if self.env().caller() == voter_id {
                return Err(Error::NotVoteItSelf);
            }

            let caller = self.env().caller();
            let caller_votes = self.votes.get(caller).unwrap_or(0);
            let power = self.power_of_vote(caller_votes);

            let voter_votes = self.votes.get(voter_id).unwrap_or(0);

            if value == TypeVote::Like {
                self.votes.insert(voter_id, &(voter_votes + power));
            } else {
                self.votes.insert(voter_id, &(voter_votes - power));
            }
            
            let resultmint = self.contract.mint_token(caller);

            if resultmint.is_err() {
                return Err(Error::NftNotMint);
            }

            if power == 0 {
                self.total_votes += 1;
            } else {
                self.total_votes += power;
            }

            self.env().emit_event(Vote { voter_id, total_votes: self.total_votes, votation: value});
            Ok(())
        }

        #[ink(message)]
        pub fn get_reputation(&self, voter_id: AccountId) -> Result<i32, Error> {
            if self.env().caller() != voter_id {
                return Err(Error::MustBeItSelf);
            }
            if !self.enabled_voters.contains(voter_id) {
                return Err(Error::VoterNotExist);
            }
            Ok(self.votes.get(voter_id).unwrap_or(0))
        }

        #[ink(message)]
        pub fn get_balance(&self, voter_id: AccountId) -> Result<u32, Error> {
            if self.env().caller() != voter_id {
                return Err(Error::MustBeItSelf);
            }
            if !self.enabled_voters.contains(voter_id) {
                return Err(Error::VoterNotExist);
            }
            Ok(self.contract.balance(voter_id))            
        }

        fn power_of_vote(&mut self, votes: i32) -> i32 {
            if self.total_votes == 0 {
                1
            } else {
                let power = (votes * 100)/self.total_votes;
                match power {
                    i if i < 0 => 0,
                    0...33 => 1,
                    34...66 => 2,
                    _ => 3
                }
            }
        }
    }

    impl Votingtraits for Voting {
    
        #[ink(message)]
        fn vote(&mut self, voter_id: AccountId, value: TypeVote) -> Result<(), Error> {        
            self.vote(voter_id, value).unwrap();
            Ok(())
        }

        #[ink(message)]
        fn get_reputation(&self, voter_id: AccountId) -> Result<i32, Error> {
            Ok(self.get_reputation(voter_id).unwrap_or(0))
        }
    }
}