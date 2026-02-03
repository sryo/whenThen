use std::sync::Arc;
use rust_cast::{
    CastDevice,
    channels::{
        media::{Media, StreamType},
        receiver::CastDeviceApp,
    },
};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::errors::{WhenThenError, Result};
use crate::models::{PlaybackState, PlaybackStatusResponse};

/// Connection attempt timeout.
const CONNECT_TIMEOUT_SECS: u64 = 10;

pub struct ChromecastConnection {
    pub device_id: String,
    pub device_name: String,
    device: Arc<Mutex<Option<CastDevice<'static>>>>,
    transport_id: Arc<Mutex<Option<String>>>,
    session_id: Arc<Mutex<Option<String>>>,
    heartbeat_shutdown: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    /// Optional handle to emit events back to the frontend.
    app_handle: Option<tauri::AppHandle>,
}

// CastDevice with thread_safe feature is Send+Sync
unsafe impl Send for ChromecastConnection {}
unsafe impl Sync for ChromecastConnection {}

impl ChromecastConnection {
    pub async fn connect(
        device_id: String,
        device_name: String,
        address: String,
        port: u16,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<Self> {
        let connect_fut = tokio::task::spawn_blocking(move || {
            CastDevice::connect_without_host_verification(address, port)
        });

        let cast_device = tokio::time::timeout(
            std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS),
            connect_fut,
        )
        .await
        .map_err(|_| WhenThenError::CastConnection(format!(
            "Connection to {} timed out after {}s", device_name, CONNECT_TIMEOUT_SECS
        )))?
        .map_err(|e| WhenThenError::CastConnection(format!("Task join error: {e}")))?
        .map_err(|e| WhenThenError::CastConnection(format!("Connect failed: {e}")))?;

        let device = Arc::new(Mutex::new(Some(cast_device)));

        let transport_id;
        let session_id;
        {
            let dev_guard = device.lock().await;
            if let Some(ref dev) = *dev_guard {
                dev.connection.connect("receiver-0")
                    .map_err(|e| WhenThenError::CastConnection(format!("Connection channel: {e}")))?;

                let app = dev.receiver.launch_app(&CastDeviceApp::DefaultMediaReceiver)
                    .map_err(|e| WhenThenError::CastConnection(format!("Launch app: {e}")))?;

                transport_id = app.transport_id.clone();
                session_id = app.session_id.clone();

                dev.connection.connect(&transport_id)
                    .map_err(|e| WhenThenError::CastConnection(format!("Transport connect: {e}")))?;
            } else {
                return Err(WhenThenError::CastConnection("Device not available".into()));
            }
        }

        let conn = Self {
            device_id: device_id.clone(),
            device_name: device_name.clone(),
            device,
            transport_id: Arc::new(Mutex::new(Some(transport_id))),
            session_id: Arc::new(Mutex::new(Some(session_id))),
            heartbeat_shutdown: Arc::new(Mutex::new(None)),
            app_handle,
        };

        conn.start_heartbeat().await;

        info!("Connected to Chromecast: {}", device_name);
        Ok(conn)
    }

    async fn start_heartbeat(&self) {
        let device = self.device.clone();
        let device_id = self.device_id.clone();
        let device_name = self.device_name.clone();
        let app_handle = self.app_handle.clone();
        let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
        *self.heartbeat_shutdown.lock().await = Some(tx);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut rx => break,
                    _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                        let dev = device.lock().await;
                        if let Some(ref d) = *dev {
                            match d.heartbeat.ping() {
                                Ok(_) => {},
                                Err(e) => {
                                    warn!("Heartbeat failed for {}: {}", device_name, e);
                                    if let Some(ref handle) = app_handle {
                                        #[derive(serde::Serialize, Clone)]
                                        struct Disconnected { id: String, name: String, reason: String }
                                        let _ = tauri::Emitter::emit(handle, "chromecast:disconnected", Disconnected {
                                            id: device_id.clone(),
                                            name: device_name.clone(),
                                            reason: format!("Heartbeat failed: {e}"),
                                        });
                                    }
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        });
    }

    pub async fn load_media(
        &self,
        url: String,
        content_type: String,
        _subtitle_url: Option<String>,
    ) -> Result<()> {
        let dev = self.device.lock().await;
        let dev = dev
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("Not connected".into()))?;
        let tid = self.transport_id.lock().await;
        let tid = tid
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("No transport".into()))?;
        let sid = self.session_id.lock().await;
        let sid = sid
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("No session".into()))?;

        dev.media.load(
            tid.as_str(),
            sid.as_str(),
            &Media {
                content_id: url,
                content_type,
                stream_type: StreamType::Buffered,
                duration: None,
                metadata: None,
            },
        )
        .map_err(|e| WhenThenError::CastPlayback(format!("Load media: {e}")))?;

        info!("Media loaded on Chromecast");
        Ok(())
    }

    pub async fn play(&self) -> Result<()> {
        let dev = self.device.lock().await;
        let dev = dev
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("Not connected".into()))?;
        let tid = self.transport_id.lock().await;
        let tid = tid
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("No transport".into()))?;

        let status = dev.media.get_status(tid.as_str(), None)
            .map_err(|e| WhenThenError::CastPlayback(format!("Get status: {e}")))?;

        if let Some(entry) = status.entries.first() {
            dev.media.play(tid.as_str(), entry.media_session_id)
                .map_err(|e| WhenThenError::CastPlayback(format!("Play: {e}")))?;
        }
        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        let dev = self.device.lock().await;
        let dev = dev
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("Not connected".into()))?;
        let tid = self.transport_id.lock().await;
        let tid = tid
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("No transport".into()))?;

        let status = dev.media.get_status(tid.as_str(), None)
            .map_err(|e| WhenThenError::CastPlayback(format!("Get status: {e}")))?;

        if let Some(entry) = status.entries.first() {
            dev.media.pause(tid.as_str(), entry.media_session_id)
                .map_err(|e| WhenThenError::CastPlayback(format!("Pause: {e}")))?;
        }
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let dev = self.device.lock().await;
        let dev = dev
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("Not connected".into()))?;
        let tid = self.transport_id.lock().await;
        let tid = tid
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("No transport".into()))?;

        let status = dev.media.get_status(tid.as_str(), None)
            .map_err(|e| WhenThenError::CastPlayback(format!("Get status: {e}")))?;

        if let Some(entry) = status.entries.first() {
            dev.media.stop(tid.as_str(), entry.media_session_id)
                .map_err(|e| WhenThenError::CastPlayback(format!("Stop: {e}")))?;
        }
        Ok(())
    }

    pub async fn seek(&self, position: f64) -> Result<()> {
        let dev = self.device.lock().await;
        let dev = dev
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("Not connected".into()))?;
        let tid = self.transport_id.lock().await;
        let tid = tid
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("No transport".into()))?;

        let status = dev.media.get_status(tid.as_str(), None)
            .map_err(|e| WhenThenError::CastPlayback(format!("Get status: {e}")))?;

        if let Some(entry) = status.entries.first() {
            dev.media.seek(
                tid.as_str(),
                entry.media_session_id,
                Some(position as f32),
                None,
            )
            .map_err(|e| WhenThenError::CastPlayback(format!("Seek: {e}")))?;
        }
        Ok(())
    }

    pub async fn set_volume(&self, level: f64) -> Result<()> {
        let dev = self.device.lock().await;
        let dev = dev
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("Not connected".into()))?;

        use rust_cast::channels::receiver::Volume;
        dev.receiver.set_volume(Volume {
            level: Some(level as f32),
            muted: None,
        })
        .map_err(|e| WhenThenError::CastPlayback(format!("Set volume: {e}")))?;

        Ok(())
    }

    pub async fn get_status(&self) -> Result<PlaybackStatusResponse> {
        let dev = self.device.lock().await;
        let dev = dev
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("Not connected".into()))?;
        let tid = self.transport_id.lock().await;
        let tid = tid
            .as_ref()
            .ok_or_else(|| WhenThenError::CastConnection("No transport".into()))?;

        let status = dev.media.get_status(tid.as_str(), None)
            .map_err(|e| WhenThenError::CastPlayback(format!("Get status: {e}")))?;

        let device_id = self.device_id.clone();

        let response = if let Some(entry) = status.entries.first() {
            let state = match entry.player_state {
                rust_cast::channels::media::PlayerState::Playing => PlaybackState::Playing,
                rust_cast::channels::media::PlayerState::Paused => PlaybackState::Paused,
                rust_cast::channels::media::PlayerState::Buffering => PlaybackState::Buffering,
                _ => PlaybackState::Idle,
            };

            PlaybackStatusResponse {
                device_id,
                state,
                current_time: entry.current_time.unwrap_or(0.0) as f64,
                duration: entry.media.as_ref().and_then(|m| m.duration).map(|d| d as f64).unwrap_or(0.0),
                volume: 1.0,
                is_muted: false,
                media_title: None,
                content_type: entry.media.as_ref().map(|m| m.content_type.clone()),
            }
        } else {
            PlaybackStatusResponse {
                device_id,
                ..Default::default()
            }
        };

        Ok(response)
    }

    pub async fn disconnect(&self) {
        if let Some(tx) = self.heartbeat_shutdown.lock().await.take() {
            let _ = tx.send(());
        }
        let mut dev = self.device.lock().await;
        *dev = None;
        info!("Disconnected from Chromecast: {}", self.device_name);
    }
}
