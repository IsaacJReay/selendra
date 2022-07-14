// This file is part of Selendra.

// Copyright (C) 2021-2022 Selendra.
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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-06-20, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bm4`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("polkadot-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/selendra
// benchmark
// pallet
// --chain=selendra-dev
// --steps=50
// --repeat=20
// --pallet=pallet_democracy
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --output=./runtime/polkadot/src/weights/pallet_elections_phragmen.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_elections_phragmen`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_elections_phragmen::WeightInfo for WeightInfo<T> {
	// Storage: PhragmenElection Candidates (r:1 w:0)
	// Storage: PhragmenElection Members (r:1 w:0)
	// Storage: PhragmenElection RunnersUp (r:1 w:0)
	// Storage: PhragmenElection Voting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	/// The range of component `v` is `[1, 16]`.
	fn vote_equal(v: u32, ) -> Weight {
		(24_689_000 as Weight)
			// Standard Error: 5_000
			.saturating_add((206_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PhragmenElection Candidates (r:1 w:0)
	// Storage: PhragmenElection Members (r:1 w:0)
	// Storage: PhragmenElection RunnersUp (r:1 w:0)
	// Storage: PhragmenElection Voting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	/// The range of component `v` is `[2, 16]`.
	fn vote_more(v: u32, ) -> Weight {
		(37_520_000 as Weight)
			// Standard Error: 8_000
			.saturating_add((192_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PhragmenElection Candidates (r:1 w:0)
	// Storage: PhragmenElection Members (r:1 w:0)
	// Storage: PhragmenElection RunnersUp (r:1 w:0)
	// Storage: PhragmenElection Voting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	/// The range of component `v` is `[2, 16]`.
	fn vote_less(v: u32, ) -> Weight {
		(37_783_000 as Weight)
			// Standard Error: 10_000
			.saturating_add((168_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PhragmenElection Voting (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	fn remove_voter() -> Weight {
		(34_185_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PhragmenElection Candidates (r:1 w:1)
	// Storage: PhragmenElection Members (r:1 w:0)
	// Storage: PhragmenElection RunnersUp (r:1 w:0)
	/// The range of component `c` is `[1, 200]`.
	fn submit_candidacy(c: u32, ) -> Weight {
		(34_746_000 as Weight)
			// Standard Error: 0
			.saturating_add((127_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: PhragmenElection Candidates (r:1 w:1)
	/// The range of component `c` is `[1, 200]`.
	fn renounce_candidacy_candidate(c: u32, ) -> Weight {
		(30_131_000 as Weight)
			// Standard Error: 0
			.saturating_add((67_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: PhragmenElection Members (r:1 w:1)
	// Storage: PhragmenElection RunnersUp (r:1 w:1)
	// Storage: Council Prime (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	fn renounce_candidacy_members() -> Weight {
		(43_814_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: PhragmenElection RunnersUp (r:1 w:1)
	fn renounce_candidacy_runners_up() -> Weight {
		(32_228_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Benchmark Override (r:0 w:0)
	fn remove_member_without_replacement() -> Weight {
		(2_000_000_000_000 as Weight)
	}
	// Storage: PhragmenElection RunnersUp (r:1 w:1)
	// Storage: PhragmenElection Members (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Council Prime (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	fn remove_member_with_replacement() -> Weight {
		(59_701_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: PhragmenElection RunnersUp (r:1 w:0)
	fn remove_member_wrong_refund() -> Weight {
		(9_232_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	// Storage: PhragmenElection Voting (r:251 w:250)
	// Storage: PhragmenElection Members (r:1 w:0)
	// Storage: PhragmenElection RunnersUp (r:1 w:0)
	// Storage: PhragmenElection Candidates (r:1 w:0)
	// Storage: Balances Locks (r:250 w:250)
	// Storage: System Account (r:250 w:250)
	/// The range of component `v` is `[250, 500]`.
	/// The range of component `d` is `[1, 250]`.
	fn clean_defunct_voters(v: u32, _d: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 56_000
			.saturating_add((50_089_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(v as Weight)))
			.saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(v as Weight)))
	}
	// Storage: PhragmenElection Candidates (r:1 w:1)
	// Storage: PhragmenElection Members (r:1 w:1)
	// Storage: PhragmenElection RunnersUp (r:1 w:1)
	// Storage: PhragmenElection Voting (r:502 w:0)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: PhragmenElection ElectionRounds (r:1 w:1)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `c` is `[1, 200]`.
	/// The range of component `v` is `[1, 500]`.
	/// The range of component `e` is `[500, 8000]`.
	fn election_phragmen(c: u32, v: u32, e: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 3_227_000
			.saturating_add((121_351_000 as Weight).saturating_mul(c as Weight))
			// Standard Error: 1_342_000
			.saturating_add((107_477_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 91_000
			.saturating_add((7_521_000 as Weight).saturating_mul(e as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(v as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
}