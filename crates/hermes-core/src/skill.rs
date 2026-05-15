
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub version: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<String>,
}

pub struct SkillStore {
    skills: HashMap<String, Skill>,
    skills_dir: String,
}

impl SkillStore {
    pub fn new<P: AsRef<Path>>(skills_dir: P) -> Self {
        SkillStore {
            skills: HashMap::new(),
            skills_dir: skills_dir.as_ref().to_str().unwrap().into(),
        }
    }
    
    pub fn add(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }
    
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }
    
    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }
    
    pub fn delete(&mut self, name: &str) -> bool {
        self.skills.remove(name).is_some()
    }
    
    pub async fn load_all(&mut self) {
        let skills_path = Path::new(&self.skills_dir);
        if !skills_path.exists() {
            if let Err(e) = fs::create_dir_all(skills_path).await {
                eprintln!("Failed to create skills dir: {}", e);
            }
            return;
        }
        
        let mut dir = match fs::read_dir(skills_path).await {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to read skills dir: {}", e);
                return;
            }
        };
        
        while let Some(entry) = dir.next_entry().await.ok().flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Ok(skill) = toml::from_str::<Skill>(&content) {
                        self.skills.insert(skill.name.clone(), skill);
                    }
                }
            }
        }
    }
    
    pub async fn save(&self, name: &str) {
        if let Some(skill) = self.skills.get(name) {
            let path = Path::new(&self.skills_dir).join(format!("{}.toml", name));
            if let Ok(content) = toml::to_string_pretty(skill) {
                let _ = fs::write(path, content).await;
            }
        }
    }
}
