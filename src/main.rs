use alloy_primitives::{address, Address};
use alloy_sol_macro::sol;
use alloy_sol_types::SolEventInterface;
use eigenda_proto::disperser::{disperser_client::DisperserClient, RetrieveBlobRequest};
use futures::{Future, TryStreamExt};
use kzgpad_rs::remove_empty_byte_from_padded_bytes;
use reth::{
    api::FullNodeComponents,
    primitives::{SealedBlockWithSenders, TransactionSigned},
};
use reth_execution_types::Chain;
use reth_exex::{ExExContext, ExExEvent};
use reth_node_ethereum::EthereumNode;
use reth_primitives::Log;
use reth_tracing::tracing::info;
use IEigenDAServiceManager::IEigenDAServiceManagerEvents;

//const TESTNET_EIGENDA_ADDRESS: Address = address!("D4A7E1Bd8015057293f0D0A557088c286942e84b");
const TESTNET_EIGENDA_ADDRESS: Address = address!("5fbdb2315678afecb367f032d93f642f64180aa3");
const TESTNET_DISPERSER_URL: &str = "https://disperser-holesky.eigenda.xyz:443";

sol!(
    contract IEigenDAServiceManager {
        #[derive(Debug)]
        event BatchConfirmed(bytes32 indexed batch_header_hash, uint32 batch_id);
    }
);

async fn exex_init<Node: FullNodeComponents>(
    ctx: ExExContext<Node>,
) -> eyre::Result<impl Future<Output = eyre::Result<()>>> {
    Ok(eigenda_exex(ctx))
}

async fn eigenda_exex<Node: FullNodeComponents>(mut ctx: ExExContext<Node>) -> eyre::Result<()> {
    let mut disperser_client = DisperserClient::connect(TESTNET_DISPERSER_URL).await?;
    println!("connected");
    while let Some(notification) = ctx.notifications.try_next().await? {
        if let Some(committed_chain) = notification.committed_chain() {
            let events = decode_chain_into_events(&committed_chain);
            for (_, _, _, event) in events {
                info!("Received event");
                if let IEigenDAServiceManagerEvents::BatchConfirmed(batch_confirmed) = event {
                    let mut blob_index = 1_u32;
                    loop {
                        let request = tonic::Request::new(RetrieveBlobRequest {
                            batch_header_hash: batch_confirmed.batch_header_hash.to_vec(),
                            blob_index,
                        });

                        let response = disperser_client.retrieve_blob(request).await;
                        if let Ok(r) = response {
                            let unpadded_data =
                                remove_empty_byte_from_padded_bytes(&r.into_inner().data);
                            info!(
                                "Got new blob with blob index {} and size {}",
                                blob_index,
                                unpadded_data.len()
                            );
                            blob_index += 1;
                        } else {
                            info!("End of batch");
                            break;
                        }
                    }
                }
            }

            ctx.events
                .send(ExExEvent::FinishedHeight(committed_chain.tip().num_hash()))?;
        }
    }

    Ok(())
}

fn decode_chain_into_events(
    chain: &Chain,
) -> impl Iterator<
    Item = (
        &SealedBlockWithSenders,
        &TransactionSigned,
        &Log,
        IEigenDAServiceManagerEvents,
    ),
> {
    chain
        .blocks_and_receipts()
        .flat_map(|(block, receipts)| {
            block
                .body
                .transactions()
                .zip(receipts.iter().flatten())
                .map(move |(tx, receipt)| (block, tx, receipt))
        })
        .flat_map(|(block, tx, receipt)| {
            receipt
                .logs
                .iter()
                .filter(|log| TESTNET_EIGENDA_ADDRESS.eq(&log.address))
                .map(move |log| (block, tx, log))
        })
        .filter_map(|(block, tx, log)| {
            IEigenDAServiceManagerEvents::decode_raw_log(log.topics(), &log.data.data, true)
                .ok()
                .map(|event| (block, tx, log, event))
        })
}

fn main() -> eyre::Result<()> {
    reth::cli::Cli::parse_args().run(|builder, _| async move {
        let handle = builder
            .node(EthereumNode::default())
            .install_exex("eigenda-exex", exex_init)
            .launch()
            .await?;

        handle.wait_for_node_exit().await
    })
}
