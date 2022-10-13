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
//! The scheduler module for indracores and indrabases.
//!
//! This module is responsible for two main tasks:
//!   - Partitioning validators into groups and assigning groups to indracores and indrabases
//!   - Scheduling indracores and indrabases
//!
//! It aims to achieve these tasks with these goals in mind:
//! - It should be possible to know at least a block ahead-of-time, ideally more,
//!   which validators are going to be assigned to which indracores.
//! - Indracores that have a candidate pending availability in this fork of the chain
//!   should not be assigned.
//! - Validator assignments should not be gameable. Malicious cartels should not be able to
//!   manipulate the scheduler to assign themselves as desired.
//! - High or close to optimal throughput of indracores and indrabases. Work among validator groups should be balanced.
//!
//! The Scheduler manages resource allocation using the concept of "Availability Cores".
//! There will be one availability core for each indracore, and a fixed number of cores
//! used for multiplexing indrabases. Validators will be partitioned into groups, with the same
//! number of groups as availability cores. Validator groups will be assigned to different availability cores
//! over time.

use frame_support::pallet_prelude::*;
use primitives::v2::{
	CollatorId, CoreIndex, CoreOccupied, GroupIndex, GroupRotationInfo, Id as IndraId,
	IndrabaseClaim, IndrabaseEntry, ScheduledCore, ValidatorIndex,
};
use scale_info::TypeInfo;
use sp_runtime::traits::{One, Saturating};
use sp_std::prelude::*;

use crate::{configuration, indras, initializer::SessionChangeNotification};

pub use pallet::*;

#[cfg(test)]
mod tests;

/// A queued indrabase entry, pre-assigned to a core.
#[derive(Encode, Decode, TypeInfo)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct QueuedIndrabase {
	claim: IndrabaseEntry,
	core_offset: u32,
}

/// The queue of all indrabase claims.
#[derive(Encode, Decode, TypeInfo)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct IndrabaseClaimQueue {
	queue: Vec<QueuedIndrabase>,
	// this value is between 0 and config.indrabase_cores
	next_core_offset: u32,
}

impl IndrabaseClaimQueue {
	/// Queue a indrabase entry to be processed.
	///
	/// Provide the entry and the number of indrabase cores, which must be greater than 0.
	fn enqueue_entry(&mut self, entry: IndrabaseEntry, n_indrabase_cores: u32) {
		let core_offset = self.next_core_offset;
		self.next_core_offset = (self.next_core_offset + 1) % n_indrabase_cores;

		self.queue.push(QueuedIndrabase { claim: entry, core_offset })
	}

	/// Take next queued entry with given core offset, if any.
	fn take_next_on_core(&mut self, core_offset: u32) -> Option<IndrabaseEntry> {
		let pos = self.queue.iter().position(|queued| queued.core_offset == core_offset);
		pos.map(|i| self.queue.remove(i).claim)
	}

	/// Get the next queued entry with given core offset, if any.
	fn get_next_on_core(&self, core_offset: u32) -> Option<&IndrabaseEntry> {
		let pos = self.queue.iter().position(|queued| queued.core_offset == core_offset);
		pos.map(|i| &self.queue[i].claim)
	}
}

impl Default for IndrabaseClaimQueue {
	fn default() -> Self {
		Self { queue: vec![], next_core_offset: 0 }
	}
}

/// Reasons a core might be freed
#[derive(Clone, Copy)]
pub enum FreedReason {
	/// The core's work concluded and the indrablock assigned to it is considered available.
	Concluded,
	/// The core's work timed out.
	TimedOut,
}

/// The assignment type.
#[derive(Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(PartialEq, Debug))]
pub enum AssignmentKind {
	/// A indracore.
	Indracore,
	/// A indrabase.
	Indrabase(CollatorId, u32),
}

/// How a free core is scheduled to be assigned.
#[derive(Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(PartialEq, Debug))]
pub struct CoreAssignment {
	/// The core that is assigned.
	pub core: CoreIndex,
	/// The unique ID of the indra that is assigned to the core.
	pub indra_id: IndraId,
	/// The kind of the assignment.
	pub kind: AssignmentKind,
	/// The index of the validator group assigned to the core.
	pub group_idx: GroupIndex,
}

impl CoreAssignment {
	/// Get the ID of a collator who is required to collate this block.
	pub fn required_collator(&self) -> Option<&CollatorId> {
		match self.kind {
			AssignmentKind::Indracore => None,
			AssignmentKind::Indrabase(ref id, _) => Some(id),
		}
	}

	/// Get the `CoreOccupied` from this.
	pub fn to_core_occupied(&self) -> CoreOccupied {
		match self.kind {
			AssignmentKind::Indracore => CoreOccupied::Indracore,
			AssignmentKind::Indrabase(ref collator, retries) =>
				CoreOccupied::Indrabase(IndrabaseEntry {
					claim: IndrabaseClaim(self.indra_id, collator.clone()),
					retries,
				}),
		}
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + configuration::Config + indras::Config {}

	/// All the validator groups. One for each core. Indices are into `ActiveValidators` - not the
	/// broader set of Selendra validators, but instead just the subset used for indracores during
	/// this session.
	///
	/// Bound: The number of cores is the sum of the numbers of indracores and indrabase multiplexers.
	/// Reasonably, 100-1000. The dominant factor is the number of validators: safe upper bound at 10k.
	#[pallet::storage]
	#[pallet::getter(fn validator_groups)]
	pub(crate) type ValidatorGroups<T> = StorageValue<_, Vec<Vec<ValidatorIndex>>, ValueQuery>;

	/// A queue of upcoming claims and which core they should be mapped onto.
	///
	/// The number of queued claims is bounded at the `scheduling_lookahead`
	/// multiplied by the number of indrabase multiplexer cores. Reasonably, 10 * 50 = 500.
	#[pallet::storage]
	pub(crate) type IndrabaseQueue<T> = StorageValue<_, IndrabaseClaimQueue, ValueQuery>;

	/// One entry for each availability core. Entries are `None` if the core is not currently occupied. Can be
	/// temporarily `Some` if scheduled but not occupied.
	/// The i'th indracore belongs to the i'th core, with the remaining cores all being
	/// indrabase-multiplexers.
	///
	/// Bounded by the maximum of either of these two values:
	///   * The number of indracores and indrabase multiplexers
	///   * The number of validators divided by `configuration.max_validators_per_core`.
	#[pallet::storage]
	#[pallet::getter(fn availability_cores)]
	pub(crate) type AvailabilityCores<T> = StorageValue<_, Vec<Option<CoreOccupied>>, ValueQuery>;

	/// An index used to ensure that only one claim on a indrabase exists in the queue or is
	/// currently being handled by an occupied core.
	///
	/// Bounded by the number of indrabase cores and scheduling lookahead. Reasonably, 10 * 50 = 500.
	#[pallet::storage]
	pub(crate) type IndrabaseClaimIndex<T> = StorageValue<_, Vec<IndraId>, ValueQuery>;

	/// The block number where the session start occurred. Used to track how many group rotations have occurred.
	///
	/// Note that in the context of indracores modules the session change is signaled during
	/// the block and enacted at the end of the block (at the finalization stage, to be exact).
	/// Thus for all intents and purposes the effect of the session change is observed at the
	/// block following the session change, block number of which we save in this storage value.
	#[pallet::storage]
	#[pallet::getter(fn session_start_block)]
	pub(crate) type SessionStartBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// Currently scheduled cores - free but up to be occupied.
	///
	/// Bounded by the number of cores: one for each indracore and indrabase multiplexer.
	///
	/// The value contained here will not be valid after the end of a block. Runtime APIs should be used to determine scheduled cores/
	/// for the upcoming block.
	#[pallet::storage]
	#[pallet::getter(fn scheduled)]
	pub(crate) type Scheduled<T> = StorageValue<_, Vec<CoreAssignment>, ValueQuery>;
	// sorted ascending by CoreIndex.
}

impl<T: Config> Pallet<T> {
	/// Called by the initializer to initialize the scheduler pallet.
	pub(crate) fn initializer_initialize(_now: T::BlockNumber) -> Weight {
		0
	}

	/// Called by the initializer to finalize the scheduler pallet.
	pub(crate) fn initializer_finalize() {}

	/// Called by the initializer to note that a new session has started.
	pub(crate) fn initializer_on_new_session(
		notification: &SessionChangeNotification<T::BlockNumber>,
	) {
		let &SessionChangeNotification { ref validators, ref new_config, .. } = notification;
		let config = new_config;

		let mut thread_queue = IndrabaseQueue::<T>::get();
		let n_indracores = <indras::Pallet<T>>::indracores().len() as u32;
		let n_cores = core::cmp::max(
			n_indracores + config.indrabase_cores,
			match config.max_validators_per_core {
				Some(x) if x != 0 => validators.len() as u32 / x,
				_ => 0,
			},
		);

		AvailabilityCores::<T>::mutate(|cores| {
			// clear all occupied cores.
			for maybe_occupied in cores.iter_mut() {
				if let Some(CoreOccupied::Indrabase(claim)) = maybe_occupied.take() {
					let queued = QueuedIndrabase {
						claim,
						core_offset: 0, // this gets set later in the re-balancing.
					};

					thread_queue.queue.push(queued);
				}
			}

			cores.resize(n_cores as _, None);
		});

		// shuffle validators into groups.
		if n_cores == 0 || validators.is_empty() {
			ValidatorGroups::<T>::set(Vec::new());
		} else {
			let group_base_size = validators.len() / n_cores as usize;
			let n_larger_groups = validators.len() % n_cores as usize;

			// Groups contain indices into the validators from the session change notification,
			// which are already shuffled.

			let mut groups: Vec<Vec<ValidatorIndex>> = Vec::new();
			for i in 0..n_larger_groups {
				let offset = (group_base_size + 1) * i;
				groups.push(
					(0..group_base_size + 1)
						.map(|j| offset + j)
						.map(|j| ValidatorIndex(j as _))
						.collect(),
				);
			}

			for i in 0..(n_cores as usize - n_larger_groups) {
				let offset = (n_larger_groups * (group_base_size + 1)) + (i * group_base_size);
				groups.push(
					(0..group_base_size)
						.map(|j| offset + j)
						.map(|j| ValidatorIndex(j as _))
						.collect(),
				);
			}

			ValidatorGroups::<T>::set(groups);
		}

		// prune out all indrabase claims with too many retries.
		// assign all non-pruned claims to new cores, if they've changed.
		IndrabaseClaimIndex::<T>::mutate(|claim_index| {
			// wipe all indrabase metadata if no indrabase cores are configured.
			if config.indrabase_cores == 0 {
				thread_queue = IndrabaseClaimQueue { queue: Vec::new(), next_core_offset: 0 };
				claim_index.clear();
				return
			}

			// prune out all entries beyond retry or that no longer correspond to live indrabase.
			thread_queue.queue.retain(|queued| {
				let will_keep = queued.claim.retries <= config.indrabase_retries &&
					<indras::Pallet<T>>::is_indrabase(queued.claim.claim.0);

				if !will_keep {
					let claim_indra = queued.claim.claim.0;

					// clean up the pruned entry from the index.
					if let Ok(i) = claim_index.binary_search(&claim_indra) {
						claim_index.remove(i);
					}
				}

				will_keep
			});

			// do re-balancing of claims.
			{
				for (i, queued) in thread_queue.queue.iter_mut().enumerate() {
					queued.core_offset = (i as u32) % config.indrabase_cores;
				}

				thread_queue.next_core_offset =
					((thread_queue.queue.len()) as u32) % config.indrabase_cores;
			}
		});
		IndrabaseQueue::<T>::set(thread_queue);

		let now = <frame_system::Pallet<T>>::block_number() + One::one();
		<SessionStartBlock<T>>::set(now);
	}

	/// Add a indrabase claim to the queue. If there is a competing claim in the queue or currently
	/// assigned to a core, this call will fail. This call will also fail if the queue is full.
	///
	/// Fails if the claim does not correspond to any live indrabase.
	#[allow(unused)]
	pub fn add_indrabase_claim(claim: IndrabaseClaim) {
		if !<indras::Pallet<T>>::is_indrabase(claim.0) {
			return
		}

		let config = <configuration::Pallet<T>>::config();
		let queue_max_size = config.indrabase_cores * config.scheduling_lookahead;

		IndrabaseQueue::<T>::mutate(|queue| {
			if queue.queue.len() >= queue_max_size as usize {
				return
			}

			let indra_id = claim.0;

			let competes_with_another =
				IndrabaseClaimIndex::<T>::mutate(|index| match index.binary_search(&indra_id) {
					Ok(_) => true,
					Err(i) => {
						index.insert(i, indra_id);
						false
					},
				});

			if competes_with_another {
				return
			}

			let entry = IndrabaseEntry { claim, retries: 0 };
			queue.enqueue_entry(entry, config.indrabase_cores);
		})
	}

	/// Free unassigned cores. Provide a list of cores that should be considered newly-freed along with the reason
	/// for them being freed. The list is assumed to be sorted in ascending order by core index.
	pub(crate) fn free_cores(just_freed_cores: impl IntoIterator<Item = (CoreIndex, FreedReason)>) {
		let config = <configuration::Pallet<T>>::config();

		AvailabilityCores::<T>::mutate(|cores| {
			for (freed_index, freed_reason) in just_freed_cores {
				if (freed_index.0 as usize) < cores.len() {
					match cores[freed_index.0 as usize].take() {
						None => continue,
						Some(CoreOccupied::Indracore) => {},
						Some(CoreOccupied::Indrabase(entry)) => {
							match freed_reason {
								FreedReason::Concluded => {
									// After a indrabase candidate has successfully been included,
									// open it up for further claims!
									IndrabaseClaimIndex::<T>::mutate(|index| {
										if let Ok(i) = index.binary_search(&entry.claim.0) {
											index.remove(i);
										}
									})
								},
								FreedReason::TimedOut => {
									// If a indrabase candidate times out, it's not the collator's fault,
									// so we don't increment retries.
									IndrabaseQueue::<T>::mutate(|queue| {
										queue.enqueue_entry(entry, config.indrabase_cores);
									})
								},
							}
						},
					}
				}
			}
		})
	}

	/// Schedule all unassigned cores, where possible. Provide a list of cores that should be considered
	/// newly-freed along with the reason for them being freed. The list is assumed to be sorted in
	/// ascending order by core index.
	pub(crate) fn schedule(
		just_freed_cores: impl IntoIterator<Item = (CoreIndex, FreedReason)>,
		now: T::BlockNumber,
	) {
		Self::free_cores(just_freed_cores);

		let cores = AvailabilityCores::<T>::get();
		let indracores = <indras::Pallet<T>>::indracores();
		let mut scheduled = Scheduled::<T>::get();
		let mut indrabase_queue = IndrabaseQueue::<T>::get();

		if ValidatorGroups::<T>::get().is_empty() {
			return
		}

		{
			let mut prev_scheduled_in_order = scheduled.iter().enumerate().peekable();

			// Updates to the previous list of scheduled updates and the position of where to insert
			// them, without accounting for prior updates.
			let mut scheduled_updates: Vec<(usize, CoreAssignment)> = Vec::new();

			// single-sweep O(n) in the number of cores.
			for (core_index, _core) in cores.iter().enumerate().filter(|(_, ref c)| c.is_none()) {
				let schedule_and_insert_at = {
					// advance the iterator until just before the core index we are looking at now.
					while prev_scheduled_in_order
						.peek()
						.map_or(false, |(_, assign)| (assign.core.0 as usize) < core_index)
					{
						let _ = prev_scheduled_in_order.next();
					}

					// check the first entry already scheduled with core index >= than the one we
					// are looking at. 3 cases:
					//  1. No such entry, clearly this core is not scheduled, so we need to schedule and put at the end.
					//  2. Entry exists and has same index as the core we are inspecting. do not schedule again.
					//  3. Entry exists and has higher index than the core we are inspecting. schedule and note
					//     insertion position.
					prev_scheduled_in_order.peek().map_or(
						Some(scheduled.len()),
						|(idx_in_scheduled, assign)| {
							if (assign.core.0 as usize) == core_index {
								None
							} else {
								Some(*idx_in_scheduled)
							}
						},
					)
				};

				let schedule_and_insert_at = match schedule_and_insert_at {
					None => continue,
					Some(at) => at,
				};

				let core = CoreIndex(core_index as u32);

				let core_assignment = if core_index < indracores.len() {
					// indracore core.
					Some(CoreAssignment {
						kind: AssignmentKind::Indracore,
						indra_id: indracores[core_index],
						core: core.clone(),
						group_idx: Self::group_assigned_to_core(core, now).expect(
							"core is not out of bounds and we are guaranteed \
									to be after the most recent session start; qed",
						),
					})
				} else {
					// indrabase core offset, rel. to beginning.
					let core_offset = (core_index - indracores.len()) as u32;

					indrabase_queue.take_next_on_core(core_offset).map(|entry| CoreAssignment {
						kind: AssignmentKind::Indrabase(entry.claim.1, entry.retries),
						indra_id: entry.claim.0,
						core: core.clone(),
						group_idx: Self::group_assigned_to_core(core, now).expect(
							"core is not out of bounds and we are guaranteed \
									to be after the most recent session start; qed",
						),
					})
				};

				if let Some(assignment) = core_assignment {
					scheduled_updates.push((schedule_and_insert_at, assignment))
				}
			}

			// at this point, because `Scheduled` is guaranteed to be sorted and we navigated unassigned
			// core indices in ascending order, we can enact the updates prepared by the previous actions.
			//
			// while inserting, we have to account for the amount of insertions already done.
			//
			// This is O(n) as well, capped at n operations, where n is the number of cores.
			for (num_insertions_before, (insert_at, to_insert)) in
				scheduled_updates.into_iter().enumerate()
			{
				let insert_at = num_insertions_before + insert_at;
				scheduled.insert(insert_at, to_insert);
			}

			// scheduled is guaranteed to be sorted after this point because it was sorted before, and we
			// applied sorted updates at their correct positions, accounting for the offsets of previous
			// insertions.
		}

		Scheduled::<T>::set(scheduled);
		IndrabaseQueue::<T>::set(indrabase_queue);
	}

	/// Note that the given cores have become occupied. Behavior undefined if any of the given cores were not scheduled
	/// or the slice is not sorted ascending by core index.
	///
	/// Complexity: O(n) in the number of scheduled cores, which is capped at the number of total cores.
	/// This is efficient in the case that most scheduled cores are occupied.
	pub(crate) fn occupied(now_occupied: &[CoreIndex]) {
		if now_occupied.is_empty() {
			return
		}

		let mut availability_cores = AvailabilityCores::<T>::get();
		Scheduled::<T>::mutate(|scheduled| {
			// The constraints on the function require that `now_occupied` is a sorted subset of the
			// `scheduled` cores, which are also sorted.

			let mut occupied_iter = now_occupied.iter().cloned().peekable();
			scheduled.retain(|assignment| {
				let retain = occupied_iter
					.peek()
					.map_or(true, |occupied_idx| occupied_idx != &assignment.core);

				if !retain {
					// remove this entry - it's now occupied. and begin inspecting the next extry
					// of the occupied iterator.
					let _ = occupied_iter.next();

					availability_cores[assignment.core.0 as usize] =
						Some(assignment.to_core_occupied());
				}

				retain
			})
		});

		AvailabilityCores::<T>::set(availability_cores);
	}

	/// Get the indra (chain or thread) ID assigned to a particular core or index, if any. Core indices
	/// out of bounds will return `None`, as will indices of unassigned cores.
	pub(crate) fn core_indra(core_index: CoreIndex) -> Option<IndraId> {
		let cores = AvailabilityCores::<T>::get();
		match cores.get(core_index.0 as usize).and_then(|c| c.as_ref()) {
			None => None,
			Some(CoreOccupied::Indracore) => {
				let indracores = <indras::Pallet<T>>::indracores();
				Some(indracores[core_index.0 as usize])
			},
			Some(CoreOccupied::Indrabase(ref entry)) => Some(entry.claim.0),
		}
	}

	/// Get the validators in the given group, if the group index is valid for this session.
	pub(crate) fn group_validators(group_index: GroupIndex) -> Option<Vec<ValidatorIndex>> {
		ValidatorGroups::<T>::get().get(group_index.0 as usize).map(|g| g.clone())
	}

	/// Get the group assigned to a specific core by index at the current block number. Result undefined if the core index is unknown
	/// or the block number is less than the session start index.
	pub(crate) fn group_assigned_to_core(
		core: CoreIndex,
		at: T::BlockNumber,
	) -> Option<GroupIndex> {
		let config = <configuration::Pallet<T>>::config();
		let session_start_block = <SessionStartBlock<T>>::get();

		if at < session_start_block {
			return None
		}

		let validator_groups = ValidatorGroups::<T>::get();

		if core.0 as usize >= validator_groups.len() {
			return None
		}

		let rotations_since_session_start: T::BlockNumber =
			(at - session_start_block) / config.group_rotation_frequency.into();

		let rotations_since_session_start =
			match <T::BlockNumber as TryInto<u32>>::try_into(rotations_since_session_start) {
				Ok(i) => i,
				Err(_) => 0, // can only happen if rotations occur only once every u32::max(),
				             // so functionally no difference in behavior.
			};

		let group_idx =
			(core.0 as usize + rotations_since_session_start as usize) % validator_groups.len();
		Some(GroupIndex(group_idx as u32))
	}

	/// Returns an optional predicate that should be used for timing out occupied cores.
	///
	/// If `None`, no timing-out should be done. The predicate accepts the index of the core, and the
	/// block number since which it has been occupied, and the respective indracore and indrabase
	/// timeouts, i.e. only within `max(config.chain_availability_period, config.thread_availability_period)`
	/// of the last rotation would this return `Some`, unless there are no rotations.
	///
	/// This really should not be a box, but is working around a compiler limitation filed here:
	/// https://github.com/rust-lang/rust/issues/73226
	/// which prevents us from testing the code if using `impl Trait`.
	pub(crate) fn availability_timeout_predicate(
	) -> Option<Box<dyn Fn(CoreIndex, T::BlockNumber) -> bool>> {
		let now = <frame_system::Pallet<T>>::block_number();
		let config = <configuration::Pallet<T>>::config();

		let session_start = <SessionStartBlock<T>>::get();
		let blocks_since_session_start = now.saturating_sub(session_start);
		let blocks_since_last_rotation =
			blocks_since_session_start % config.group_rotation_frequency;

		let absolute_cutoff =
			sp_std::cmp::max(config.chain_availability_period, config.thread_availability_period);

		let availability_cores = AvailabilityCores::<T>::get();

		if blocks_since_last_rotation >= absolute_cutoff {
			None
		} else {
			Some(Box::new(move |core_index: CoreIndex, pending_since| {
				match availability_cores.get(core_index.0 as usize) {
					None => true,       // out-of-bounds, doesn't really matter what is returned.
					Some(None) => true, // core not occupied, still doesn't really matter.
					Some(Some(CoreOccupied::Indracore)) => {
						if blocks_since_last_rotation >= config.chain_availability_period {
							false // no pruning except recently after rotation.
						} else {
							now.saturating_sub(pending_since) >= config.chain_availability_period
						}
					},
					Some(Some(CoreOccupied::Indrabase(_))) => {
						if blocks_since_last_rotation >= config.thread_availability_period {
							false // no pruning except recently after rotation.
						} else {
							now.saturating_sub(pending_since) >= config.thread_availability_period
						}
					},
				}
			}))
		}
	}

	/// Returns a helper for determining group rotation.
	pub(crate) fn group_rotation_info(now: T::BlockNumber) -> GroupRotationInfo<T::BlockNumber> {
		let session_start_block = Self::session_start_block();
		let group_rotation_frequency =
			<configuration::Pallet<T>>::config().group_rotation_frequency;

		GroupRotationInfo { session_start_block, now, group_rotation_frequency }
	}

	/// Return the next thing that will be scheduled on this core assuming it is currently
	/// occupied and the candidate occupying it became available.
	///
	/// For indracores, this is always the ID of the indracore and no specified collator.
	/// For indrabases, this is based on the next item in the `IndrabaseQueue` assigned to that
	/// core, and is None if there isn't one.
	pub(crate) fn next_up_on_available(core: CoreIndex) -> Option<ScheduledCore> {
		let indracores = <indras::Pallet<T>>::indracores();
		if (core.0 as usize) < indracores.len() {
			Some(ScheduledCore { indra_id: indracores[core.0 as usize], collator: None })
		} else {
			let queue = IndrabaseQueue::<T>::get();
			let core_offset = (core.0 as usize - indracores.len()) as u32;
			queue.get_next_on_core(core_offset).map(|entry| ScheduledCore {
				indra_id: entry.claim.0,
				collator: Some(entry.claim.1.clone()),
			})
		}
	}

	/// Return the next thing that will be scheduled on this core assuming it is currently
	/// occupied and the candidate occupying it became available.
	///
	/// For indracores, this is always the ID of the indracore and no specified collator.
	/// For indrabases, this is based on the next item in the `IndrabaseQueue` assigned to that
	/// core, or if there isn't one, the claim that is currently occupying the core, as long
	/// as the claim's retries would not exceed the limit. Otherwise None.
	pub(crate) fn next_up_on_time_out(core: CoreIndex) -> Option<ScheduledCore> {
		let indracores = <indras::Pallet<T>>::indracores();
		if (core.0 as usize) < indracores.len() {
			Some(ScheduledCore { indra_id: indracores[core.0 as usize], collator: None })
		} else {
			let queue = IndrabaseQueue::<T>::get();

			// This is the next scheduled indra on this core.
			let core_offset = (core.0 as usize - indracores.len()) as u32;
			queue
				.get_next_on_core(core_offset)
				.map(|entry| ScheduledCore {
					indra_id: entry.claim.0,
					collator: Some(entry.claim.1.clone()),
				})
				.or_else(|| {
					// Or, if none, the claim currently occupying the core,
					// as it would be put back on the queue after timing out.
					let cores = AvailabilityCores::<T>::get();
					cores.get(core.0 as usize).and_then(|c| c.as_ref()).and_then(|o| {
						match o {
							CoreOccupied::Indrabase(entry) => Some(ScheduledCore {
								indra_id: entry.claim.0,
								collator: Some(entry.claim.1.clone()),
							}),
							CoreOccupied::Indracore => None, // defensive; not possible.
						}
					})
				})
		}
	}

	// Free all scheduled cores and return indrabase claims to queue, with retries incremented.
	pub(crate) fn clear() {
		let config = <configuration::Pallet<T>>::config();
		IndrabaseQueue::<T>::mutate(|queue| {
			for core_assignment in Scheduled::<T>::take() {
				if let AssignmentKind::Indrabase(collator, retries) = core_assignment.kind {
					if !<indras::Pallet<T>>::is_indrabase(core_assignment.indra_id) {
						continue
					}

					let entry = IndrabaseEntry {
						claim: IndrabaseClaim(core_assignment.indra_id, collator),
						retries: retries + 1,
					};

					if entry.retries <= config.indrabase_retries {
						queue.enqueue_entry(entry, config.indrabase_cores);
					}
				}
			}
		});
	}
}
