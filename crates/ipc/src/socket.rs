// Unix domain socket communication
//
// This module provides Unix domain socket based IPC for high-performance
// communication between the compositor and client applications.

use compositor_utils::prelude::*;
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use bytes::Bytes;
use std::path::Path;

/// Unix socket server for IPC communication
pub struct SocketServer {
    listener: Option<UnixListener>,
    socket_path: String,
}

impl SocketServer {
    /// Create a new socket server
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Result<Self> {
        let path_str = socket_path.as_ref().to_string_lossy().to_string();
        info!("Creating socket server at: {}", path_str);
        
        Ok(Self {
            listener: None,
            socket_path: path_str,
        })
    }
    
    /// Start listening for connections
    pub async fn start(&mut self) -> Result<()> {
        // Remove existing socket file if it exists
        if Path::new(&self.socket_path).exists() {
            std::fs::remove_file(&self.socket_path)?;
        }
        
        let listener = UnixListener::bind(&self.socket_path)?;
        info!("Socket server listening on: {}", self.socket_path);
        
        self.listener = Some(listener);
        Ok(())
    }
    
    /// Accept incoming connections
    pub async fn accept(&self) -> Result<UnixStream> {
        if let Some(ref listener) = self.listener {
            let (stream, _) = listener.accept().await?;
            Ok(stream)
        } else {
            Err(CompositorError::ipc("Socket server not started").into())
        }
    }
}

/// Socket client for connecting to the compositor
pub struct SocketClient {
    stream: Option<UnixStream>,
}

impl SocketClient {
    /// Create a new socket client
    pub fn new() -> Self {
        Self { stream: None }
    }
    
    /// Connect to the compositor socket
    pub async fn connect<P: AsRef<Path>>(&mut self, socket_path: P) -> Result<()> {
        let stream = UnixStream::connect(socket_path).await?;
        self.stream = Some(stream);
        Ok(())
    }
    
    /// Send data to the compositor
    pub async fn send(&mut self, _data: Bytes) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            let _framed = FramedWrite::new(stream, LengthDelimitedCodec::new());
            // TODO: Implement actual sending
            Ok(())
        } else {
            Err(CompositorError::ipc("Not connected").into())
        }
    }
}

impl Default for SocketClient {
    fn default() -> Self {
        Self::new()
    }
}
