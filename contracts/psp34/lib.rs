#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::psp34::ContractRef;

#[openbrush::implementation(PSP34)]
#[openbrush::contract]
pub mod psp34 {
    use openbrush::{traits::Storage, contracts::psp34::{self, Id}};

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        psp34: psp34::Data,
        next_id: u8,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[ink(message)]
        pub fn mint_token(&mut self, to: AccountId) -> Result<(), PSP34Error> {
            psp34::Internal::_mint_to(self, to, Id::U8(self.next_id))?;
            self.next_id += 1;
            Ok(())
        }

        #[ink(message)]
        pub fn balance(&self, caller: AccountId) -> u32 {
            psp34::BalancesManagerImpl::_balance_of(self, &caller)
        }
    }
}