//! Text to speech module.
//!
//! This module provides a client to interact with the Azure Text to Speech service.
//!
//! # Example
//!
//! ```no_run
//!use azure_speech::{synthesizer, Auth, stream::StreamExt};
//! use std::env;
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     tracing_subscriber::fmt()
//!         .with_max_level(tracing::Level::DEBUG)
//!         .init();
//!     // Add your Azure region and subscription key to the environment variables
//!     let auth = Auth::from_subscription(
//!             env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
//!             env::var("AZURE_SUBSCRIPTION_KEY")
//!                 .expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
//!         );
//!
//!     // Set auth and the configuration for the synthesizer
//!     let client = synthesizer::Client::connect(auth,synthesizer::Config::default()).await.expect("to connect to azure");
//!     let mut stream = client.synthesize("Hello world!").await.expect("to synthesize");
//!
//!     while let Some(event) = stream.next().await {
//!         match event {
//!              _ => tracing::info!("Event: {:?}", event)
//!         }
//!     }
//!     Ok(())
//! }
//!
//! ```

mod audio_format;
mod client;
mod config;
mod event;
mod language;
pub mod message;
mod session;
mod text_stream;
mod utils;
mod voice;

mod callback;
pub mod ssml;

pub use audio_format::*;
pub use callback::*;
pub use client::*;
pub use config::*;
pub use event::*;
pub use language::*;
pub use text_stream::*;
pub use voice::*;

#[derive(Clone, Debug, Default)]
pub struct StreamingRequest {
    pub pitch: Option<String>,
    pub rate: Option<String>,
    pub volume: Option<String>,
    pub style: Option<String>,
    pub temperature: Option<f32>,
    pub prefer_locales: Option<String>,
    pub custom_lexicon_url: Option<String>,
}

impl StreamingRequest {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn pitch(mut self, v: impl Into<String>) -> Self {
        self.pitch = Some(v.into());
        self
    }
    pub fn rate(mut self, v: impl Into<String>) -> Self {
        self.rate = Some(v.into());
        self
    }
    pub fn volume(mut self, v: impl Into<String>) -> Self {
        self.volume = Some(v.into());
        self
    }
    pub fn style(mut self, v: impl Into<String>) -> Self {
        self.style = Some(v.into());
        self
    }
    pub fn temperature(mut self, v: f32) -> Self {
        self.temperature = Some(v);
        self
    }
    pub fn prefer_locales(mut self, v: impl Into<String>) -> Self {
        self.prefer_locales = Some(v.into());
        self
    }
    pub fn custom_lexicon_url(mut self, v: impl Into<String>) -> Self {
        self.custom_lexicon_url = Some(v.into());
        self
    }
}
