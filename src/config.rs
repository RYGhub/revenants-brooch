//! This module is about fetching configuration values and parsing them appropriately.

use std::str::FromStr;

/// Get the ID of the Dota guild to follow from the `FOLLOWED_GUILD_ID` envvar.
pub fn followed_guild_id() -> i64 {
    let value = std::env::var("FOLLOWED_GUILD_ID").expect("Missing FOLLOWED_GUILD_ID envvar");
    let value = i64::from_str(&value).expect("Failed to parse FOLLOWED_GUILD_ID envvar");
    value
}

/// Get the [Stratz API key](https://stratz.com/api) from the `STRATZ_JWT` envvar.
pub fn stratz_jwt() -> String {
    let value = std::env::var("STRATZ_JWT").expect("Missing STRATZ_JWT envvar");
    value
}

/// Get the Discord webhook URL from the `DISCORD_WEBHOOK_URL` envvar.
pub fn discord_webhook_url() -> String {
    let value = std::env::var("DISCORD_WEBHOOK_URL").expect("Missing DISCORD_WEBHOOK_URL envvar");
    value
}