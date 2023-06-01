pub use acceptance_policy::AcceptancePolicy;
pub use block_finalizer::MockedBlockFinalizer;
pub use block_request::MockedBlockRequester;
pub use client::{TestClient, TestClientBuilder, TestClientBuilderExt};
pub use proposal::{
	selendra_data_from_blocks, selendra_data_from_headers, unvalidated_proposal_from_headers,
};
use selendra_primitives::BlockNumber;
use sp_runtime::traits::BlakeTwo256;
use substrate_test_runtime::Extrinsic;

use crate::IdentifierFor;

type Hashing = BlakeTwo256;
pub type TBlock = sp_runtime::generic::Block<THeader, Extrinsic>;
pub type THeader = sp_runtime::generic::Header<BlockNumber, Hashing>;
pub type THash = substrate_test_runtime::Hash;
pub type TBlockIdentifier = IdentifierFor<TBlock>;

mod acceptance_policy;
mod block_finalizer;
mod block_request;
mod client;
mod proposal;
mod single_action_mock;
