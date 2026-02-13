//! Sample-accurate audio waveform buffer.
//!
//! Records pin-level transitions with CPU tick timestamps during each frame,
//! then converts them to PCM audio samples at any target sample rate.
//! This provides much higher audio fidelity than per-frame frequency detection,
//! especially for GPIO bit-bang audio and rapid frequency changes.
//!
//! ## Architecture
//!
//! Two independent channels (left / right) each maintain a ring buffer of
//! [`AudioEdge`] events. At the end of each frame, the frontend calls
//! [`AudioBuffer::render_samples`] to convert edges into `f32` PCM data.

/// A single pin-level transition event.
#[derive(Debug, Clone, Copy)]
pub struct AudioEdge {
    /// CPU tick when the transition occurred.
    pub tick: u64,
    /// Pin level after transition (true = high).
    pub level: bool,
}

/// Per-channel edge buffer with current pin state.
#[derive(Debug)]
pub struct ChannelBuffer {
    /// Recorded edges this frame.
    edges: Vec<AudioEdge>,
    /// Current pin level (carried across frames).
    pub level: bool,
}

impl ChannelBuffer {
    pub fn new() -> Self {
        ChannelBuffer { edges: Vec::with_capacity(4096), level: false }
    }

    /// Record a pin transition.
    #[inline]
    pub fn push(&mut self, tick: u64, level: bool) {
        if level != self.level {
            self.edges.push(AudioEdge { tick, level });
            self.level = level;
        }
    }

    /// Clear edges for next frame (pin level is preserved).
    pub fn clear(&mut self) {
        self.edges.clear();
    }

    /// Number of edges recorded this frame.
    pub fn len(&self) -> usize { self.edges.len() }

    /// Access the raw edge slice.
    pub fn edges(&self) -> &[AudioEdge] { &self.edges }
}

/// Stereo audio buffer: left (Speaker1 / PC6) and right (Speaker2 / PB5).
pub struct AudioBuffer {
    pub left: ChannelBuffer,
    pub right: ChannelBuffer,
    /// Frame start tick (set at beginning of run_frame).
    pub frame_start: u64,
    /// Frame end tick (set at end of run_frame).
    pub frame_end: u64,
}

impl AudioBuffer {
    pub fn new() -> Self {
        AudioBuffer {
            left: ChannelBuffer::new(),
            right: ChannelBuffer::new(),
            frame_start: 0,
            frame_end: 0,
        }
    }

    /// Begin a new frame: store start tick, clear edge buffers.
    pub fn begin_frame(&mut self, tick: u64) {
        self.frame_start = tick;
        self.left.clear();
        self.right.clear();
    }

    /// End the current frame: store end tick.
    pub fn end_frame(&mut self, tick: u64) {
        self.frame_end = tick;
    }

    /// Returns true if any audio activity was recorded this frame.
    pub fn has_audio(&self) -> bool {
        self.left.len() > 0 || self.right.len() > 0
    }

    /// Render edge buffers to interleaved stereo f32 PCM samples.
    ///
    /// `out` receives interleaved [L, R, L, R, ...] samples at `sample_rate` Hz.
    /// `volume` scales the square wave amplitude (0.0–1.0).
    /// `clock_hz` is the CPU clock frequency (16 MHz).
    ///
    /// Returns the number of stereo sample pairs written.
    pub fn render_samples(
        &self,
        out: &mut Vec<f32>,
        sample_rate: u32,
        clock_hz: u32,
        volume: f32,
    ) -> usize {
        let frame_ticks = self.frame_end.saturating_sub(self.frame_start);
        if frame_ticks == 0 { return 0; }

        // Number of samples for this frame duration
        let num_samples = ((frame_ticks as f64 * sample_rate as f64) / clock_hz as f64)
            .ceil() as usize;

        out.clear();
        out.reserve(num_samples * 2);

        let ticks_per_sample = clock_hz as f64 / sample_rate as f64;
        let start = self.frame_start;

        let mut li = 0usize; // left edge index
        let mut ri = 0usize; // right edge index
        let l_edges = self.left.edges();
        let r_edges = self.right.edges();

        // Initial levels: the carried-over state from before the first edge.
        // If edges exist, the level before the first edge is the opposite of the
        // first edge's target level (since it was a transition TO that level).
        // If no edges, the current channel level is used (steady state).
        let mut l_level = if l_edges.is_empty() {
            self.left.level
        } else {
            !l_edges[0].level // level before the first transition
        };
        let mut r_level = if r_edges.is_empty() {
            self.right.level
        } else {
            !r_edges[0].level
        };

        for i in 0..num_samples {
            let sample_tick = start + (i as f64 * ticks_per_sample) as u64;

            // Advance left channel to current tick
            while li < l_edges.len() && l_edges[li].tick <= sample_tick {
                l_level = l_edges[li].level;
                li += 1;
            }
            // Advance right channel to current tick
            while ri < r_edges.len() && r_edges[ri].tick <= sample_tick {
                r_level = r_edges[ri].level;
                ri += 1;
            }

            let l_sample = if l_level { volume } else { -volume };
            let r_sample = if r_level { volume } else { -volume };
            out.push(l_sample);
            out.push(r_sample);
        }

        num_samples
    }
}
