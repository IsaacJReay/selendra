# open-runtime-module-library

[![Crates.io](https://img.shields.io/crates/v/orml-tokens)](https://crates.io/search?q=orml)
[![codecov](https://codecov.io/gh/open-web3-stack/open-runtime-module-library/branch/master/graph/badge.svg?token=FZ4HZYMW9A)](https://codecov.io/gh/open-web3-stack/open-runtime-module-library)
[![GitHub](https://img.shields.io/github/license/open-web3-stack/open-runtime-module-library)](https://github.com/open-web3-stack/open-runtime-module-library/blob/master/LICENSE)

The Open Runtime Module Library (ORML) is a community maintained collection of Substrate runtime modules.

## Runtime Modules Overview

- [orml-currencies](./currencies)
	- Provide `MultiCurrency` implementation using `pallet-balances` and `orml-tokens` module.
- [orml-gradually-update](./gradually-update)
	- Provides way to adjust numeric parameter gradually over a period of time.
- [orml-nft](./nft)
 	- Non-fungible-token module provides basic functions to create and manager NFT(non fungible token) such as `create_class`, `transfer`, `mint`, `burn`, `destroy_class`.
- [orml-oracle](./oracle)
	- Oracle module that makes off-chain data available on-chain.
- [orml-tokens](./tokens)
	- Fungible tokens module that implements `MultiCurrency` trait.
- [orml-traits](./traits)
	- Shared traits including `BasicCurrency`, `MultiCurrency`, `Auction` and more.
- [orml-utilities](./utilities)
	- Various utilities including `OrderSet`.
- [orml-vesting](./vesting)
	- Provides scheduled balance locking mechanism, in a *graded vesting* way.
- [orml-xcm-support](./xcm-support)
	- Provides traits, types, and implementations to support XCM integration.
- [orml-xtokens](./xtokens)
	- Provides way to do cross-chain assets transfer.
	- [Step-by-Step guide](https://github.com/open-web3-stack/open-runtime-module-library/wiki/xtokens) to make XCM cross-chain fungible asset transfer available on your parachain

## Example

Checkout [orml-workshop](https://github.com/xlc/orml-workshop) for example usage.

### `Cargo.toml`

ORML use `Cargo.dev.toml` to avoid workspace conflicts with project cargo config. To use cargo commands in ORML workspace, create `Cargo.toml` by running

- `cp Cargo.dev.toml Cargo.toml`, or
- `make Cargo.toml`, or
- change the command to `make dev-check` etc which does the copy. (For the full list of `make` commands, check `Makefile`)

# Web3 Foundation Grant Project
ORML is part of the bigger `Open-Web3-Stack` initiative, that is currently under a General Grant from Web3 Foundation. See Application details [here](https://github.com/open-web3-stack/General-Grants-Program/blob/master/grants/speculative/open_web3_stack.md). The 1st milestone has been delivered.

# Projects using ORML
- [If you intend or are using ORML, please add your project here](https://github.com/open-web3-stack/open-runtime-module-library/edit/master/README.md)