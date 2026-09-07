use anyhow::Result;
use termusiclib::player::protobuf::{common::Empty, server::server_control_server::ServerControl};
use termusicplayback::{PlayerCmd, PlayerCmdSender};
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct ServerControlService {
    cmd_tx: PlayerCmdSender,
}

impl ServerControlService {
    pub fn new(cmd_tx: PlayerCmdSender) -> Self {
        Self { cmd_tx }
    }
}

impl ServerControlService {
    /// Send a command without a callback.
    fn command(&self, cmd: PlayerCmd) {
        if let Err(e) = self.cmd_tx.send(cmd.clone()) {
            error!("error {cmd:?}: {e}");
        }
    }
}

#[tonic::async_trait]
impl ServerControl for ServerControlService {
    async fn reload_config(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let reply = Empty {};
        self.command(PlayerCmd::ReloadConfig);

        Ok(Response::new(reply))
    }

    async fn quit_server(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let reply = Empty {};
        self.command(PlayerCmd::Quit(crate::quit_sources::CLIENT));

        Ok(Response::new(reply))
    }
}
