//! Versioned, strict, dependency-free RGB effect profile shared by CLI, user
//! daemon and desktop GUI. Only settings (never keystrokes) are stored.
use crate::effects::{Config, Preset};
use crate::{rgb_from_hex, Rgb};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Profile {
    pub preset: Preset,
    pub config: Config,
    pub fps: u32,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            preset: Preset::Ripple,
            config: Config::default(),
            fps: 24,
        }
    }
}
pub fn color_hex(rgb: Rgb) -> String {
    format!("{:02x}{:02x}{:02x}", rgb.red, rgb.green, rgb.blue)
}

impl Profile {
    pub fn validate(&self) -> Result<(), String> {
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
                _ => return Err(format!("Unknown profile field {name}")),
            }
        }
        if version != Some(1) { return Err("Profile requires version=1".into()); }
        result.validate()?;
        Ok(result)
    }

    pub fn encode(&self) -> Result<String, String> {
        self.validate()?;
        Ok(format!(
            "# Apex Pro Mini native RGB profile. No input history is saved.\nversion=1\neffect={}\ncolor={}\naccent={}\nspeed={:.3}\nbrightness={:.3}\nfps={}\n",
            self.preset.as_str(),
            color_hex(self.config.base),
            color_hex(self.config.accent),
            self.config.speed,
            self.config.brightness,
            self.fps
        ))
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
        }
    }
    #[test]
    fn reject_unknown_duplicate_and_unbounded_settings() {
        assert!(Profile::decode("version=1\neffect=unknown\n").is_err());
        assert!(Profile::decode("version=1\nfps=41\n").is_err());
        assert!(Profile::decode("version=1\nspeed=NaN\n").is_err());
        assert!(Profile::decode("version=1\nfps=24\nfps=5\n").is_err());
        assert!(Profile::decode("version=1\ncolor=xyz\n").is_err());
        assert!(Profile::decode("version=2\n").is_err());
    }
}
