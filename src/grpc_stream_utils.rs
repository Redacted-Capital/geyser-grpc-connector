use futures::{Stream, StreamExt};
use log::{debug, trace};
use std::pin::pin;
use tokio::spawn;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;

pub async fn channelize_stream<T>(
    grpc_source_stream: impl Stream<Item = T> + Send + 'static,
) -> (Receiver<T>, JoinHandle<()>)
where
    T: Clone + Send + 'static,
{
    // note: broadcast channel will fill up if receivers are slacking
    let (tx, multiplexed_messages) = tokio::sync::broadcast::channel::<T>(1000);

    let jh_channelizer = spawn(async move {
        let mut source_stream = pin!(grpc_source_stream);
        'main_loop: while let Some(payload) = source_stream.next().await {
            match tx.send(payload) {
                Ok(receivers) => {
                    trace!("sent data to {} receivers", receivers);
                }
                Err(send_error) => match send_error {
                    SendError(_) => {
                        debug!("no active receivers - skipping message");
                        continue 'main_loop;
                    }
                },
            };
        }
        panic!("channelizer task failed");
    });

    (multiplexed_messages, jh_channelizer)
}
