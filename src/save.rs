//! Save data: settings, unlock progress, and best scores, serialised
//! as one bounded text line the shell keeps in localStorage.
//!
//! Format: `HF1;calm=<0|1>;glow=<0|1>;sound=<0|1>;unlocked=<0..3>;
//! best=<0..1000>,<0..1000>,<0..1000>`. Parsing is strict: wrong magic,
//! missing fields, out-of-range values, or trailing junk are typed
//! errors and never mutate caller state (validate → prepare → commit).

use crate::levels::LEVEL_COUNT;

/// Maximum accepted save length in bytes.
pub const SAVE_MAX_BYTES: usize = 256;

/// Player-facing settings. The safety-relevant ones (calm mode) are
/// reachable from the first settings visit and never gated (R-403).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Reduce secondary-motion amplitude.
    pub calm_mode: bool,
    /// Iris-glow layer default.
    pub glow_default: bool,
    /// Generated audio on/off (the shell owns playback).
    pub sound_on: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            calm_mode: false,
            glow_default: true,
            sound_on: true,
        }
    }
}

/// The full persisted record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveData {
    /// Settings.
    pub settings: Settings,
    /// Highest level index unlocked (0-based; all start at 0).
    pub unlocked: usize,
    /// Best appraisal score per level, 0..=1000.
    pub best: [u16; LEVEL_COUNT],
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            unlocked: 0,
            best: [0; LEVEL_COUNT],
        }
    }
}

/// Why a save string was rejected.
#[derive(Debug, PartialEq, Eq)]
pub enum SaveError {
    /// Input longer than [`SAVE_MAX_BYTES`].
    TooLong,
    /// Magic/version mismatch.
    BadMagic,
    /// A required field is absent or unparseable.
    BadField(&'static str),
    /// A field parsed but is outside its contract range.
    OutOfRange(&'static str),
}

impl SaveData {
    /// Serialise. Always succeeds; output is well under the cap.
    pub fn serialise(&self) -> String {
        format!(
            "HF1;calm={};glow={};sound={};unlocked={};best={},{},{}",
            self.settings.calm_mode as u8,
            self.settings.glow_default as u8,
            self.settings.sound_on as u8,
            self.unlocked,
            self.best[0],
            self.best[1],
            self.best[2],
        )
    }

    /// Parse strictly. On any error the function returns before any
    /// caller state is touched.
    pub fn parse(text: &str) -> Result<Self, SaveError> {
        if text.len() > SAVE_MAX_BYTES {
            return Err(SaveError::TooLong);
        }
        let mut parts = text.trim().split(';');
        if parts.next() != Some("HF1") {
            return Err(SaveError::BadMagic);
        }
        let mut data = SaveData::default();
        let mut seen = 0u8;
        for field in parts {
            let (key, value) = field.split_once('=').ok_or(SaveError::BadField("shape"))?;
            match key {
                "calm" => {
                    data.settings.calm_mode = parse_flag(value, "calm")?;
                    seen |= 1;
                }
                "glow" => {
                    data.settings.glow_default = parse_flag(value, "glow")?;
                    seen |= 2;
                }
                "sound" => {
                    data.settings.sound_on = parse_flag(value, "sound")?;
                    seen |= 4;
                }
                "unlocked" => {
                    let n: i64 = value.parse().map_err(|_| SaveError::BadField("unlocked"))?;
                    if !(0..LEVEL_COUNT as i64).contains(&n) {
                        return Err(SaveError::OutOfRange("unlocked"));
                    }
                    data.unlocked = n as usize;
                    seen |= 8;
                }
                "best" => {
                    let mut scores = value.split(',');
                    for slot in &mut data.best {
                        let raw = scores.next().ok_or(SaveError::BadField("best"))?;
                        let n: i64 = raw.parse().map_err(|_| SaveError::BadField("best"))?;
                        if !(0..=1000).contains(&n) {
                            return Err(SaveError::OutOfRange("best"));
                        }
                        *slot = n as u16;
                    }
                    if scores.next().is_some() {
                        return Err(SaveError::BadField("best"));
                    }
                    seen |= 16;
                }
                _ => return Err(SaveError::BadField("unknown")),
            }
        }
        if seen != 0b1_1111 {
            return Err(SaveError::BadField("missing"));
        }
        Ok(data)
    }
}

fn parse_flag(value: &str, field: &'static str) -> Result<bool, SaveError> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(SaveError::BadField(field)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_is_identity() {
        let data = SaveData {
            settings: Settings {
                calm_mode: true,
                glow_default: false,
                sound_on: true,
            },
            unlocked: 2,
            best: [750, 0, 1000],
        };
        assert_eq!(SaveData::parse(&data.serialise()), Ok(data));
    }

    #[test]
    fn default_round_trips() {
        let data = SaveData::default();
        assert_eq!(SaveData::parse(&data.serialise()), Ok(data));
    }

    // ---- adversarial ----

    #[test]
    fn garbage_is_rejected() {
        assert_eq!(SaveData::parse(""), Err(SaveError::BadMagic));
        assert_eq!(SaveData::parse("HF2;calm=0"), Err(SaveError::BadMagic));
        assert!(SaveData::parse("HF1;calm=2;glow=0;sound=0;unlocked=0;best=0,0,0").is_err());
        assert!(SaveData::parse("HF1;calm=0;glow=0;sound=0;unlocked=9;best=0,0,0").is_err());
        assert!(SaveData::parse("HF1;calm=0;glow=0;sound=0;unlocked=0;best=0,0,1001").is_err());
        assert!(SaveData::parse("HF1;calm=0;glow=0;sound=0;unlocked=0;best=0,0").is_err());
        assert!(SaveData::parse("HF1;calm=0;glow=0;sound=0;unlocked=0;best=0,0,0;junk=1").is_err());
    }

    #[test]
    fn oversized_is_rejected() {
        let big = format!("HF1;{}", "x".repeat(SAVE_MAX_BYTES));
        assert_eq!(SaveData::parse(&big), Err(SaveError::TooLong));
    }

    #[test]
    fn truncated_is_rejected() {
        let good = SaveData::default().serialise();
        for cut in 0..good.len() {
            assert!(SaveData::parse(&good[..cut]).is_err(), "cut {cut} parsed");
        }
    }
}
