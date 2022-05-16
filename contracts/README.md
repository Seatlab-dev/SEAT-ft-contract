# SEAT Fungible Token

Description: This contains the SEAT fungible token contract.

## User Stories

- [Integrated User Story 1](./doc/story-1.md): Init, user registration and minting.
- [Testnet 1](./doc/testnet-1.sh): Init, user registration and minting.

## Building

Please check the [build.sh](./build.sh) script file.

That build script assumes you have some software required for optmizing the wasm binary files. By checking the script or the github workflow file should make it easier to understand how to install it.

## Simulation Testing

Please check the [test.sh](./test.sh) script file.

The simulation tests also outputs the near CLI commands that they would make. To capture them, you can run the tests with:

```bash
cargo nextest run --fail-fast --no-capture 2>&1 | tee tests_outputs.txt
```

## Useful Links

- [Live Contracts](../README.md#live-contracts).
- Downloads available on the [Releases](https://github.com/Seatlab-dev/SEAT-ft-contract/releases) page.
