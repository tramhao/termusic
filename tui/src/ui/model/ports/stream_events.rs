use std::{fmt::Debug, pin::Pin};

use anyhow::{Context, Result};
use futures_util::Stream;
use termusiclib::player::{PlayerProgress, StreamUpdates, UpdateEvents};
use tokio_stream::StreamExt;
use tuirealm::{
    Event,
    listener::{ListenerResult, PollAsync},
};

use crate::ui::{model::UserEvent, msg::Msg};

pub type WrappedStreamEvents = Pin<Box<dyn Stream<Item = Result<StreamUpdates>> + Send>>;

/// tuirealm async port to provide events that are not "common" in tuirealm.
pub struct PortStreamEvents(WrappedStreamEvents);

impl PortStreamEvents {
    pub fn new(stream_events: WrappedStreamEvents) -> Self {
        Self(stream_events)
    }
}

impl Debug for PortStreamEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PortStreamEvents")
            .field(&"<stream>")
            .finish()
    }
}

#[tuirealm::async_trait]
impl PollAsync<UserEvent> for PortStreamEvents {
    async fn poll(&mut self) -> ListenerResult<Option<Event<UserEvent>>> {
        match self.0.next().await {
            Some(ev) => {
                let ev = match ev
                    .map(UpdateEvents::try_from)
                    .context("Conversion from StreamUpdates to UpdateEvents failed!")
                {
                    Ok(v) => v,
                    Err(err) => {
                        error!("Failed to convert UpdateEvent: {err:#?}");
                        return Ok(None);
                    }
                };

                // dont log progress events, as that spams the log
                if log::log_enabled!(log::Level::Debug) && !is_progress(&ev) {
                    debug!("Stream Event: {ev:?}");
                }

                // just exit on first error, but still print it first
                let Ok(ev) = ev else {
                    return Ok(None);
                };
                Ok(Some(Event::User(UserEvent::Forward(Msg::StreamUpdate(ev)))))
            }
            None => Ok(None),
        }
    }
}

/// Determine if a given event is a [`UpdateEvents::Progress`].
fn is_progress(ev: &Result<UpdateEvents>) -> bool {
    if let Ok(ev) = ev {
        std::mem::discriminant(ev)
            == std::mem::discriminant(&UpdateEvents::Progress(PlayerProgress {
                position: None,
                total_duration: None,
            }))
    } else {
        false
    }
}
