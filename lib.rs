#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flipper {

    use ink::prelude::vec::Vec;
    use scale::{Decode, Encode};

    /// Error management.
    #[derive(PartialEq, Debug, Eq, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        WrongRole,
        AdminCannotBeAVoter,
        AdminCannotDeliverVote,
        AdminCannotReciveVote,
        VoterAlreadyExists,
        VoterNotExist,
        NotVoteItself,
        NotIsVoter,
    }
    
    /// Definition type of vote.
    #[derive(PartialEq, Debug, Eq, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]    
    pub enum TypeVote {
        Like,
        Unlike,
    }

    /// Voter definition.
    #[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Voter {
        /// Voter Id.
        voter_id: AccountId,
        /// Reputation of the voter, summary votes received.
        reputation: i8,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Flipper {
        /// Voter administration and including the administrator.
        admin: AccountId,
        voters: Vec<Voter>,
    }

    impl Flipper {
        /// Constructor that initializes the `administrator` value to the given `AccountId` and create the list of voters.
        #[ink(constructor)]
        pub fn new(admin: AccountId) -> Self {
            Self { 
                admin,
                voters: Vec::new(),
            }
        }

        fn get_index(&self, voter_id: AccountId) -> Option<usize> {
            self.voters
                .iter()
                .position(|c| c.voter_id == voter_id)
        }

        /// Add a voter after validation.
        #[ink(message)]
        pub fn add_voter(&mut self, voter_id: AccountId) -> Result<(), Error> {
            if self.admin != self.env().caller() {
                return Err(Error::WrongRole);
            }

            if self.admin == voter_id {
                return Err(Error::AdminCannotBeAVoter);
            }

            if self.get_index(voter_id).is_some() {
                return Err(Error::VoterAlreadyExists);
            }

            self.voters.push(Voter {
                voter_id,
                reputation : 0,
            });
            Ok(())
        }

        /// Remove a voter after validation.
        #[ink(message)]
        pub fn remove_voter(&mut self, voter_id: AccountId) -> Result<(), Error> {
            if self.admin != self.env().caller() {
                return Err(Error::WrongRole);
            }

            if self.admin == voter_id {
                return Err(Error::AdminCannotBeAVoter);
            }

            if self.get_index(voter_id).is_none() {
                return Err(Error::VoterNotExist);
            }

            let index = self.get_index(voter_id);

            self.voters.remove(index.unwrap());
            Ok(())
        }

        /// Send a vote, the caller delivers the vote to the `voting`
        #[ink(message)]
        pub fn vote(&mut self, voting: AccountId, value: TypeVote) -> Result<(), Error> {

            let delivers_id = self.env().caller();

            if self.admin == delivers_id {
                return Err(Error::AdminCannotDeliverVote);
            }

            if self.get_index(delivers_id).is_none() {
                return Err(Error::NotIsVoter);
            }

            if self.admin == voting {
                return Err(Error::AdminCannotReciveVote);
            }

            if delivers_id == voting {
                return Err(Error::NotVoteItself);
            }

            let index = self.get_index(voting);

            if index.is_none() {
                return Err(Error::VoterNotExist);
            }

            let voted = self.voters.get_mut(index.unwrap()).unwrap();
            match value {
                TypeVote::Like => voted.reputation += 1,
                TypeVote::Unlike => voted.reputation -= 1,
            }
            Ok(())
        }

        /// Getting the reputation of a voter.
        #[ink(message)]
        pub fn get_reputation(&self, voter_id: AccountId) -> Result<i8, Error> {

            let index = self.get_index(voter_id);

            if index.is_none() {
                return Err(Error::VoterNotExist);
            }

            let voter = self.voters.get(index.unwrap()).unwrap();
            Ok(voter.reputation)
        }
    }          

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink::env::{test::set_caller, DefaultEnvironment};

        pub struct EnvironmentTest {
            contract: Flipper,
            administrator: AccountId,
            user1: AccountId,
            user2: AccountId,
            user3: AccountId,            
        }

        impl EnvironmentTest {
            pub fn new() -> Self {
                let administrator = AccountId::from([u8::MAX; 32]);
                let contract = Flipper::new(administrator);

                let user1 = AccountId::from([1; 32]);
                let user2 = AccountId::from([2; 32]);
                let user3 = AccountId::from([3; 32]);

                Self {
                    contract,
                    administrator,
                    user1,
                    user2,
                    user3,
                }
            }

            pub fn get_voter(&self, index: usize) -> &Voter {
                self.contract.voters.get(index).unwrap()
            }
        }

        #[ink::test]
        fn constructor_test() {
            let environment = EnvironmentTest::new();

            assert_eq!(environment.contract.admin, environment.administrator);
            assert_eq!(environment.contract.voters.len(), 0);
        }


        #[ink::test]
        fn add_voter_test() {
            let mut environment = EnvironmentTest::new();

            set_caller::<DefaultEnvironment>(environment.administrator);
            assert_eq!(environment.contract.add_voter(environment.user1), Ok(()));
            assert_eq!(environment.contract.add_voter(environment.user1),Err(Error::VoterAlreadyExists));
            assert_eq!(environment.contract.add_voter(environment.administrator),Err(Error::AdminCannotBeAVoter));

            // user1 is voter
            set_caller::<DefaultEnvironment>(environment.user1);
            assert_eq!(environment.contract.add_voter(environment.user2),Err(Error::WrongRole));

            // user2 is not voter
            set_caller::<DefaultEnvironment>(environment.user2);
            assert_eq!(environment.contract.add_voter(environment.user1),Err(Error::WrongRole));
        }

        #[ink::test]
        fn remove_voter_test() {
            let mut environment = EnvironmentTest::new();

            set_caller::<DefaultEnvironment>(environment.administrator);
            let _ = environment.contract.add_voter(environment.user1);

            set_caller::<DefaultEnvironment>(environment.user1);
            assert_eq!(environment.contract.remove_voter(environment.user2),Err(Error::WrongRole));

            set_caller::<DefaultEnvironment>(environment.administrator);
            assert_eq!(environment.contract.remove_voter(environment.user1), Ok(()));
            assert_eq!(environment.contract.remove_voter(environment.user1),Err(Error::VoterNotExist));
            assert_eq!(environment.contract.remove_voter(environment.user2),Err(Error::VoterNotExist));
            assert_eq!(environment.contract.remove_voter(environment.administrator),Err(Error::AdminCannotBeAVoter));
        }

        #[ink::test]
        fn vote_test() {
            let mut environment = EnvironmentTest::new();

            let like = TypeVote::Like;
            let unlike = TypeVote::Unlike;

            set_caller::<DefaultEnvironment>(environment.administrator);
            assert_eq!(environment.contract.voters.len(), 0);
            assert_eq!(environment.contract.add_voter(environment.user1), Ok(()));
            assert_eq!(environment.contract.add_voter(environment.user2), Ok(()));
            assert_eq!(environment.get_voter(1).reputation, 0);
            assert_eq!(environment.get_voter(2).reputation, 0);
            assert_eq!(environment.contract.vote(environment.user1, like.clone()),Err(Error::AdminCannotDeliverVote));

            set_caller::<DefaultEnvironment>(environment.user1);
            assert_eq!(environment.contract.vote(environment.user2, TypeVote::Unlike), Ok(())); // use direct TypeVote::Unlike
            assert_eq!(environment.get_voter(1).reputation, 1);
            assert_eq!(environment.get_voter(2).reputation, 2);
            assert_eq!(environment.contract.vote(environment.user2, like.clone()), Ok(()));
            assert_eq!(environment.get_voter(1).reputation, 1);
            assert_eq!(environment.get_voter(2).reputation, 2);
            assert_eq!(environment.contract.vote(environment.user1, unlike),Err(Error::NotVoteItself));
            assert_eq!(environment.contract.vote(environment.administrator, like.clone()),Err(Error::AdminCannotReciveVote));

            set_caller::<DefaultEnvironment>(environment.user2);
            assert_eq!(environment.contract.vote(environment.user1, like.clone()), Ok(()));
            assert_eq!(environment.get_voter(0).reputation, 1);
            assert_eq!(environment.get_voter(1).reputation, 2);

            set_caller::<DefaultEnvironment>(environment.user1);
            assert_eq!(environment.contract.vote(environment.user3, like.clone()),Err(Error::VoterNotExist));

            set_caller::<DefaultEnvironment>(environment.user3);
            assert_eq!(environment.contract.vote(environment.user1, like),Err(Error::NotIsVoter));
        }

        #[ink::test]
        fn get_reputation_test() {
            let mut environment = EnvironmentTest::new();

            let like = TypeVote::Like;
            let unlike = TypeVote::Unlike;

            set_caller::<DefaultEnvironment>(environment.administrator);
            assert_eq!(environment.contract.voters.len(), 0);
            assert_eq!(environment.contract.add_voter(environment.user1), Ok(()));
            assert_eq!(environment.contract.add_voter(environment.user2), Ok(()));

            set_caller::<DefaultEnvironment>(environment.user1);
            assert_eq!(environment.contract.vote(environment.user2, like), Ok(()));
            assert_eq!(environment.contract.get_reputation(environment.user1), Ok(1));
            assert_eq!(environment.contract.get_reputation(environment.user2), Ok(2));
            assert_eq!(environment.contract.vote(environment.user2, unlike), Ok(()));
            assert_eq!(environment.contract.get_reputation(environment.user1), Ok(1));
            assert_eq!(environment.contract.get_reputation(environment.user2), Ok(2));
            assert_eq!(environment.contract.get_reputation(environment.user3),Err(Error::VoterNotExist));

            set_caller::<DefaultEnvironment>(environment.user2);
            assert_eq!(environment.contract.get_reputation(environment.user1), Ok(1));
            assert_eq!(environment.contract.get_reputation(environment.user2), Ok(2));
            assert_eq!(environment.contract.get_reputation(environment.user3),Err(Error::VoterNotExist));

            set_caller::<DefaultEnvironment>(environment.administrator);
            assert_eq!(environment.contract.get_reputation(environment.user1), Ok(1));
            assert_eq!(environment.contract.get_reputation(environment.user2), Ok(2));
            assert_eq!(environment.contract.get_reputation(environment.user3),Err(Error::VoterNotExist));

            set_caller::<DefaultEnvironment>(environment.user3);
            assert_eq!(environment.contract.get_reputation(environment.user1), Ok(1));
            assert_eq!(environment.contract.get_reputation(environment.user2), Ok(2));
            assert_eq!(environment.contract.get_reputation(environment.user3),Err(Error::VoterNotExist));
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = FlipperRef::default();

            // When
            let contract_account_id = client
                .instantiate("flipper", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<FlipperRef>(contract_account_id.clone())
                .call(|flipper| flipper.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = FlipperRef::new(false);
            let contract_account_id = client
                .instantiate("flipper", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<FlipperRef>(contract_account_id.clone())
                .call(|flipper| flipper.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<FlipperRef>(contract_account_id.clone())
                .call(|flipper| flipper.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<FlipperRef>(contract_account_id.clone())
                .call(|flipper| flipper.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
