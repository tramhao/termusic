#[allow(clippy::pedantic)]
pub mod monitor {
    tonic::include_proto!("monitor");
}

use chrono::Utc;
use monitor::{
    ClientHeartbeat, ClientInfo, HeartbeatResponse, RegisterResponse, UnregisterResponse,
    connection_monitor_server::ConnectionMonitor,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

/// Client status structure
#[derive(Debug, Clone)]
pub struct ClientStatus {
    last_heartbeat: i64,
    is_online: bool,
}

/// gRPC Service to track connected clients
#[derive(Clone)]
pub struct MonitorService {
    clients: Arc<RwLock<HashMap<String, ClientStatus>>>,
}

impl MonitorService {
    #[must_use]
    pub fn new() -> Self {
        let service = Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        };

        // Background task: check offline clients every 3s
        let clients_ref = service.clients.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3)).await;
                let now = Utc::now().timestamp_millis();

                let mut map = clients_ref.write().await;
                for (client_id, status) in map.iter_mut() {
                    if now - status.last_heartbeat > 5000 {
                        status.is_online = false;
                        info!("Client [{client_id}] timed out -> offline");
                    }
                }
            }
        });

        service
    }
    /// SYNC safe check for active clients (NO async, NO `block_on`, NO errors)
    #[must_use]
    pub fn has_active_clients_sync(&self) -> bool {
        // Use blocking_write() because we are in a SYNC thread
        let map = self.clients.blocking_write();
        map.values().any(|s| s.is_online)
    }
}

impl Default for MonitorService {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl ConnectionMonitor for MonitorService {
    async fn register(
        &self,
        request: tonic::Request<ClientInfo>,
    ) -> Result<tonic::Response<RegisterResponse>, tonic::Status> {
        let client_id = request.into_inner().client_id;
        let now = Utc::now().timestamp_millis();

        let mut map = self.clients.write().await;
        map.insert(
            client_id,
            ClientStatus {
                last_heartbeat: now,
                is_online: true,
            },
        );
        Ok(tonic::Response::new(RegisterResponse { success: true }))
    }

    async fn heartbeat(
        &self,
        request: tonic::Request<ClientHeartbeat>,
    ) -> Result<tonic::Response<HeartbeatResponse>, tonic::Status> {
        let client_id = request.into_inner().client_id;
        let now = Utc::now().timestamp_millis();

        let mut map = self.clients.write().await;
        if let Some(status) = map.get_mut(&client_id) {
            status.last_heartbeat = now;
            status.is_online = true;
        }
        Ok(tonic::Response::new(HeartbeatResponse { alive: true }))
    }

    async fn unregister(
        &self,
        request: tonic::Request<ClientInfo>,
    ) -> Result<tonic::Response<UnregisterResponse>, tonic::Status> {
        let client_id = request.into_inner().client_id;
        let mut map = self.clients.write().await;
        map.remove(&client_id);
        Ok(tonic::Response::new(UnregisterResponse { success: true }))
    }
}
