use anyhow::Result;
use std::pin::Pin;
use termusiclib::player::protobuf::common::Empty;
use termusiclib::player::protobuf::stream::stream_events_server::StreamEvents;
use termusiclib::player::protobuf::stream::{StreamUpdates, UpdateMissedEvents, stream_updates};
use termusicplayback::StreamTX;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct StreamEventsService {
    stream_tx: StreamTX,
}

impl StreamEventsService {
    pub fn new(stream_tx: StreamTX) -> Self {
        Self { stream_tx }
    }
}

#[tonic::async_trait]
impl StreamEvents for StreamEventsService {
    type SubscribeServerUpdatesStream =
        Pin<Box<dyn Stream<Item = Result<StreamUpdates, Status>> + Send>>;
    async fn subscribe_server_updates(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<Self::SubscribeServerUpdatesStream>, Status> {
        let rx = self.stream_tx.subscribe();

        // map to the grpc types
        let receiver_stream = BroadcastStream::new(rx).map(|res| match res {
            Ok(ev) => Ok(ev.into()),
            Err(err) => {
                let BroadcastStreamRecvError::Lagged(amount) = err;
                Ok(StreamUpdates {
                    r#type: Some(stream_updates::Type::MissedEvents(UpdateMissedEvents {
                        amount,
                    })),
                })

                // else case if ever necessary
                // Err(Status::from_error(Box::new(err)))
            }
        });
        Ok(Response::new(Box::pin(receiver_stream)))
    }
}
