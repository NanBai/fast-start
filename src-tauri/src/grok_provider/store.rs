use super::profile::GrokProfile;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub struct ProfileStore {
    path: PathBuf,
}

impl ProfileStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn list(&self) -> Result<Vec<GrokProfile>, String> {
        self.ensure_dir()?;
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        if data.trim().is_empty() {
            return Ok(Vec::new());
        }
        let profiles: Vec<GrokProfile> =
            serde_json::from_str(&data).map_err(|e| format!("解析 profiles.json 失败: {e}"))?;
        Ok(profiles)
    }

    pub fn get(&self, id: &str) -> Result<GrokProfile, String> {
        self.list()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| "供应商不存在".to_string())
    }

    pub fn create(&self, mut profile: GrokProfile) -> Result<GrokProfile, String> {
        let mut profiles = self.list()?;
        let now = Utc::now();
        if profile.id.is_empty() {
            profile.id = uuid::Uuid::new_v4().to_string();
        }
        if profile.created_at.is_none() {
            profile.created_at = Some(now);
        }
        profile.updated_at = Some(now);
        profile = profile.normalize();
        if profiles.iter().any(|p| p.id == profile.id) {
            return Err("供应商 id 已存在".to_string());
        }
        profiles.push(profile.clone());
        self.write(&profiles)?;
        Ok(profile)
    }

    pub fn update(&self, id: &str, mut next: GrokProfile) -> Result<GrokProfile, String> {
        let mut profiles = self.list()?;
        let Some(idx) = profiles.iter().position(|p| p.id == id) else {
            return Err("供应商不存在".to_string());
        };
        next.id = id.to_string();
        next.created_at = profiles[idx].created_at;
        next.is_active = profiles[idx].is_active;
        next.updated_at = Some(Utc::now());
        next = next.normalize();
        profiles[idx] = next.clone();
        self.write(&profiles)?;
        Ok(next)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let mut profiles = self.list()?;
        let before = profiles.len();
        profiles.retain(|p| p.id != id);
        if profiles.len() == before {
            return Err("供应商不存在".to_string());
        }
        self.write(&profiles)
    }

    pub fn set_active(&self, id: &str) -> Result<(), String> {
        let mut profiles = self.list()?;
        let mut found = false;
        let now = Utc::now();
        for p in &mut profiles {
            let active = p.id == id;
            if active {
                found = true;
                p.updated_at = Some(now);
            }
            p.is_active = active;
        }
        if !found {
            return Err("供应商不存在".to_string());
        }
        self.write(&profiles)
    }

    fn ensure_dir(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn write(&self, profiles: &[GrokProfile]) -> Result<(), String> {
        self.ensure_dir()?;
        let data = serde_json::to_string_pretty(profiles).map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, data).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &self.path).map_err(|e| e.to_string())
    }
}
