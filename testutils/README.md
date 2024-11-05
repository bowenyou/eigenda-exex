# Testing Utilities

Provides tools to test the execution extension in production without syncing a node.

## Env variables

```bash
export DEPLOY_PRIVATEKEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
export RPC_URL=127.0.0.1:8545
```

The private key is a junk one corresponding to the address `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`.
The rpc url is the default one provided by `reth`.

## Testing

To test in a dev node, a few preliminary steps need to be taken.

1. Start a dev node:

```bash
cargo run --release -- node --dev --dev.block-time 12s --datadir data
```

This starts a dev node writing the data to a custom data directory called `data`.

2. Deploy the test contract. This is to mock the actual `EigenDAServiceManager` contract.

```bash
forge create src/TestContract.sol:TestContract --private-key $DEPLOY_PRIVATEKEY --rpc-url $RPC_URL
```

Make note of the deployed contract address.

3. Stop the node.

4. Change the `TESTNET_EIGENDA_ADDRESS` variable in `main.rs` with the deployed contract address.

5. Restart the node using the same command as step 1.

6. Now we can submit some mock events to simulate the actual `confirmBatch` function of the `EigenDAServiceManager` contract.
Make sure to replace the address with the correct deployed address.

```bash
cast send 0x5FbDB2315678afecb367f032d93F642f64180aa3 "echo(bytes32,uint32)" 0xc59a9b755235bec651f628f200711e0e974828db73c28c948537fae086d4464b 74629 --private-key $DEPLOY_PRIVATEKEY --rpc-url $RPC_URL
```

7. Now you should see some reth exex logs, similar to this

```bash
2024-11-05T04:38:52.059444Z  INFO exex{id="eigenda-exex"}: Got new blob with blob index 1 and size 484
2024-11-05T04:38:52.412457Z  INFO exex{id="eigenda-exex"}: Got new blob with blob index 2 and size 484
2024-11-05T04:38:53.599202Z  INFO exex{id="eigenda-exex"}: Got new blob with blob index 3 and size 126976
```

When you want to run on actual testnet, just uncomment this line:

```rust
const TESTNET_EIGENDA_ADDRESS: Address = address!("D4A7E1Bd8015057293f0D0A557088c286942e84b");
```
