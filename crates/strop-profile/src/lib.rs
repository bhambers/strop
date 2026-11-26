//! TOML profile persistence for strop headset configuration.
//!
//! The profile stores EQ slot settings that are re-applied when the dongle
//! reconnects. Lives at `~/.config/strop/profile.toml` (XDG-compatible).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Number of EQ bands (10).
pub const BANDS: usize = 10;

/// Number of EQ slots (9).
pub const SLOTS: usize = 9;

/// One EQ slot as the profile records it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Slot {
    pub index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub bands: Vec<i32>,
}

/// The whole saved state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Slot to select after writing. -1 means leave it alone.
    #[serde(default = "unset")]
    pub active: i32,
    /// Mic monitoring level, 0–255. -1 means leave it alone.
    #[serde(default = "unset")]
    pub sidetone: i32,
    #[serde(default)]
    pub slots: Vec<Slot>,
}

fn unset() -> i32 {
    -1
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            active: -1,
            sidetone: -1,
            slots: Vec::new(),
        }
    }
}

impl Profile {
    /// Create a new empty profile that changes nothing when applied.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the profile against device constraints.
    pub fn validate(&self) -> Result<(), String> {
        if self.active >= SLOTS as i32 {
            return Err(format!("active slot is {}, the headset has {SLOTS}", self.active));
        }
        if self.sidetone > 255 {
            return Err(format!("sidetone is {}, must be 0 to 255", self.sidetone));
        }

        let mut seen = std::collections::HashSet::new();
        for slot in &self.slots {
            if slot.index < 0 || slot.index >= SLOTS as i32 {
                return Err(format!("slot index {} is outside 0 to {}", slot.index, SLOTS - 1));
            }
            if !seen.insert(slot.index) {
                return Err(format!("slot {} appears twice", slot.index));
            }
            if slot.bands.len() != BANDS {
                return Err(format!("slot {} has {} bands, need {BANDS}", slot.index, slot.bands.len()));
            }
            for &v in &slot.bands {
                if v < -127 || v > 127 {
                    return Err(format!(
                        "slot {} has band value {}, outside sign-magnitude range",
                        slot.index, v
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Default config file path.
///
/// Uses `$XDG_CONFIG_HOME/strop/profile.toml` or `~/.config/strop/profile.toml`
/// on all platforms (consistent with the legacy occam convention).
pub fn default_path() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        let mut p = PathBuf::from(dir);
        p.push("strop");
        p.push("profile.toml");
        return Ok(p);
    }
    if let Some(home) = home_dir() {
        let mut p = PathBuf::from(home);
        p.push(".config");
        p.push("strop");
        p.push("profile.toml");
        return Ok(p);
    }
    Err("cannot determine home directory".into())
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        // On macOS, std::env::var("HOME") is reliable.
        std::env::var("HOME").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

/// Load a profile from a TOML file.
///
/// Returns an error if the file doesn't exist or can't be parsed.
pub fn load(path: &std::path::Path) -> Result<Profile, String> {
    let body = std::fs::read_to_string(path)
        .map_err(|e| format!("reading profile: {e}"))?;
    let p: Profile = toml::from_str(&body)
        .map_err(|e| format!("parsing profile: {e}"))?;
    p.validate()?;
    Ok(p)
}

/// Save a profile to a TOML file, creating the directory if needed.
pub fn save(path: &std::path::Path, profile: &Profile) -> Result<(), String> {
    profile.validate()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("creating directory: {e}"))?;
    }
    let mut header = String::from("# strop profile\n");
    header.push_str("# re-applied whenever the dongle reconnects\n\n");
    let body = toml::to_string(profile)
        .map_err(|e| format!("serializing profile: {e}"))?;
    std::fs::write(path, header + &body)
        .map_err(|e| format!("writing profile: {e}"))?;
    Ok(())
}

/// Convert a Slot's bands to an `Eq` (wire format).
pub fn slot_eq(slot: &Slot) -> Result<crate::Eq, String> {
    if slot.bands.len() != BANDS {
        return Err(format!("slot {} has {} bands, need {BANDS}", slot.index, slot.bands.len()));
    }
    let mut arr = [0i8; BANDS];
    for (i, &v) in slot.bands.iter().enumerate() {
        arr[i] = v as i8;
    }
    Ok(crate::Eq(arr))
}

/// Build a Slot from an `Eq` curve and name.
pub fn slot_from_eq(index: i32, name: Option<String>, eq: crate::Eq) -> Slot {
    let bands = eq.0.iter().map(|&v| v as i32).collect();
    Slot { index, name, bands }
}

// Re-export Eq from strop-proto so profile crate users don't need both.
pub use strop_proto::Eq;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn profile_path(tmp: &TempDir) -> PathBuf {
        tmp.path().join("profile.toml")
    }

    #[test]
    fn round_trip() {
        let tmp = TempDir::new().unwrap();
        let path = profile_path(&tmp);

        let mut want = Profile::new();
        want.active = 3;
        want.sidetone = 40;
        want.slots = vec![
            slot_from_eq(3, Some("game".into()), Eq([2, 2, 5, 5, 1, -1, 2, 3, 3, 3])),
            slot_from_eq(7, None, Eq([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
        ];

        save(&path, &want).expect("save");
        let got = load(&path).expect("load");

        assert_eq!(got.active, want.active);
        assert_eq!(got.sidetone, want.sidetone);
        assert_eq!(got.slots.len(), want.slots.len());
        assert_eq!(got.slots[0].name, Some("game".into()));
        assert_eq!(got.slots[0].bands, vec![2, 2, 5, 5, 1, -1, 2, 3, 3, 3]);
    }

    /// An absent key must not read as zero: an unspecified active slot
    /// silently selecting slot 0 on every apply would be a nasty surprise.
    #[test]
    fn absent_keys_stay_unset() {
        let tmp = TempDir::new().unwrap();
        let path = profile_path(&tmp);
        let body = "[[slot]]\nindex = 2\nbands = [0,0,0,0,0,0,0,0,0,0]\n";
        std::fs::write(&path, body).expect("write minimal profile");

        let p = load(&path).expect("load");
        assert_eq!(p.active, -1, "active should be -1 with no key present");
        assert_eq!(p.sidetone, -1, "sidetone should be -1 with no key present");
    }

    #[test]
    fn validate_rejects_invalid() {
        let ten = vec![0i32; 10];
        let cases: Vec<(&str, Profile)> = vec![
            ("active out of range", Profile { active: SLOTS as i32, sidetone: -1, slots: vec![] }),
            ("sidetone too large", Profile { active: -1, sidetone: 300, slots: vec![] }),
            ("slot index negative", Profile { active: -1, sidetone: -1, slots: vec![Slot { index: -1, name: None, bands: ten.clone() }] }),
            ("slot index too high", Profile { active: -1, sidetone: -1, slots: vec![Slot { index: SLOTS as i32, name: None, bands: ten.clone() }] }),
            ("duplicate slot", Profile { active: -1, sidetone: -1, slots: vec![
                Slot { index: 1, name: None, bands: ten.clone() },
                Slot { index: 1, name: None, bands: ten.clone() },
            ]}),
            ("wrong band count", Profile { active: -1, sidetone: -1, slots: vec![
                Slot { index: 1, name: None, bands: vec![0, 0, 0] },
            ]}),
            ("band out of range", Profile { active: -1, sidetone: -1, slots: vec![
                Slot { index: 1, name: None, bands: vec![200, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
            ]}),
        ];
        for (name, p) in cases {
            let result = p.validate();
            assert!(result.is_err(), "{name}: validated without error");
        }
    }

    #[test]
    fn validate_accepts_empty() {
        let p = Profile::new();
        assert!(p.validate().is_ok(), "empty profile should be valid");
    }

    #[test]
    fn default_path_honours_xdg() {
        // Temporarily set XDG_CONFIG_HOME and check the path
        let key = "XDG_CONFIG_HOME";
        std::env::set_var(key, "/tmp/xdg");
        let got = default_path().expect("default path");
        assert_eq!(got, PathBuf::from("/tmp/xdg/strop/profile.toml"));
        std::env::remove_var(key);
    }

    #[test]
    fn default_path_falls_back_to_home_config() {
        let key = "XDG_CONFIG_HOME";
        std::env::remove_var(key);
        let got = default_path().expect("default path");
        let s = got.to_string_lossy();
        assert!(s.contains(".config/strop/profile.toml"), "unexpected path: {s}");
    }

    #[test]
    fn save_creates_directory() {
        let tmp = TempDir::new().unwrap();
        let deep = tmp.path().join("a").join("b").join("c").join("profile.toml");
        let p = Profile::new();
        save(&deep, &p).expect("save should create directories");
        assert!(deep.exists(), "file should exist after save");
    }

    #[test]
    fn load_nonexistent_returns_error() {
        let result = load(std::path::Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn load_invalid_toml_returns_error() {
        let tmp = TempDir::new().unwrap();
        let path = profile_path(&tmp);
        std::fs::write(&path, "not valid toml [[[").expect("write");
        let result = load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn slot_eq_conversion() {
        let slot = Slot {
            index: 1,
            name: None,
            bands: vec![2, 2, 5, 5, 1, -1, 2, 3, 3, 3],
        };
        let eq = slot_eq(&slot).expect("convert");
        assert_eq!(eq.0[0], 2);
        assert_eq!(eq.0[5], -1);
    }

    #[test]
    fn slot_from_eq_conversion() {
        let eq = Eq([1, 2, 3, 0, -1, 0, 0, 0, 0, 0]);
        let slot = slot_from_eq(5, Some("test".into()), eq);
        assert_eq!(slot.index, 5);
        assert_eq!(slot.name, Some("test".into()));
        assert_eq!(slot.bands, vec![1, 2, 3, 0, -1, 0, 0, 0, 0, 0]);
    }
}
