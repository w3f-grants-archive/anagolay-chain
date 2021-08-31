// This file is part of Anagolay Foundation.

// Copyright (C) 2019-2021 Anagolay Foundation.
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

use anagolay::{CreatorId, ForWhat, GenericId};
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::{clone::Clone, default::Default, vec, vec::Vec};

/// key-value where key is Operation.op and value is fn(Operation)
#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
// #[cfg_attr(feature = "std", derive(Debug))]
pub struct ProofParams {
  /// Operation.name, hex encoded using Parity scale codec
  k: Vec<u8>,
  /// operation Output value serialized using cbor and represented as CID
  v: Vec<u8>,
}

/// Proof Info, this is what gets stored
#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
// #[cfg_attr(feature = "std", derive(Debug))]
pub struct ProofInfo<Proof, AccountId, BlockNumber> {
  pub proof: Proof,
  pub account_id: AccountId,
  pub block_number: BlockNumber,
}

/// PHash Info, what gets stored
#[derive(Encode, Decode, Clone, PartialEq, Default, RuntimeDebug)]
// #[cfg_attr(feature = "std", derive(Debug))]
pub struct PhashInfo {
  pub p_hash: Vec<u8>,
  pub proof_id: GenericId,
}

/// Proof Incoming data
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
// #[cfg_attr(feature = "std", derive(Debug))]
pub struct ProofData {
  pub rule_id: GenericId,
  // which rule is executed
  prev_id: GenericId,
  creator: CreatorId,
  pub groups: Vec<ForWhat>,
  // must be the same as for the rule
  params: Vec<ProofParams>,
}

impl Default for ProofData {
  fn default() -> Self {
    ProofData {
      rule_id: GenericId::default(),
      prev_id: GenericId::default(),
      groups: vec![ForWhat::default()],
      creator: CreatorId::default(),
      params: vec![],
    }
  }
}

/// PoE Proof
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
// #[cfg_attr(feature = "std", derive(Debug))]
pub struct Proof {
  pub id: GenericId,
  // which rule is executed
  pub data: ProofData,
}

impl Default for Proof {
  fn default() -> Self {
    let data = ProofData::default();
    Proof {
      id: b"".to_vec(),
      data,
    }
  }
}
