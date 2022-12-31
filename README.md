# CheckpointQ

CheckpointQ, stands Checkpoint Quorum. It is a tool for establishing quorum over finalized checkpoints across multiple 
ethereum checkpoint providers.

It makes request to multiple checkpoint providers and only returns the finalized checkpoint block root if 
more than 2/3 of the configured providers return the same checkpoint block root.      

## Installation

For now CheckpointQ requires to be built from source using the Rust toolchain.

- Install Rust. See [here](https://www.rust-lang.org/tools/install) for details.
- Clone the repository.
- Run `cargo build --release` to build the binary.
- Run `./target/release/checkpointq` to run the binary.

## Usage

Run the binary with the `--help` flag to see the available options:

```bash
➜  checkpointq git:(master) ✗ ./target/release/checkpointq --help
Tool for establishing checkpoint quorum for finalized checkpoints across multiple checkpoint providers

Usage: checkpointq [OPTIONS]

Options:
      --network <NETWORK>      [possible values: mainnet, goerli, sepolia]
  -e, --endpoints <ENDPOINTS>  path to config file where endpoints for network are listed. default is ./endpoint.yaml
  -i, --state-id <STATE_ID>    provide the slot number or finalized [default: finalized]
  -v, --verbose                display verbose result or not
  -s, --server                 start an HTTP server for the checkpoint data
  -p, --port <PORT>            port for HTTP server [default: 7070]
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
Block root:
        0xf5369df9f9b1a162023593e8d1c2b138fee2e21f4eed9921802d0a138fe5878c
```

The tool can be run in two modes: `server` and `client`. The `server` mode can be started by passing the `--server` flag
which starts an HTTP server that serves the finalized checkpoint.

For example:

```bash
➜  checkpointq git:(master) ✗ ./target/release/checkpointq --network sepolia --endpoints ./endpoints.yaml --server
Block root:
        0xf5369df9f9b1a162023593e8d1c2b138fee2e21f4eed9921802d0a138fe5878c
Starting server on port 7070
```

Requests to the server can be made using the `/finalized` endpoint. The server can be queried using `curl`:

```bash
➜  checkpointq git:(master) ✗ curl http://localhost:7070/finalized | jq
{
  "block_root": "0xf5369df9f9b1a162023593e8d1c2b138fee2e21f4eed9921802d0a138fe5878c"
}

```