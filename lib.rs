#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
#[ink::contract]
mod mapper { // Mapping + Voter
    use ink::storage::Mapping;//, env::call::ConstructorReturnType};

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
        pub fn add_voter(&mut self, voter_id: AccountId) {
            assert!(self.env().caller() == self.admin.address);
            assert!(!self.enabled_voters.contains(voter_id));
                        
            self.enabled_voters.insert(voter_id, &());
            self.env().emit_event(NewVoter { voter_id });
        }        

        #[ink(message)]
        pub fn remove_voter(&mut self, voter_id: AccountId) {
            assert!(self.env().caller() == self.admin.address);
            assert!(self.enabled_voters.contains(voter_id));
                        
            self.enabled_voters.remove(voter_id);
            self.env().emit_event(RemoveVoter { voter_id });
        }

        #[ink(message)]
        pub fn get_reputation(&mut self, voter_id: AccountId) -> u32 {
            assert!(self.env().caller() == voter_id);
            assert!(self.enabled_voters.contains(voter_id));
                        
            self.votes.get(voter_id).unwrap_or(0)                      
        }    

        #[ink(message)]
        pub fn vote(&mut self, voter_id: AccountId) {
            assert!(self.enabled_voters.contains(voter_id));
            assert!(self.env().caller() != voter_id);

            let caller = self.env().caller();
            let caller_votes =self.votes.get(caller).unwrap_or(0);
            let power = self.power_of_vote(caller_votes);            
            
            let voter_votes = self.votes.get(voter_id).unwrap_or(0);
            self.votes.insert(voter_id, &(voter_votes + power));

            self.total_votes += power;
            self.env().emit_event(Vote { voter_id });
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