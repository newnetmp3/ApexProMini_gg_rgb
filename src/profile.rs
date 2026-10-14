//! Versioned, strict, dependency-free RGB effect profile shared by CLI, user
//! daemon and desktop GUI. Only settings (never keystrokes) are stored.
use crate::effects::{Config, Preset};
use crate::{rgb_from_hex, Rgb, ADDRESSABLE_HID_CODES};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Profile {
    pub preset: Preset,
    pub config: Config,
    pub fps: u32,
    pub key_overrides: BTreeMap<u8, Rgb>,
    pub idle: Option<Preset>,
    pub reactive: Option<Preset>,
    pub idle_after_seconds: f32,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            preset: Preset::Ripple,
            config: Config::default(),
            fps: 24,
            key_overrides: BTreeMap::new(),
            idle: None,
            reactive: None,
            idle_after_seconds: 15.0,
        }
    }
}
pub fn color_hex(rgb: Rgb) -> String {
    format!("{:02x}{:02x}{:02x}", rgb.red, rgb.green, rgb.blue)
}

impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        if !self.idle_after_seconds.is_finite()
            || !(1.0..=3600.0).contains(&self.idle_after_seconds) {
            return Err("Idle timeout must be finite and between 1 and 3600 seconds".into());
        }
        if self.idle.is_some_and(Preset::needs_key_events) {
            return Err("Idle effect must not require keypress input".into());
        }
        if self.reactive.is_some_and(|p| !p.needs_key_events()) {
            return Err("Reactive overlay must be a keypress-driven effect".into());
        }
        for &hid in self.key_overrides.keys() {
            if !ADDRESSABLE_HID_CODES.contains(&hid) {
                return Err(format!("Invalid keyboard key override: HID {hid}"));
            }
        }
        if !(5..=40).contains(&self.fps) {
            return Err("FPS must be between 5 and 40".into());
        }
        if !self.config.speed.is_finite()
            || !(0.1..=5.0).contains(&self.config.speed)
        {
            return Err("Speed must be a finite number between 0.1 and 5.0".into());
        }
        if !self.config.brightness.is_finite()
            || !(0.0..=1.0).contains(&self.config.brightness)
        {
            return Err("Brightness must be a finite number between 0 and 1".into());
        }
        Ok(())
    }

    pub fn decode(raw: &str) -> Result<Self, String> {
        let mut result = Self::default();
        let mut version = None::<u8>;
        let mut seen = std::collections::HashSet::new();
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let (name, value) = line.split_once('=').ok_or("Profile line must use key=value")?;
            let name = name.trim();
            let value = value.trim();
            if !seen.insert(name.to_string()) { return Err(format!("Duplicate profile field {name}")); }
            match name {
                "version" => version = Some(value.parse::<u8>().map_err(|_| "Bad profile version")?),
                "effect" => result.preset = Preset::from_name(value).ok_or_else(|| format!("Unknown effect {value}"))?,
                "color" => result.config.base = rgb_from_hex(value)?,
                "accent" => result.config.accent = rgb_from_hex(value)?,
                "speed" => result.config.speed = value.parse::<f32>().map_err(|_| "Invalid speed")?,
                "brightness" => result.config.brightness = value.parse::<f32>().map_err(|_| "Invalid brightness")?,
                "fps" => result.fps = value.parse::<u32>().map_err(|_| "Invalid frame rate")?,
                "idle" => result.idle = if value.eq_ignore_ascii_case("none") { None }
                    else { Some(Preset::from_name(value).ok_or("Unknown idle effect")?) },
                "reactive" => result.reactive = if value.eq_ignore_ascii_case("none") { None }
                    else { Some(Preset::from_name(value).ok_or("Unknown reactive effect")?) },
                "idle_seconds" => result.idle_after_seconds = value.parse::<f32>().map_err(|_| "Invalid idle timeout")?,
                key if key.starts_with("key.") => {
                    let hid: u8 = key[4..].parse().map_err(|_| "Invalid HID key override number")?;
                    if !ADDRESSABLE_HID_CODES.contains(&hid) {
                        return Err(format!("Unknown keyboard HID usage {hid}"));
                    }
                    result.key_overrides.insert(hid, rgb_from_hex(value)?);
                }
                _ => return Err(format!("Unknown profile field {name}")),
            }
        }
        if version != Some(1) { return Err("Profile requires version=1".into()); }
        result.validate()?;
        Ok(result)
    }

    pub fn encode(&self) -> Result<String, String> {
        self.validate()?;
        let mut text = format!(
            "# Apex Pro Mini native RGB profile. No input history is saved.\nversion=1\neffect={}\ncolor={}\naccent={}\nspeed={:.3}\nbrightness={:.3}\nfps={}\n",
            self.preset.as_str(),
            color_hex(self.config.base),
            color_hex(self.config.accent),
            self.config.speed,
            self.config.brightness,
            self.fps
        );
        text.push_str(&format!(
            "idle={}\nreactive={}\nidle_seconds={:.3}\n",
            self.idle.map(Preset::as_str).unwrap_or("none"),
            self.reactive.map(Preset::as_str).unwrap_or("none"),
            self.idle_after_seconds
        ));
        for (hid, color) in &self.key_overrides {
            text.push_str(&format!("key.{hid}={}\n", color_hex(*color)));
        }
        Ok(text)
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        Self::decode(&data)
    }

    /// Atomic replacement prevents a running daemon from observing a partial profile.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let data = self.encode()?;
        let parent = path.parent().ok_or("Profile has no parent directory")?;
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        let mut temp = path.to_path_buf();
        temp.set_extension(format!("tmp.{}", std::process::id()));
        let write = (|| -> io::Result<()> {
            use std::io::Write;
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
            file.write_all(data.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temp, path)?;
            Ok(())
        })();
        if let Err(err) = write {
            let _ = fs::remove_file(&temp);
            return Err(format!("Could not save profile {}: {err}", path.display()));
        }
        Ok(())
    }
}

pub fn default_profile_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME not set")?;
    Ok(PathBuf::from(home).join(".config/apex-mini-rgb/profile.conf"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_roundtrip_all_modes() {
        for effect in crate::effects::PRESET_NAMES.split(", ") {
            let p = Profile { preset: Preset::from_name(effect).unwrap(), ..Profile::default() };
            let next = Profile::decode(&p.encode().unwrap()).unwrap();
            assert_eq!(next.preset, p.preset);
            assert_eq!(next.fps, p.fps);
            assert_eq!(next.config.base, p.config.base);
            assert_eq!(next.key_overrides, p.key_overrides);
            assert_eq!(next.idle, p.idle);
            assert_eq!(next.reactive, p.reactive);
        }
    }
    #[test]
    fn per_key_color_override_roundtrip() {
        let mut p = Profile::default();
        p.key_overrides.insert(4, Rgb { red: 255, green: 20, blue: 5 });
        let parsed = Profile::decode(&p.encode().unwrap()).unwrap();
        assert_eq!(parsed.key_overrides.get(&4), p.key_overrides.get(&4));
    }
    #[test]
    fn layered_profile_roundtrip_preserves_selectors() {
        let p = Profile {
            preset: Preset::from_name("gg:aqua").unwrap(),
            idle: Some(Preset::from_name("gg:steelseriesOrange").unwrap()),
            reactive: Some(Preset::from_name("gg:reactiveRipple").unwrap()),
            idle_after_seconds: 19.5, ..Profile::default()
        };
        let decoded = Profile::decode(&p.encode().unwrap()).unwrap();
        assert_eq!(decoded.preset, p.preset);
        assert_eq!(decoded.idle, p.idle);
        assert_eq!(decoded.reactive, p.reactive);
        assert_eq!(decoded.idle_after_seconds, p.idle_after_seconds);
    }
    #[test]
    fn old_version_one_profiles_remain_compatible() {
        let old = Profile::decode("version=1\neffect=ripple\ncolor=081020\naccent=ff5500\nspeed=1.0\nbrightness=0.75\nfps=24\n").unwrap();
        assert!(old.idle.is_none());
        assert!(old.reactive.is_none());
        assert_eq!(old.idle_after_seconds, 15.0);
    }
    #[test]
    fn reject_unknown_duplicate_and_unbounded_settings() {
        assert!(Profile::decode("version=1\neffect=unknown\n").is_err());
        assert!(Profile::decode("version=1\nfps=41\n").is_err());
        assert!(Profile::decode("version=1\nspeed=NaN\n").is_err());
        assert!(Profile::decode("version=1\nfps=24\nfps=5\n").is_err());
        assert!(Profile::decode("version=1\ncolor=xyz\n").is_err());
        assert!(Profile::decode("version=2\n").is_err());
        assert!(Profile::decode("version=1\nkey.0=ffffff\n").is_err());
        assert!(Profile::decode("version=1\nidle=gg:reactiveRipple\n").is_err());
        assert!(Profile::decode("version=1\nreactive=gg:prism\n").is_err());
    }
}
