//! Multi-account resolution and identity probing for Apify credentials.
//!
//! Keys can come from:
//! 1. `--api-key <key>` explicit flag.
//! 2. `--account <name>` (`-a <name>`) stored in the OS keystore.
//! 3. `APIFY_TOKEN` (or `YOUTUBE_API_KEY`) environment variable.

use serde_json::Value;

use crate::client::Client;
use crate::config::{self, AccountConfig, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct Resolved {
    pub name: String,
    pub config: AccountConfig,
    pub api_key: String,
}

impl Resolved {
    pub fn client(&self) -> Client {
        Client::new(&self.api_key)
    }
}

pub fn resolve(account: Option<&str>, explicit_key: Option<&str>) -> Result<Resolved> {
    if let Some(key) = explicit_key.map(str::trim).filter(|k| !k.is_empty()) {
        return Ok(Resolved { name: "direct".into(), config: AccountConfig::default(), api_key: key.to_string() });
    }

    if let Some(requested) = account.map(str::trim).filter(|s| !s.is_empty()) {
        let config = config::load()?;
        let Some((name, account_cfg)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'."))
                .detail(describe(&config))
                .fix("youtube accounts list"));
        };

        let key = secrets::store()?.get(&secrets::account_key(name))?;
        let Some(api_key) = key.filter(|k| !k.trim().is_empty()) else {
            return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{name}' has no stored API key."))
                .detail("The config entry exists but the keystore has nothing under it.")
                .fix(format!("youtube accounts add {name} --api-key <key>")));
        };

        return Ok(Resolved { name: name.clone(), config: account_cfg.clone(), api_key });
    }

    if let Some(token) = std::env::var("APIFY_TOKEN")
        .or_else(|_| std::env::var("YOUTUBE_API_KEY"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        return Ok(Resolved { name: "env".into(), config: AccountConfig::default(), api_key: token });
    }

    let config = config::load()?;
    Err(Error::new(
        ErrorCode::NoAccount,
        "No API key or account specified. Pass --account <name>, --api-key <key>, or set APIFY_TOKEN.",
    )
    .detail(describe(&config))
    .fix("youtube login"))
}

fn describe(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No accounts are configured yet. Run 'youtube accounts add <name> --api-key <key>' or 'youtube login'."
            .into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| if v.identity.trim().is_empty() { k.clone() } else { format!("{k} ({})", v.identity) })
        .collect();
    format!("Configured accounts: {}", listed.join(", "))
}

pub mod identity {
    use super::Value;

    const CANDIDATES: &[&str] = &["username", "email", "userEmail", "user_email", "name", "userName", "login", "id"];

    pub fn describe(me: &Value) -> String {
        let target = me.get("data").filter(|d| d.is_object()).unwrap_or(me);
        first_string(target, CANDIDATES).or_else(|| nested(target, "user", CANDIDATES)).unwrap_or_default()
    }

    pub fn organization(me: &Value) -> String {
        let target = me.get("data").filter(|d| d.is_object()).unwrap_or(me);
        if let Some(plan) = target.get("plan") {
            if let Some(name) = plan.get("name").and_then(Value::as_str) {
                return name.to_string();
            }
            if let Some(s) = plan.as_str() {
                return s.to_string();
            }
        }
        first_string(target, &["organization", "organizationId", "orgId"]).unwrap_or_default()
    }

    fn nested(root: &Value, property: &str, names: &[&str]) -> Option<String> {
        root.get(property).filter(|c| c.is_object()).and_then(|c| first_string(c, names))
    }

    fn first_string(v: &Value, names: &[&str]) -> Option<String> {
        names
            .iter()
            .filter_map(|n| v.get(*n).and_then(Value::as_str))
            .find(|s| !s.trim().is_empty())
            .map(str::to_string)
    }

    #[cfg(test)]
    mod tests {
        use serde_json::json;

        #[test]
        fn reads_apify_me_shape() {
            let me = json!({
                "data": {
                    "id": "u123",
                    "username": "tester",
                    "email": "tester@example.com",
                    "plan": {"name": "FREE"}
                }
            });
            assert_eq!(super::describe(&me), "tester");
            assert_eq!(super::organization(&me), "FREE");
        }
    }
}
