#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
#[ink::contract]
mod mapper { // Mapping + Voter
    use ink::storage::Mapping;//, env::call::ConstructorReturnType};
    use scale::{Decode, Encode};

    /// Error management.
    #[derive(PartialEq, Debug, Eq, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotIsAdmin,
        MustBeItSelf,
        VoterAlreadyExists,
        VoterNotExist,
        NotVoteItself,
        NotIsVoter,
    }

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
    }

    #[derive(Debug)]
    #[ink::storage_item]
    pub struct Admin {
        address: AccountId,
        modified_date: u64,
    }

    #[ink(storage)]
    pub struct Mapper {
        admin: Admin,
        votes: Mapping<AccountId, u32>,
        enabled_voters: Mapping<AccountId, ()>,
        total_votes: u32,
    }

    impl Mapper {
        #[ink(constructor)]
        pub fn new(admin: AccountId) -> Self {
            let now = Self::env().block_timestamp();
            Self {
                admin: Admin {
                    address: admin,
                    modified_date: now,
                },
                votes: Mapping::default(),
                enabled_voters: Mapping::default(),
                total_votes: 0,
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
        pub fn get_reputation(&mut self, voter_id: AccountId) -> Result<u32, Error> {
            if self.env().caller() != voter_id {
                return Err(Error::MustBeItSelf);
            }
            if !self.enabled_voters.contains(voter_id) {
                return Err(Error::VoterNotExist);
            }                        
            Ok(self.votes.get(voter_id).unwrap_or(0))
        }    

        #[ink(message)]
        pub fn vote(&mut self, voter_id: AccountId) -> Result<(), Error> {
            if !self.enabled_voters.contains(self.env().caller()) {
                return Err(Error::NotIsVoter);
            }
            if !self.enabled_voters.contains(voter_id) {
                return Err(Error::VoterNotExist);
            }
            if self.env().caller() == voter_id {
                return Err(Error::NotVoteItself);
            }

            let caller = self.env().caller();
            let caller_votes =self.votes.get(caller).unwrap_or(0);
            let power = self.power_of_vote(caller_votes);            
            
            let voter_votes = self.votes.get(voter_id).unwrap_or(0);
            self.votes.insert(voter_id, &(voter_votes + power));

            self.total_votes += power;
            self.env().emit_event(Vote { voter_id });
            Ok(())
        }

        fn power_of_vote(&mut self, votes: u32) -> u32 {
            if self.total_votes == 0 {
                1
            } else {
                let power = (votes * 100)/self.total_votes;
                match power {
                    0...33 => 1,
                    34...66 => 2,
                    _ => 3
                }
            }
        }
    }
}