// This file is part of Anagolay Network.

// Copyright (C) 2019-2023 Anagolay Network.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Test utilities

#![cfg(test)]

use crate as poe;
use crate::{types::PoeVerificationKeyGenerator, Config};
use frame_support::{parameter_types, traits::UnixTime};
use sp_core::{sr25519, sr25519::Signature, H256};
use sp_runtime::{
  testing::{Header, TestXt},
  traits::{BlakeTwo256, IdentityLookup},
};
use std::{
  convert::{TryFrom, TryInto},
  time::Duration,
};
use verification::types::NaiveVerificationInvalidator;

type Extrinsic = TestXt<Call, ()>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        AnagolayTest: anagolay_support::{Pallet, Call, Storage},
        Workflows: workflows::{Pallet, Call, Storage, Event<T>},
        TestPoe: poe::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
  type BaseCallFilter = frame_support::traits::Everything;
  type BlockWeights = ();
  type BlockLength = ();
  type Origin = Origin;
  type Call = Call;
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = sr25519::Public;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type Event = ();
  type BlockHashCount = BlockHashCount;
  type DbWeight = ();
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = ();
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = ();
  type SS58Prefix = SS58Prefix;
  type OnSetCode = ();
  type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl verification::Config for Test {
  type AuthorityId = verification::crypto::VerificationAuthId;
  type Event = ();
  type VerificationKeyGenerator = PoeVerificationKeyGenerator<Self>;
  type VerificationInvalidator = NaiveVerificationInvalidator<Self>;
  type WeightInfo = ();
  type Currency = ();

  const REGISTRATION_FEE: u32 = 10;
  const MAX_REQUESTS_PER_CONTEXT: u32 = 1;
}

type VerificationCall = verification::Call<Test>;
impl From<VerificationCall> for Call {
  fn from(_: VerificationCall) -> Self {
    todo!()
  }
}

impl<VerificationCall> frame_system::offchain::SendTransactionTypes<VerificationCall> for Test
where
  Call: From<VerificationCall>,
{
  type OverarchingCall = Call;
  type Extrinsic = Extrinsic;
}

impl frame_system::offchain::SigningTypes for Test {
  type Public = <Signature as sp_runtime::traits::Verify>::Signer;
  type Signature = Signature;
}

pub struct MockTime {}

impl UnixTime for MockTime {
  fn now() -> Duration {
    core::time::Duration::from_millis(1000)
  }
}

impl Config for Test {
  type Event = ();
  type WeightInfo = ();

  const MAX_PROOFS_PER_WORKFLOW: u32 = 1;
}

impl workflows::Config for Test {
  type Event = ();
  type WeightInfo = ();
  type TimeProvider = MockTime;
  const MAX_VERSIONS_PER_WORKFLOW: u32 = 100;
}

impl anagolay_support::Config for Test {
  const MAX_ARTIFACTS: u32 = 1_000_000;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  frame_system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap()
    .into()
}
