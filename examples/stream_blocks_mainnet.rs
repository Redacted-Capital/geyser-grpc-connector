use log::{info};
use tokio::sync::broadcast::{Receiver};
use tokio::time::{sleep, Duration};
use yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeUpdateBlock};
use geyser_grpc_connector::grpcmultiplex_fastestwins::{create_multiplex, GrpcSourceConfig};

fn start_example_consumer(blocks_notifier: Receiver<Box<SubscribeUpdateBlock>>) {
    tokio::spawn(async move {
        let mut blocks_notifier = blocks_notifier;
        loop {
            let block = blocks_notifier.recv().await.unwrap();
            info!("received block #{} with {} txs", block.slot, block.transactions.len());
        }
    });
}

#[tokio::main]
pub async fn main() {
    // RUST_LOG=info,grpc_using_streams=debug
    tracing_subscriber::fmt::init();
    // console_subscriber::init();

    // mango validator (mainnet)
    let grpc_addr_mainnet_triton = "http://202.8.9.108:10000".to_string();
    // via toxiproxy
    let grpc_addr_mainnet_triton_toxi = "http://127.0.0.1:10001".to_string();
    // ams81 (mainnet)
    let grpc_addr_mainnet_ams81 = "http://202.8.8.12:10000".to_string();
    // testnet - NOTE: this connection has terrible lags (almost 5 minutes)
    // let grpc_addr = "http://147.28.169.13:10000".to_string();

    let (block_sx, blocks_notifier) = tokio::sync::broadcast::channel(1000);

    let green_config = GrpcSourceConfig::new("triton".to_string(), grpc_addr_mainnet_triton, None);
    let blue_config = GrpcSourceConfig::new("mangoams81".to_string(), grpc_addr_mainnet_ams81, None);
    let toxiproxy_config = GrpcSourceConfig::new("toxiproxy".to_string(), grpc_addr_mainnet_triton_toxi, None);

    create_multiplex(
        vec![green_config, blue_config, toxiproxy_config],
        CommitmentLevel::Confirmed,
        block_sx);

    start_example_consumer(blocks_notifier);

    // "infinite" sleep
    sleep(Duration::from_secs(1800)).await;

}
