// Copyright 2019-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

pub use self::delegator::Delegator;
use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod delegator {
    use accumulator::Accumulator;
    use adder::Adder;
    use ink_core::storage2::{
        traits::{
            PackedLayout,
            SpreadLayout,
        },
        Lazy,
    };
    use subber::Subber;

    /// Specifies the state of the delegator.
    ///
    /// In `Adder` state the delegator will delegate to the `Adder` contract
    /// and in `Subber` state will delegate to the `Subber` contract.
    ///
    /// The initial state is `Adder`.
    #[derive(
        Debug,
        Copy,
        Clone,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
        SpreadLayout,
        PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(::type_metadata::Metadata, ::ink_core::storage2::traits::StorageLayout,)
    )]
    pub enum Which {
        Adder,
        Subber,
    }

    /// Delegates calls to an adder or subber contract to mutate
    /// a value in an accumulator contract.
    ///
    /// In order to deploy the delegator smart contract we first
    /// have to manually put the code of the accumulator, adder
    /// and subber smart contracts, receive their code hashes from
    /// the signalled events and put their code hash into our
    /// delegator smart contract.
    #[ink(storage)]
    struct Delegator {
        /// Says which of adder or subber is currently in use.
        which: Which,
        /// The accumulator smart contract.
        accumulator: Lazy<Accumulator>,
        /// The adder smart contract.
        adder: Lazy<Adder>,
        /// The subber smart contract.
        subber: Lazy<Subber>,
    }

    impl Delegator {
        /// Instantiate a delegator with the given sub-contract codes.
        #[ink(constructor)]
        fn new(
            init_value: i32,
            accumulator_code_hash: Hash,
            adder_code_hash: Hash,
            subber_code_hash: Hash,
        ) -> Self {
            let total_balance = Self::env().balance();
            let accumulator = Accumulator::new(init_value)
                .endowment(total_balance / 4)
                .using_code(accumulator_code_hash)
                .instantiate()
                .expect("failed at instantiating the `Accumulator` contract");
            let adder = Adder::new(accumulator.clone())
                .endowment(total_balance / 4)
                .using_code(adder_code_hash)
                .instantiate()
                .expect("failed at instantiating the `Adder` contract");
            let subber = Subber::new(accumulator.clone())
                .endowment(total_balance / 4)
                .using_code(subber_code_hash)
                .instantiate()
                .expect("failed at instantiating the `Subber` contract");
            Self {
                which: Which::Adder,
                accumulator: Lazy::new(accumulator),
                adder: Lazy::new(adder),
                subber: Lazy::new(subber),
            }
        }

        /// Returns the accumulator's value.
        #[ink(message)]
        fn get(&self) -> i32 {
            self.accumulator.get()
        }

        /// Delegates the call to either `Adder` or `Subber`.
        #[ink(message)]
        fn change(&mut self, by: i32) {
            match self.which {
                Which::Adder => self.adder.inc(by),
                Which::Subber => self.subber.dec(by),
            }
        }

        /// Switches the delegator.
        #[ink(message)]
        fn switch(&mut self) {
            match self.which {
                Which::Adder => {
                    self.which = Which::Subber;
                }
                Which::Subber => {
                    self.which = Which::Adder;
                }
            }
        }
    }
}
