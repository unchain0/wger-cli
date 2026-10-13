//! wger REST API client, shared by the CLI and the MCP server.
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};

pub mod mcp;

/// Resources the tool is allowed to touch (path segment under /api/v2/).
pub const RESOURCES: &[&str] = &[
    "routine",
    "day",
    "slot",
    "slot-entry",
    "workoutlog",
    "workoutsession",
    "sets-config",
    "max-sets-config",
    "repetitions-config",
    "max-repetitions-config",
    "weight-config",
    "max-weight-config",
    "rir-config",
    "max-rir-config",
    "rest-config",
    "max-rest-config",
    "exercise",
    "exerciseinfo",
    "exercise-translation",
    "language",
    "muscle",
    "equipment",
    "exercisecategory",
];

pub struct Client {
    base: String,
    token: Option<String>,
    agent: ureq::Agent,
}

/// Ids are integers for most resources and UUIDs for workoutlog/workoutsession.
pub fn check_id(id: &str) -> Result<()> {
    if !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        Ok(())
    } else {
        bail!("invalid id '{id}'")
    }
}

fn check_resource(r: &str) -> Result<()> {
    if RESOURCES.contains(&r) {
        Ok(())
    } else {
        bail!("unknown resource '{r}'. Allowed: {}", RESOURCES.join(", "))
    }
}

impl Client {
    pub fn new(base: &str, token: Option<String>) -> Self {
        let token = token.filter(|t| !t.trim().is_empty());
        Client {
            base: base.trim_end_matches('/').to_string(),
            token,
            agent: ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("wger-cli/0.1")
                .build(),
        }
    }

    pub fn from_env() -> Self {
        let base = std::env::var("WGER_BASE_URL").unwrap_or_else(|_| "https://wger.de".into());
        Self::new(&base, std::env::var("WGER_API_TOKEN").ok())
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}/api/v2/{}", self.base, path.trim_start_matches('/'))
        }
    }

    fn send(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
    ) -> Result<Value> {
        let mut req = self
            .agent
            .request(method, &self.url(path))
            .set("Accept", "application/json");
        for (k, v) in query {
            req = req.query(k, v);
        }
        if let Some(t) = &self.token {
            req = req.set("Authorization", &format!("Token {t}"));
        } else if method != "GET" {
            bail!("WGER_API_TOKEN is not set; write operations need a wger API token");
        }
        let res = match body {
            Some(b) => req.send_json(b.clone()),
            None => req.call(),
        };
        match res {
            Ok(r) => {
                if r.status() == 204 {
                    return Ok(json!({"ok": true}));
                }
                let text = r.into_string()?;
                if text.trim().is_empty() {
                    return Ok(json!({"ok": true}));
                }
                serde_json::from_str(&text).context("invalid JSON from wger")
            }
            Err(ureq::Error::Status(code, r)) => {
                let text = r.into_string().unwrap_or_default();
                Err(anyhow!(
                    "wger API error {code} on {method} {path}: {}",
                    text.chars().take(2000).collect::<String>()
                ))
            }
            Err(e) => Err(anyhow!("request failed: {e}")),
        }
    }

    /// List a resource. With `all`, follows `next` links (max 50 pages).
    pub fn list(&self, resource: &str, query: &[(String, String)], all: bool) -> Result<Value> {
        check_resource(resource)?;
        let first = self.send("GET", &format!("{resource}/"), query, None)?;
        if !all {
            return Ok(first);
        }
        let mut results: Vec<Value> = first["results"].as_array().cloned().unwrap_or_default();
        let mut next = first["next"].as_str().map(String::from);
        let mut pages = 1;
        while let Some(n) = next {
            if pages >= 50 {
                break;
            }
            let page = self.send("GET", &n, &[], None)?;
            results.extend(page["results"].as_array().cloned().unwrap_or_default());
            next = page["next"].as_str().map(String::from);
            pages += 1;
        }
        Ok(json!({"count": results.len(), "results": results}))
    }

    pub fn get(&self, resource: &str, id: &str) -> Result<Value> {
        check_resource(resource)?;
        check_id(id)?;
        self.send("GET", &format!("{resource}/{id}/"), &[], None)
    }

    pub fn create(&self, resource: &str, data: &Value) -> Result<Value> {
        check_resource(resource)?;
        self.send("POST", &format!("{resource}/"), &[], Some(data))
    }

    pub fn update(&self, resource: &str, id: &str, data: &Value) -> Result<Value> {
        check_resource(resource)?;
        check_id(id)?;
        self.send("PATCH", &format!("{resource}/{id}/"), &[], Some(data))
    }

    pub fn delete(&self, resource: &str, id: &str, confirmed: bool) -> Result<Value> {
        check_resource(resource)?;
        check_id(id)?;
        if !confirmed {
            bail!("refusing to delete {resource}/{id} without confirmation (--yes / confirm=true)");
        }
        self.send("DELETE", &format!("{resource}/{id}/"), &[], None)
    }

    /// Routine sub-views: structure, logs, stats, date-sequence-display.
    pub fn routine_view(&self, id: u64, view: &str) -> Result<Value> {
        if ![
            "structure",
            "logs",
            "stats",
            "date-sequence-display",
            "date-sequence-gym",
        ]
        .contains(&view)
        {
            bail!("unknown routine view '{view}'");
        }
        self.send("GET", &format!("routine/{id}/{view}/"), &[], None)
    }

    /// Resolve a language code ("pt") or numeric id to the wger language id.
    pub fn language_id(&self, lang: &str) -> Result<u64> {
        if let Ok(n) = lang.parse::<u64>() {
            return Ok(n);
        }
        let r = self.send(
            "GET",
            "language/",
            &[("short_name".into(), lang.into())],
            None,
        )?;
        r["results"]
            .as_array()
            .and_then(|a| a.iter().find(|l| l["short_name"] == lang))
            .and_then(|l| l["id"].as_u64())
            .ok_or_else(|| anyhow!("unknown language '{lang}'"))
    }

    /// Create or update the translation of an exercise in a language.
    pub fn translation_set(
        &self,
        exercise: u64,
        lang: &str,
        name: Option<&str>,
        description: Option<&str>,
        author: Option<&str>,
    ) -> Result<Value> {
        let lang_id = self.language_id(lang)?;
        let q = vec![
            ("exercise".to_string(), exercise.to_string()),
            ("language".to_string(), lang_id.to_string()),
        ];
        let existing = self.send("GET", "exercise-translation/", &q, None)?;
        let mut data = serde_json::Map::new();
        if let Some(n) = name {
            data.insert("name".into(), json!(n));
        }
        if let Some(d) = description {
            data.insert("description".into(), json!(d));
        }
        if let Some(a) = author {
            data.insert("license_author".into(), json!(a));
        }
        // The API may ignore the language filter, so match the language explicitly.
        let found = existing["results"]
            .as_array()
            .and_then(|a| {
                a.iter().find(|t| {
                    t["language"].as_u64() == Some(lang_id)
                        && t["exercise"].as_u64() == Some(exercise)
                })
            })
            .and_then(|t| t["id"].as_u64());
        if let Some(id) = found {
            if data.is_empty() {
                bail!("nothing to update: pass name and/or description");
            }
            let out = self.send(
                "PATCH",
                &format!("exercise-translation/{id}/"),
                &[],
                Some(&Value::Object(data)),
            )?;
            Ok(json!({"action": "updated", "translation": out}))
        } else {
            if name.is_none() || description.is_none() {
                bail!("creating a new translation needs both name and description");
            }
            data.insert("exercise".into(), json!(exercise));
            data.insert("language".into(), json!(lang_id));
            let out = self.send(
                "POST",
                "exercise-translation/",
                &[],
                Some(&Value::Object(data)),
            )?;
            Ok(json!({"action": "created", "translation": out}))
        }
    }
}

pub fn parse_query(pairs: &[String]) -> Result<Vec<(String, String)>> {
    pairs
        .iter()
        .map(|p| {
            p.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .ok_or_else(|| anyhow!("bad query '{p}', expected key=value"))
        })
        .collect()
}
