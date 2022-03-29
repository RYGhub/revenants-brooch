//! This module is about performing GraphQL requests to [STRATZ](https://stratz.com/api) to fetch data.

use crate::config;
use graphql_client::GraphQLQuery;

/// Binding for the `Long` type of the GraphQL schema to [i64].
type Short = i16;
type Long = i64;
type Byte = u8;

/// Query to fetch the 10 latest match of a specific guild.
#[derive(GraphQLQuery)]
#[graphql(schema_path="src/stratz_schema.gql", query_path="src/latest_guild_matches.gql", response_derives="Clone,Debug")]
struct MatchesQuery;
pub type Response = graphql_client::Response<matches_query::ResponseData>;
pub use matches_query::ResponseData;
pub use matches_query::LobbyTypeEnum as LobbyType;
pub use matches_query::GameModeEnumType as GameMode;
pub use matches_query::MatchesQueryGuild as Guild;
pub use matches_query::MatchesQueryGuildMatches as Match;
pub use matches_query::MatchesQueryGuildMatchesPlayers as Player;
pub use matches_query::MatchesQueryGuildMatchesPlayersHero as Hero;
pub use matches_query::MatchesQueryGuildMatchesPlayersSteamAccount as Steam;

/// Error enumeration for possible Stratz errors.
#[derive(Clone, Debug)]
pub enum StratzError {
    /// An error occurred while performing a request to Stratz.
    Request,
    /// The response of a request could not be deserialized.
    Parse,
}

/// Get the Stratz GraphQL API URL, with the JWT prefilled.
fn api_url() -> String {
    format!("https://api.stratz.com/graphql?jwt={}", &config::stratz_jwt())
}

/// Fetch the latest `take` matches of the guild having the specified `guild_id`.
pub async fn fetch_matches(client: reqwest::Client, guild_id: i64, take: i64) -> Result<Response, StratzError> {
    debug!("Fetching {take} matches of guild {guild_id}");

    trace!("Constructing variables object...");
    let vars = matches_query::Variables { guild_id, take };
    trace!("Building query...");
    let body = MatchesQuery::build_query(vars);
    trace!("Posting request...");
    let resp = client.post(api_url()).json(&body).send().await.map_err(|_| StratzError::Request)?;
    trace!("Parsing response...");
    let data = resp.json::<Response>().await.map_err(|_| StratzError::Parse)?;
    trace!("Successfully parsed response!");

    Ok(data)
}
