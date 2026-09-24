//! `youtube accounts add|list|test|remove`. Manages stored Apify accounts and keys.

use std::io::{BufRead, IsTerminal, Write};

use serde_json::Value;

use crate::account::{self, identity};
use crate::cli::{AccountsArgs, AccountsSubcommand};
use crate::client::Client;
use crate::config::{self, AccountConfig};
use crate::error::{Error, ErrorCode, Result};
use crate::obj;
use crate::output;
use crate::secrets::{self, Store};

pub fn run(args: AccountsArgs) -> Result<()> {
    match args.command {
        AccountsSubcommand::Add { name, api_key, api_key_stdin, force, no_verify } => {
            let api_key = if api_key_stdin { Some(read_stdin_key()?) } else { api_key };
            add(name, api_key, force, no_verify)
        }
        AccountsSubcommand::List { check } => list(check),
        AccountsSubcommand::Test { name } => test(&name),
        AccountsSubcommand::Remove { name, yes } => remove(&name, yes),
    }
}

fn add(name: String, api_key: Option<String>, force: bool, no_verify: bool) -> Result<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(Error::invalid("An account name is required."));
    }

    let store = secrets::store()?;
    let config = config::load()?;

    let existing = config.find(&name).map(|(k, _)| k.clone());
    if let Some(existing) = &existing
        && !force
    {
        return Err(Error::invalid(format!("An account named '{existing}' already exists.")).fix(format!(
            "Pick a different name, or replace its key: youtube accounts add {existing} --api-key <key> --force"
        )));
    }
    let name = existing.clone().unwrap_or(name);

    let key = match api_key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty()) {
        Some(k) => k,
        None => prompt_key(&name)?,
    };

    let (mut ident, mut org) = (String::new(), String::new());
    if !no_verify {
        let me = Client::new(&key).get("users/me")?;
        ident = identity::describe(&me);
        org = identity::organization(&me);
    }

    {
        let _lock = config::lock()?;
        store.set(&secrets::account_key(&name), &key)?;

        let mut config = config::load()?;
        config.accounts.insert(
            name.clone(),
            AccountConfig { identity: ident.clone(), organization: org.clone(), added_at: config::now_utc() },
        );
        config::save(&config)?;
    }

    output::write(&obj! {
        "status" => if existing.is_none() { "added" } else { "replaced" },
        "name" => name,
        "identity" => ident,
        "organization" => org,
        "verified" => !no_verify,
        "secretStore" => store.name(),
        "configDir" => config::config_dir().display().to_string(),
        "nextStep" => format!("youtube search \"rust tutorial\" --account {name}"),
    });

    Ok(())
}

fn read_stdin_key() -> Result<String> {
    let mut key = String::new();
    std::io::stdin().lock().read_line(&mut key).map_err(|e| Error::invalid(format!("Could not read stdin: {e}")))?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(Error::invalid("--api-key-stdin was given but stdin was empty."));
    }
    Ok(key)
}

fn prompt_key(name: &str) -> Result<String> {
    if !std::io::stdin().is_terminal() {
        return Err(Error::invalid("No API key given and no terminal to prompt on.")
            .fix(format!("pbpaste | youtube accounts add {name} --api-key-stdin")));
    }
    loop {
        let key = rpassword::prompt_password(format!("Apify API token for {name}: "))
            .map_err(|e| Error::other("Could not read the API key.").detail(e.to_string()))?;
        let key = key.trim().to_string();
        if !key.is_empty() {
            return Ok(key);
        }
        eprintln!("Cannot be empty");
    }
}

fn list(check: bool) -> Result<()> {
    let store = secrets::store()?;
    let config = config::load()?;
    let sorted = config.sorted();

    let statuses: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> =
            sorted.iter().map(|(name, _acct)| scope.spawn(move || status_of(name, store, check))).collect();
        handles.into_iter().map(|h| h.join().unwrap_or_else(|_| "unreachable".into())).collect()
    });

    let accounts: Vec<Value> = sorted
        .iter()
        .zip(statuses)
        .map(|((name, a), status)| {
            obj! {
                "name" => name,
                "identity" => a.identity,
                "organization" => a.organization,
                "addedAt" => a.added_at,
                "keyStatus" => status,
            }
        })
        .collect();

    output::write(&obj! {
        "count" => accounts.len(),
        "accounts" => accounts,
        "secretStore" => store.name(),
        "configDir" => config::config_dir().display().to_string(),
    });

    Ok(())
}

fn status_of(name: &str, store: Store, check: bool) -> String {
    let key = match store.get(&secrets::account_key(name)) {
        Ok(Some(k)) if !k.trim().is_empty() => k,
        Ok(_) => return "missing_key".into(),
        Err(_) => return "unreadable".into(),
    };
    if !check {
        return "stored".into();
    }
    match Client::new(&key).get("users/me") {
        Ok(_) => "valid".into(),
        Err(e) if e.code == ErrorCode::AuthRequired => "rejected".into(),
        Err(_) => "unreachable".into(),
    }
}

fn test(name: &str) -> Result<()> {
    let account = account::resolve(Some(name), None)?;
    let me = account.client().get("users/me")?;
    let ident = identity::describe(&me);
    let org = identity::organization(&me);

    let mut result = obj! {
        "name" => account.name,
        "identity" => ident,
        "organization" => org,
        "keyStatus" => "valid",
        "me" => me,
    };

    if let Some(w) = drift(&account.config.identity, &ident).or_else(|| drift(&account.config.organization, &org)) {
        result["warning"] = Value::String(w);
    }

    output::write(&result);
    Ok(())
}

fn drift(recorded: &str, current: &str) -> Option<String> {
    (!recorded.trim().is_empty() && !current.trim().is_empty() && !current.eq_ignore_ascii_case(recorded))
        .then(|| format!("This account was added as {recorded}, but the stored key now reports {current}."))
}

fn remove(requested: &str, yes: bool) -> Result<()> {
    let store = secrets::store()?;
    let config = config::load()?;

    let Some((name, acct)) = config.find(requested) else {
        return Err(
            Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'.")).fix("youtube accounts list")
        );
    };
    let (name, acct) = (name.clone(), acct.clone());

    if !yes {
        if !std::io::stdin().is_terminal() {
            return Err(Error::invalid(format!(
                "Removing '{name}' needs confirmation and there is no terminal to ask on."
            ))
            .fix(format!("youtube accounts remove {name} --yes")));
        }
        let label = if acct.identity.trim().is_empty() { name.clone() } else { format!("{name} ({})", acct.identity) };
        eprint!("Remove account {label}? [y/N] ");
        let _ = std::io::stderr().flush();
        let mut answer = String::new();
        let _ = std::io::stdin().lock().read_line(&mut answer);
        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
            return Err(Error::invalid("Cancelled."));
        }
    }

    {
        let _lock = config::lock()?;
        store.delete(&secrets::account_key(&name))?;
        let mut config = config::load()?;
        config.accounts.shift_remove(&name);
        config::save(&config)?;
    }

    output::write(&obj! {
        "status" => "removed",
        "name" => name,
        "identity" => acct.identity,
        "note" => "The API key was deleted locally. Revoke it at https://console.apify.com/account/integrations if it should stop working everywhere.",
    });

    Ok(())
}
