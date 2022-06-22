#![allow(clippy::too_many_arguments)]
#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]

use polkadot::RuntimeApi;
use primitive_types::H256;
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::{
    sp_runtime::{generic::Header, traits::BlakeTwo256, OpaqueExtrinsic},
    BasicError, ClientBuilder, DefaultConfig, PolkadotExtrinsicParams,
};
use tokio::signal;

pub mod plotter;
pub mod rpc_ext;

use clap::Parser;
use plotter::Extrinsic as SimpleExtrinsic;
use plotter::{plot, Block as SimpleBlock};

#[subxt::subxt(runtime_metadata_path = "ice_metadata.scale")]
pub mod polkadot {}

/// Simple program plot extrinsics Fee cost
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// URL of the node WS
    #[clap(short, long, value_parser, default_value = "ws://localhost:9944")]
    ws_url: String,

    /// Location where to save the output graph as PNG
    #[clap(short, long, value_parser, default_value = "plotters-doc-data/0.png")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let api = ClientBuilder::new()
        .set_url(args.ws_url)
        .build()
        .await
        .expect("Could not connect to the node")
        .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();

    let mut blocks_sub = api.client.rpc().subscribe_blocks().await.unwrap();

    let mut blocks = Vec::new();

    println!("Press Ctrl+C to stop waiting for blocks");

    loop {
        tokio::select! {
            _ =  signal::ctrl_c() => {
                println!("Ctrl+C was received. Will break");
                break;
            }
            block_header = blocks_sub.next() => {
                if let Some(Ok(block_header)) = block_header {
                    match process_block_header(&api, block_header).await {
                        Ok(block) => blocks.push(block),
                        Err(msg) => println!("{:?}", msg),
                    }
                }
            }
        }
    }

    plot(blocks, args.output)?;

    Ok(())
}

async fn process_block_header(
    api: &RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>,
    block_header: Header<u32, BlakeTwo256>,
) -> Result<SimpleBlock, BasicError> {
    println!("Processing block: {}\n->", block_header.hash());

    let signed_block = api
        .client
        .rpc()
        .block(Some(block_header.hash()))
        .await?
        .unwrap();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let block_hash = signed_block.block.header.hash();
    let mut extrinsics = Vec::with_capacity(signed_block.block.extrinsics.len());

    for extrinsic in signed_block.block.extrinsics.iter() {
        let simple_extrinsic = process_extrinsics_in_block(api, extrinsic, &block_hash).await;

        match simple_extrinsic {
            Ok(simple_extrinsic) => extrinsics.push(simple_extrinsic),
            Err(msg) => println!("{:?}", msg),
        }
    }

    println!("<-\n");

    Ok(SimpleBlock {
        timestamp,
        block_hash,
        extrinsics,
    })
}

async fn process_extrinsics_in_block(
    api: &RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>,
    extrinsic: &OpaqueExtrinsic,
    block_hash: &H256,
) -> Result<SimpleExtrinsic, BasicError> {
    println!("Processing extrinsic {:?}", extrinsic);

    let fee_details = rpc_ext::query_fee_details(api, &extrinsic, Some(*block_hash)).await?;
    println!("{fee_details:?}");

    Ok(SimpleExtrinsic {
        body: extrinsic.clone(),
        fee_details,
    })
}
