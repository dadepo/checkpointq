# CheckpointQ

CheckpointQ, where the Q stands for quorum, is a tool for establishing quorum over finalized checkpoints across multiple
Ethereum checkpoint providers.

It makes requests to multiple checkpoint providers and only returns the finalized checkpoint block root if more than 2/3 of the configured providers return the same checkpoint block root.

This ensures you do not have to trust a single checkpoint provider. The more providers agree on what the finalized checkpoint
is, the more assured you can be.

## Installation

At present, the installation process for CheckpointQ involves utilizing the Rust/Cargo toolchain. 
However, the release and distribution of the CheckpointQ binary are aspects that will be addressed in the future.

You can install CheckpointQ from cargo by running `cargo install checkpointq`. Or you can also build directly from 
the source yourself.

- Install Rust. See [here](https://www.rust-lang.org/tools/install) for details.
- Clone the repository.
- Run `cargo build --release` to build the binary.
- Run `./target/release/checkpointq` to run the binary.

## Usage

Run the binary with the `--help` flag to see the available options:

```bash
Tool for establishing checkpoint quorum for finalized checkpoints across multiple checkpoint providers

Usage: checkpointq [OPTIONS] [COMMAND]

Commands:
  server  Run in server mode
  help   Print this message or the help of the given subcommand(s)

Options:
  -e, --endpoints <ENDPOINTS>  Path to config file where endpoints for network are listed. default is ./endpoint.yaml
  -n, --network <NETWORK>      [possible values: mainnet, goerli, sepolia]
  -v, --verbose                Display verbose result or not
  -h, --help                   Print help information
  -V, --version                Print version information
```

CheckpointQ requires a config file. An example file can be found [here](./endpoints.yaml). The config file is a yaml file
that contains the endpoints of the checkpoint providers. The config file can be passed to the tool using the `-e` flag. An
example of the contents of the yaml file is shown below:

```yaml
endpoints:
  mainnet:
      - https://mainnet-checkpoint-sync.attestant.io
      - https://beaconstate.ethstaker.cc
      - https://beaconstate.info
      - https://mainnet-checkpoint-sync.stakely.io
      - https://checkpointz.pietjepuk.net
      - https://sync.invis.tools
      - https://sync-mainnet.beaconcha.in
      - https://mainnet.checkpoint.sigp.io
      - https://beaconstate-mainnet.chainsafe.io
  goerli:
      - https://sync-goerli.beaconcha.in
      - https://goerli-sync.invis.tools
      - https://goerli.beaconstate.ethstaker.cc
      - https://prater-checkpoint-sync.stakely.io
      - https://beaconstate-goerli.chainsafe.io
      - https://goerli.checkpoint-sync.ethdevops.io
      - https://goerli.beaconstate.info
      - https://prater.checkpoint.sigp.io
  sepolia:
      - https://beaconstate-sepolia.chainsafe.io
      - https://sepolia.beaconstate.info
      - https://sepolia.checkpoint-sync.ethdevops.io
```

for example:

```bash
➜  checkpointq git:(master) ✗ ./target/release/checkpointq --network sepolia --endpoints ./endpoints.yaml
Block root: 0x32c1b19ee499bfbd68b656eed0cf96278c4362942ad48b6cc7d15f620401351c
Epoch:  44614
```

The tool can be run in two modes: _Default_ mode that fetches the current finalized block root and print it to the console
and a `server` mode that runs a server and exposes `/:network/finalized` path, where finalized block root can be requested.

The `server` mode is available by running the  `server` command:


For example:

```bash
➜  checkpointq git:(master) ✗ ./target/release/checkpointq server --endpoints ./endpoints.yaml
```

Default port is `7070`. Requests to the server can be made using the `/:network/finalized` endpoint for example:

```bash
➜  checkpointq git:(master) ✗ curl http://localhost:7070/sepolia/finalized | jq
{
"block_root": "0x32c1b19ee499bfbd68b656eed0cf96278c4362942ad48b6cc7d15f620401351c",
"epoch": "44614"
}
```