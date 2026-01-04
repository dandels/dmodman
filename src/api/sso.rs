use super::ApiError;
use std::net::TcpStream;
// use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
// use tokio::net::TcpStream;
// use tokio_stream::StreamExt;
use tungstenite::{stream::MaybeTlsStream, WebSocket};
// use tungstenite::{MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

/* Documentation for SSO integration:
 * https://github.com/Nexus-Mods/sso-integration-demo */

pub struct SsoClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    session_params: SsoSession,
}

#[derive(Deserialize, Serialize)]
pub struct SsoSession {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    protocol: u8,
}

#[derive(Deserialize)]
pub struct SsoResponse {
    pub success: bool,
    pub data: ResponseData,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct ResponseData {
    pub api_key: Option<String>,
    connection_token: Option<String>,
}

const SSO_ENDPOINT: &str = "wss://sso.nexusmods.com";
const APP_SLUG: &str = "dmodman";

impl SsoClient {
    pub fn new() -> Result<SsoClient, ApiError> {
        let session_params = SsoSession {
            id: Uuid::new_v4().to_string(),
            token: None,
            protocol: 2,
        };

        let (socket, _response) = tungstenite::connect(SSO_ENDPOINT)?;
        Ok(Self { socket, session_params })
    }

    pub fn start_flow(&mut self) -> Result<(), ApiError> {
        let msg = serde_json::to_string(&self.session_params).unwrap();

        self.socket.send(msg.into())?;
        self.socket.flush()?;
        // Unwrap here should be safe because the internal value shouldn't be a None
        let resp = self.socket.read()?;

        // set connection_token on the first (and probably only) time we connect
        if self.session_params.token.is_none() {
            let sso_resp: SsoResponse = serde_json::from_str(&resp.into_text().unwrap())?;
            self.session_params.token = Some(sso_resp.data.connection_token.unwrap());
        }
        Ok(())
    }

    pub fn get_url(&self) -> String {
        format!("https://www.nexusmods.com/sso?id={}&application={}", self.session_params.id, APP_SLUG)
    }

    pub fn wait_apikey_response(&mut self) -> Result<SsoResponse, ApiError> {
        let resp = self.socket.read()?;
        let sso_resp: SsoResponse = serde_json::from_str(&resp.into_text().unwrap())?;
        Ok(sso_resp)
    }

    pub fn close_connection(&mut self) -> Result<(), ApiError> {
        Ok(self.socket.close(None)?)
    }
}
