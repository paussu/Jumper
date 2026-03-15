use raylib::prelude::*;

const MUSIC_SAMPLE_RATE: u32 = 22_050;
const MUSIC_BPM: f32 = 132.0;

pub struct AudioFx<'aud> {
    muted: bool,
    start: Sound<'aud>,
    jump: Sound<'aud>,
    double_jump: Sound<'aud>,
    land: Sound<'aud>,
    coin: Sound<'aud>,
    clear: Sound<'aud>,
    hit: Sound<'aud>,
}

pub struct BackgroundMusic<'aud> {
    muted: bool,
    volume: f32,
    music: Music<'aud>,
    music_bytes: Vec<u8>,
}

impl<'aud> AudioFx<'aud> {
    pub fn new(audio: &'aud RaylibAudio) -> Result<Self, String> {
        let start = sound_from_wav_bytes(audio, &build_start_wav())?;
        let jump = sound_from_wav_bytes(audio, &build_jump_wav())?;
        let double_jump = sound_from_wav_bytes(audio, &build_double_jump_wav())?;
        let land = sound_from_wav_bytes(audio, &build_land_wav())?;
        let coin = sound_from_wav_bytes(audio, &build_coin_wav())?;
        let clear = sound_from_wav_bytes(audio, &build_clear_wav())?;
        let hit = sound_from_wav_bytes(audio, &build_hit_wav())?;

        start.set_volume(0.42);
        jump.set_volume(0.34);
        double_jump.set_volume(0.38);
        land.set_volume(0.26);
        coin.set_volume(0.42);
        clear.set_volume(0.24);
        hit.set_volume(0.48);

        Ok(Self {
            muted: false,
            start,
            jump,
            double_jump,
            land,
            coin,
            clear,
            hit,
        })
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn play_start(&self) {
        if !self.muted {
            self.start.play();
        }
    }

    pub fn play_jump(&self) {
        if !self.muted {
            self.jump.play();
        }
    }

    pub fn play_double_jump(&self) {
        if !self.muted {
            self.double_jump.play();
        }
    }

    pub fn play_land(&self) {
        if !self.muted {
            self.land.play();
        }
    }

    pub fn play_coin(&self) {
        if !self.muted {
            self.coin.play();
        }
    }

    pub fn play_clear(&self) {
        if !self.muted {
            self.clear.play();
        }
    }

    pub fn play_hit(&self) {
        if !self.muted {
            self.hit.play();
        }
    }
}

impl<'aud> BackgroundMusic<'aud> {
    pub fn new(audio: &'aud RaylibAudio) -> Result<Self, String> {
        let music_bytes = build_background_music_wav();
        let music = audio
            .new_music_from_memory(".wav", &music_bytes)
            .map_err(|error| error.to_string())?;

        let background_music = Self {
            muted: false,
            volume: 0.18,
            music,
            music_bytes,
        };

        background_music.music.set_volume(background_music.volume);
        background_music.music.play_stream();

        Ok(background_music)
    }

    pub fn update(&mut self) {
        let _keep_alive = self.music_bytes.len();
        self.music.update_stream();

        if self.music.get_time_played() >= self.music.get_time_length() - 0.05 {
            self.music.seek_stream(0.0);
        }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.music.set_volume(if self.muted { 0.0 } else { self.volume });
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }
}

fn sound_from_wav_bytes<'aud>(audio: &'aud RaylibAudio, bytes: &[u8]) -> Result<Sound<'aud>, String> {
    let wave = audio
        .new_wave_from_memory(".wav", bytes)
        .map_err(|error| error.to_string())?;
    audio
        .new_sound_from_wave(&wave)
        .map_err(|error| error.to_string())
}

fn synthesize_wav(duration_seconds: f32, sample_rate: u32, mut sample_fn: impl FnMut(usize, f32, f32) -> f32) -> Vec<u8> {
    let sample_count = (duration_seconds * sample_rate as f32).max(1.0) as usize;
    let data_bytes = sample_count * std::mem::size_of::<i16>();
    let mut bytes = Vec::with_capacity(44 + data_bytes);

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data_bytes as u32).to_le_bytes());

    for index in 0..sample_count {
        let t = index as f32 / sample_rate as f32;
        let progress = index as f32 / sample_count as f32;
        let sample = sample_fn(index, t, progress).clamp(-1.0, 1.0);
        let amplitude = (sample * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&amplitude.to_le_bytes());
    }

    bytes
}

fn build_start_wav() -> Vec<u8> {
    synthesize_wav(0.22, 22_050, |_, t, progress| {
        let split = if progress < 0.45 { 0.0 } else { 1.0 };
        let freq = if split == 0.0 { 523.25 } else { 783.99 };
        let envelope = smooth_envelope(progress, 0.02, 0.18) * 0.45;
        ((std::f32::consts::TAU * freq * t).sin() + 0.35 * (std::f32::consts::TAU * freq * 2.0 * t).sin()) * envelope
    })
}

fn build_jump_wav() -> Vec<u8> {
    synthesize_wav(0.13, 22_050, |_, t, progress| {
        let freq = lerp(460.0, 720.0, progress);
        let envelope = smooth_envelope(progress, 0.02, 0.24) * 0.42;
        ((std::f32::consts::TAU * freq * t).sin() + 0.22 * (std::f32::consts::TAU * freq * 2.0 * t).sin()) * envelope
    })
}

fn build_double_jump_wav() -> Vec<u8> {
    synthesize_wav(0.16, 22_050, |_, t, progress| {
        let vibrato = (std::f32::consts::TAU * 11.0 * t).sin() * 26.0;
        let freq = lerp(620.0, 980.0, progress) + vibrato;
        let sparkle = (std::f32::consts::TAU * (freq * 1.5) * t).sin() * 0.18;
        let envelope = smooth_envelope(progress, 0.01, 0.2) * 0.42;
        ((std::f32::consts::TAU * freq * t).sin() + sparkle) * envelope
    })
}

fn build_land_wav() -> Vec<u8> {
    synthesize_wav(0.12, 22_050, |index, t, progress| {
        let freq = lerp(180.0, 72.0, progress);
        let noise = pseudo_noise(index) * 0.18;
        let envelope = smooth_envelope(progress, 0.0, 0.28) * 0.5;
        ((std::f32::consts::TAU * freq * t).sin() * 0.85 + noise) * envelope
    })
}

fn build_coin_wav() -> Vec<u8> {
    synthesize_wav(0.14, 22_050, |_, t, progress| {
        let base = if progress < 0.45 { 960.0 } else { 1320.0 };
        let overtone = base * 1.5;
        let envelope = smooth_envelope(progress, 0.01, 0.14) * 0.4;
        ((std::f32::consts::TAU * base * t).sin() + 0.45 * (std::f32::consts::TAU * overtone * t).sin()) * envelope
    })
}

fn build_clear_wav() -> Vec<u8> {
    synthesize_wav(0.08, 22_050, |_, t, progress| {
        let freq = lerp(300.0, 420.0, progress);
        let envelope = smooth_envelope(progress, 0.0, 0.35) * 0.32;
        (std::f32::consts::TAU * freq * t).sin() * envelope
    })
}

fn build_hit_wav() -> Vec<u8> {
    synthesize_wav(0.24, 22_050, |index, t, progress| {
        let freq = lerp(360.0, 95.0, progress);
        let square = if (std::f32::consts::TAU * freq * t).sin() >= 0.0 { 1.0 } else { -1.0 };
        let noise = pseudo_noise(index) * 0.24;
        let envelope = smooth_envelope(progress, 0.0, 0.12) * 0.34;
        (square * 0.75 + noise) * envelope
    })
}

fn build_background_music_wav() -> Vec<u8> {
    synthesize_wav(14.55, MUSIC_SAMPLE_RATE, |index, t, _| compose_music_sample(index as u64, t))
}

fn compose_music_sample(sample_cursor: u64, t: f32) -> f32 {
    const REST: i32 = -1;
    const LEAD_PATTERN: [i32; 32] = [
        72, REST, 76, REST, 79, REST, 76, REST,
        77, REST, 76, REST, 72, REST, 69, REST,
        67, REST, 72, REST, 74, REST, 72, REST,
        71, REST, 69, REST, 67, REST, 64, REST,
    ];
    const BASS_PATTERN: [i32; 4] = [45, 41, 48, 43];

    let sample_rate = MUSIC_SAMPLE_RATE as f32;
    let step_samples = sample_rate * 60.0 / MUSIC_BPM / 4.0;
    let step_pos = sample_cursor as f32 / step_samples;
    let step_index = step_pos.floor() as usize;
    let step_progress = step_pos.fract();
    let measure_step = step_index % 16;

    let lead_note = LEAD_PATTERN[step_index % LEAD_PATTERN.len()];
    let lead = if lead_note >= 0 {
        let freq = midi_to_freq(lead_note as f32);
        let body = pulse_wave(t, freq, 0.24) * 0.72 + triangle_wave(t, freq * 2.0) * 0.28;
        body * note_envelope(step_progress, 0.04, 0.72) * 0.24
    } else {
        0.0
    };

    let bass_phrase_pos = step_pos / 8.0;
    let bass_index = bass_phrase_pos.floor() as usize % BASS_PATTERN.len();
    let bass_progress = bass_phrase_pos.fract();
    let bass_freq = midi_to_freq(BASS_PATTERN[bass_index] as f32);
    let bass = (triangle_wave(t, bass_freq) * 0.78 + pulse_wave(t, bass_freq * 0.5, 0.5) * 0.22)
        * note_envelope(bass_progress, 0.02, 0.22)
        * 0.26;

    let pad_freq = midi_to_freq(BASS_PATTERN[bass_index] as f32 + 12.0);
    let pad = ((std::f32::consts::TAU * pad_freq * t).sin() * 0.65
        + (std::f32::consts::TAU * pad_freq * 1.5 * t).sin() * 0.35)
        * note_envelope((step_pos / 16.0).fract(), 0.1, 0.1)
        * 0.10;

    let kick = if measure_step == 0 || measure_step == 8 {
        let kick_freq = lerp(132.0, 48.0, step_progress);
        (std::f32::consts::TAU * kick_freq * t).sin() * (1.0 - step_progress).powf(3.4) * 0.38
    } else {
        0.0
    };

    let snare = if measure_step == 4 || measure_step == 12 {
        (pseudo_noise(sample_cursor as usize * 3) * 0.8 + triangle_wave(t, 196.0) * 0.2)
            * (1.0 - step_progress).powf(5.0)
            * 0.22
    } else {
        0.0
    };

    let hat = if measure_step % 2 == 0 {
        pseudo_noise(sample_cursor as usize * 7) * (1.0 - step_progress).powf(11.0) * 0.06
    } else {
        0.0
    };

    (lead + bass + pad + kick + snare + hat) * 0.84
}

fn midi_to_freq(note: f32) -> f32 {
    440.0 * 2.0_f32.powf((note - 69.0) / 12.0)
}

fn pulse_wave(time: f32, frequency: f32, duty_cycle: f32) -> f32 {
    let phase = (time * frequency).fract();
    if phase < duty_cycle { 1.0 } else { -1.0 }
}

fn triangle_wave(time: f32, frequency: f32) -> f32 {
    let phase = (time * frequency).fract();
    1.0 - 4.0 * (phase - 0.5).abs()
}

fn note_envelope(progress: f32, attack: f32, release: f32) -> f32 {
    let attack_gain = if progress < attack {
        (progress / attack.max(f32::EPSILON)).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let release_start = release.clamp(0.0, 1.0);
    let release_gain = if progress <= release_start {
        1.0
    } else {
        ((1.0 - progress) / (1.0 - release_start).max(f32::EPSILON)).clamp(0.0, 1.0)
    };

    attack_gain * release_gain
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn smooth_envelope(progress: f32, attack: f32, release: f32) -> f32 {
    let attack_gain = if attack <= 0.0 {
        1.0
    } else {
        (progress / attack).clamp(0.0, 1.0)
    };
    let release_start = (1.0 - release).clamp(0.0, 1.0);
    let release_gain = if progress <= release_start {
        1.0
    } else {
        ((1.0 - progress) / release.max(f32::EPSILON)).clamp(0.0, 1.0)
    };
    attack_gain * release_gain
}

fn pseudo_noise(index: usize) -> f32 {
    let x = index as f32 * 12.9898;
    ((x.sin() * 43_758.547).fract() * 2.0) - 1.0
}
