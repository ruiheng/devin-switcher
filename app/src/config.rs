use std::fs;

use crate::paths;

/// Read devin.org_id from the CLI's config.json, if present.
pub fn current_org_id() -> Option<String> {
    let v: serde_json::Value =
        serde_json::from_slice(&fs::read(paths::config_path()).ok()?).ok()?;
    v.get("devin")?
        .get("org_id")?
        .as_str()
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

/// Point config.json's devin.org_id at the org captured for the profile.
/// `None` removes the key — leaving a previous account's org behind would be
/// worse than leaving the key unset.
pub fn apply_org_id(org_id: Option<&str>) -> Result<(), String> {
    let path = paths::config_path();
    let mut v: serde_json::Value = match fs::read(&path) {
        Ok(b) => serde_json::from_slice(&b).map_err(|e| format!("parse config.json: {e}"))?,
        Err(_) => serde_json::json!({}),
    };
    if !v.is_object() {
        return Err("config.json is not a JSON object".into());
    }
    let devin = v
        .as_object_mut()
        .unwrap()
        .entry("devin")
        .or_insert_with(|| serde_json::json!({}));
    if !devin.is_object() {
        *devin = serde_json::json!({});
    }
    let devin = devin.as_object_mut().unwrap();
    match org_id {
        Some(id) if !id.is_empty() => {
            devin.insert("org_id".into(), serde_json::json!(id));
        }
        _ => {
            devin.remove("org_id");
            if devin.is_empty() {
                v.as_object_mut().unwrap().remove("devin");
            }
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    crate::store::write_atomic(
        &path,
        serde_json::to_string_pretty(&v)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
}
