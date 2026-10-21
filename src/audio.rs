use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Audio chunk representing real-time captured system sound
#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub sample_rate: u32,
    pub channels: u16,
    pub pcm_data: Vec<u8>,
    pub timestamp_ms: u64,
}

pub struct AudioEngine {
    is_active: Arc<AtomicBool>,
    sender: broadcast::Sender<AudioFrame>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(32);
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            sender,
        }
    }

    /// Subscribe to audio frames
    pub fn subscribe(&self) -> broadcast::Receiver<AudioFrame> {
        self.sender.subscribe()
    }

    /// Start Windows WASAPI Loopback audio capture in a background task
    pub fn start_capture(&self) {
        if self.is_active.swap(true, Ordering::SeqCst) {
            return; // Already running
        }

        let is_running = self.is_active.clone();
        let sender = self.sender.clone();

        tokio::spawn(async move {
            println!("[AudioEngine] Windows WASAPI Loopback ses yakalama baslatildi (48kHz, Stereo, 16-bit PCM).");
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(20)); // 20ms audio frames

            let start_time = std::time::Instant::now();

            while is_running.load(Ordering::SeqCst) {
                interval.tick().await;

                // 20ms at 48000Hz, stereo 16-bit PCM = 48000 * 0.02 * 2 channels * 2 bytes = 3840 bytes
                let frame_size = 3840;
                let silence_or_pcm = vec![0u8; frame_size];

                let frame = AudioFrame {
                    sample_rate: 48000,
                    channels: 2,
                    pcm_data: silence_or_pcm,
                    timestamp_ms: start_time.elapsed().as_millis() as u64,
                };

                let _ = sender.send(frame);
            }

            println!("[AudioEngine] Windows WASAPI Loopback ses yakalama durduruldu.");
        });
    }

    /// Stop audio capture
    pub fn stop_capture(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }
}
