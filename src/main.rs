extern crate pretty_env_logger;
#[macro_use] extern crate log;

use crate::stratz::StratzError;

mod config;
mod stratz;

/// The period of time elapsed between two match scans.
const MATCH_SCAN_PERIOD: tokio::time::Duration = tokio::time::Duration::from_secs(60 * 30);

/// The amount of matches to request on every scan.
const MATCH_SCAN_TAKE: i64 = 10;

/// The minimum number of players that must be in a match for it to be announced.
const MATCH_ANNOUNCE_PLAYERS: usize = 1;

#[tokio::main]
async fn main() -> ! {
    pretty_env_logger::init();
    debug!("Logger initialized!");

    trace!("Entering main loop...");
    let mut current_match_id: i64 = -1;
    loop {
        trace!("Starting iteration of the main loop...");
        match match_scan(&mut current_match_id).await {
            Ok(()) => debug!("Completed match scan successfully!"),
            Err(e) => error!("Error in match scan: {:#?}", &e),
        }
        trace!("Sleeping in the main loop...");
        tokio::time::sleep(MATCH_SCAN_PERIOD).await;
    }
}


#[derive(Clone, Debug)]
enum RefreshError {
    Stratz(StratzError),
    Data,
    Discord,
}


async fn match_scan(current_match_id: &mut i64) -> Result<(), RefreshError> {
    debug!("Starting match scan...");

    trace!("Creating new reqwest client...");
    let http_client = reqwest::Client::new();
    trace!("Creating new webhook client...");
    let webhook_client = webhook::client::WebhookClient::new(&config::discord_webhook_url());
    trace!("Fetching matches...");
    let response = stratz::fetch_matches(http_client, config::followed_guild_id(), MATCH_SCAN_TAKE).await.map_err(|e| RefreshError::Stratz(e))?;

    trace!("Ensuring there are no errors in the data...");
    if let Some(errors) = response.errors {
        error!("Errors in STRATZ response: {:#?}", errors);
        return Err(RefreshError::Data);
    }

    trace!("Ensuring the data object exists...");
    let data: stratz::ResponseData = response.data.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the guild object exists...");
    let guild: stratz::Guild = data.guild.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the guild id exists...");
    let id: i64 = guild.id.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the guild name exists...");
    let name: String = guild.name.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the guild logo exists...");
    let logo: String = guild.logo.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the matches object exists...");
    let matches: Vec<Option<stratz::Match>> = guild.matches.ok_or_else(|| RefreshError::Data)?;
    trace!("Parsing matches from the last to the first...");
    for match_ in matches.into_iter().rev() {
        trace!("Ensuring the match object exists...");
        let match_ = match_.ok_or_else(|| RefreshError::Data)?;
        match_announce(current_match_id, &webhook_client, match_, &id, &name, &logo).await?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
enum MatchResult {
    None,
    Victory,
    Defeat,
    Both,
}

async fn match_announce(current_match_id: &mut i64, client: &webhook::client::WebhookClient, match_: stratz::Match, guild_id: &i64, guild_name: &str, guild_logo: &str) -> Result<(), RefreshError> {
    trace!("Ensuring the match ID exists...");
    let id: i64 = match_.id.ok_or_else(|| RefreshError::Data)?;

    trace!("Checking if the match should be announced...");
    if id <= *current_match_id {
        trace!("Skipping announcement of {id}, as it was already announced.");
        return Ok(())
    }
    trace!("Bumping current match id up to the current value...");
    *current_match_id = id;

    trace!("Ensuring the player list exists...");
    let players: Vec<Option<stratz::Player>> = match_.players.ok_or_else(|| RefreshError::Data)?;

    if players.len() < MATCH_ANNOUNCE_PLAYERS {
        trace!("Skipping announcement of {id}, as it was already announced.");
        return Ok(())
    }

    debug!("Announcing match {id}!");

    trace!("Ensuring the lobby type exists...");
    let lobby_type: stratz::LobbyType = match_.lobby_type.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the game mode exists...");
    let game_mode: stratz::GameMode = match_.game_mode.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the duration exists...");
    let duration = match_.duration_seconds.ok_or_else(|| RefreshError::Data)?;
    let duration: chrono::Duration = chrono::Duration::seconds(duration);
    trace!("Ensuring the end date time exists...");
    let end = match_.end_date_time.ok_or_else(|| RefreshError::Data)?;
    let end = i64::try_from(end).map_err(|_| RefreshError::Data)?;
    let end = chrono::NaiveDateTime::from_timestamp(end, 0);
    let end = chrono::DateTime::<chrono::Utc>::from_utc(end, chrono::Utc);

    trace!("Determining duration string...");
    let mins = duration.num_seconds() / 60;
    let secs = duration.num_seconds() % 60;
    let duration_field = format!("{}:{:02}", &mins, &secs);

    trace!("Determining match result...");
    let mut is_victory: bool = false;
    let mut is_defeat: bool = false;
    for player in players.iter().cloned() {
        trace!("Ensuring the player object exists...");
        let player: stratz::Player = player.ok_or_else(|| RefreshError::Data)?;
        trace!("Ensuring the victory property exists...");
        let player_result: bool = player.is_victory.ok_or_else(|| RefreshError::Data)?;
        trace!("Marking player's result...");
        match player_result {
            true => is_victory = true,
            false => is_defeat = true,
        }
    }
    let match_result = match (is_victory, is_defeat) {
        (false, false) => MatchResult::None,
        (true, false) => MatchResult::Victory,
        (false, true) => MatchResult::Defeat,
        (true, true) => MatchResult::Both,
    };
    trace!("Match result is: {match_result:?}");

    // Ughhh, I'd really like to use map-reduce here...
    trace!("Determining players' teams...");
    let mut radiant_players: Vec<stratz::Player> = Vec::new();
    let mut dire_players: Vec<stratz::Player> = Vec::new();
    for player in players.iter().cloned() {
        trace!("Ensuring the player object exists...");
        let player: stratz::Player = player.ok_or_else(|| RefreshError::Data)?;
        trace!("Ensuring the radiant property exists...");
        let player_team: bool = player.is_radiant.ok_or_else(|| RefreshError::Data)?;
        trace!("Marking player's team...");
        match player_team {
            true => radiant_players.push(player),
            false => dire_players.push(player),
        }
    }
    trace!("Teams determined successfully!");

    // Ughhh, here as well...
    trace!("Creating Radiant's players field...");
    let mut radiant_field = String::new();
    for player in radiant_players {
        let line = render_player(player)?;
        radiant_field.push_str(&line);
        radiant_field.push('\n');
    }
    trace!("Creating Dire's players field...");
    let mut dire_field = String::new();
    for player in dire_players {
        let line = render_player(player)?;
        dire_field.push_str(&line);
        dire_field.push('\n');
    }

    debug!("Sending match announcement...");
    client.send(|mut msg| {
        msg = msg.content(&*format!("https://stratz.com/matches/{}", &id));
        msg = msg.embed(|mut embed| {
            embed = embed.author(
                guild_name,
                Some(format!("https://stratz.com/guilds/{}", &guild_id)),
                Some(format!("https://steamusercontent-a.akamaihd.net/ugc/{}/", &guild_logo)),
            );
            embed = embed.title(&*format!(
                "{} · {} · {}",
                &*match match_result {
                    MatchResult::None => "Cancelled",
                    MatchResult::Victory => "Victory",
                    MatchResult::Defeat => "Defeat",
                    MatchResult::Both => "Clash",
                },
                &*match lobby_type {
                    stratz::LobbyType::UNRANKED => "Unranked",
                    stratz::LobbyType::PRACTICE => "Lobby",
                    stratz::LobbyType::TOURNAMENT => "The International",
                    stratz::LobbyType::TUTORIAL => "Tutorial",
                    stratz::LobbyType::COOP_VS_BOTS => "Bots",
                    stratz::LobbyType::TEAM_MATCH => "Guild",
                    stratz::LobbyType::SOLO_QUEUE => "Solo Ranked",
                    stratz::LobbyType::RANKED => "Ranked",
                    stratz::LobbyType::SOLO_MID => "Duel",
                    stratz::LobbyType::BATTLE_CUP => "Battle Cup",
                    stratz::LobbyType::EVENT => "Event",
                    _ => "Unknown",
                },
                &*match game_mode {
                    stratz::GameMode::NONE => "None",
                    stratz::GameMode::ALL_PICK => "All Pick",
                    stratz::GameMode::CAPTAINS_MODE => "Captains Mode",
                    stratz::GameMode::RANDOM_DRAFT => "Random Draft",
                    stratz::GameMode::SINGLE_DRAFT => "Single Draft",
                    stratz::GameMode::ALL_RANDOM => "All Random",
                    stratz::GameMode::INTRO => "Intro",
                    stratz::GameMode::THE_DIRETIDE => "Diretide",
                    stratz::GameMode::REVERSE_CAPTAINS_MODE => "Reverse Captains Mode",
                    stratz::GameMode::THE_GREEVILING => "Greeviling",
                    stratz::GameMode::TUTORIAL => "Tutorial",
                    stratz::GameMode::MID_ONLY => "Mid Only",
                    stratz::GameMode::LEAST_PLAYED => "Least Played",
                    stratz::GameMode::NEW_PLAYER_POOL => "Limited Heroes",
                    stratz::GameMode::COMPENDIUM_MATCHMAKING => "Compendium",
                    stratz::GameMode::CUSTOM => "Custom",
                    stratz::GameMode::CAPTAINS_DRAFT => "Captains Draft",
                    stratz::GameMode::BALANCED_DRAFT => "Balanced Draft",
                    stratz::GameMode::ABILITY_DRAFT => "Ability Draft",
                    stratz::GameMode::EVENT => "Event",
                    stratz::GameMode::ALL_RANDOM_DEATH_MATCH => "All Random Deathmatch",
                    stratz::GameMode::SOLO_MID => "Solo Mid",
                    stratz::GameMode::ALL_PICK_RANKED => "All Draft",
                    stratz::GameMode::TURBO => "Turbo",
                    stratz::GameMode::MUTATION => "Mutation",
                    _ => "Unknown",
                }
            ));
            embed = embed.color(&*match match_result {
                MatchResult::None => format!("{}", 0x5865F2),
                MatchResult::Victory => format!("{}", 0x57F287),
                MatchResult::Defeat => format!("{}", 0xED4245),
                MatchResult::Both => format!("{}", 0xFEE75C),
            });

            if radiant_field.len() > 0 {
                embed = embed.field("<:radiant:958274781919207505> Radiant", &radiant_field, true);
            }
            if dire_field.len() > 0 {
                embed = embed.field("<:dire:958274694203719740> Dire", &dire_field, true);
            }

            embed = embed.field(":clock3: Duration", &duration_field, false);

            // embed = embed.footer("Powered by STRATZ", Some(String::from("https://cdn.discordapp.com/icons/268890221943324677/12b63c55a83a715ec569e91e40641db0.webp?size=96")));
            embed = embed.timestamp(&end.to_rfc3339());

            return embed;
        });
        return msg;
    }).await.map_err(|_| RefreshError::Discord)?;

    Ok(())
}

fn render_player(player: stratz::Player) -> Result<String, RefreshError> {
    trace!("Ensuring the player's Steam account exists...");
    let steam: stratz::Steam = player.steam_account.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the player's name exists...");
    let name: String = steam.name.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the player's hero exists...");
    let hero: stratz::Hero = player.hero.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the player's hero ID exists...");
    let hero_id: i16 = hero.id.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the player's kill number exists...");
    let kills: u8 = player.kills.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the player's kill number exists...");
    let deaths: u8 = player.deaths.ok_or_else(|| RefreshError::Data)?;
    trace!("Ensuring the player's kill number exists...");
    let assists: u8 = player.assists.ok_or_else(|| RefreshError::Data)?;

    trace!("Matching hero ID to a Discord emoji...");
    let emoji = match hero_id {
        1 => "<:antimage:958248644652458005>",
        2 => "<:axe:958248644547608586>",
        3 => "<:bane:958249951480123394>",
        4 => "<:bloodseeker:958248644585332796>",
        5 => "<:crystal_maiden:958248644606320680>",
        6 => "<:drow_ranger:958248644799238194>",
        7 => "<:earthshaker:958248644748922900>",
        8 => "<:juggernaut:958248644853760052>",
        9 => "<:mirana:958248645038325771>",
        10 => "<:morphling:958248645025759282>",
        11 => "<:shadow_fiend:958248645147385866>",
        12 => "<:phantom_lancer:958249951857610772>",
        13 => "<:puck:958248645013147648>",
        14 => "<:pudge:958248645088645160>",
        15 => "<:razor:958248645134794762>",
        16 => "<:sand_king:958248645113815080>",
        17 => "<:storm_spirit:958249951262031934>",
        18 => "<:sven:958249951467548682>",
        19 => "<:tiny:958249951681450035>",
        20 => "<:vengeful_spirit:958249951710826516>",
        21 => "<:windranger:958249951652106310>",
        22 => "<:zeus:958249951459168288>",
        23 => "<:kunkka:958248645059313694>",
        25 => "<:lina:958248645000560660>",
        26 => "<:lion:958248644971229194>",
        27 => "<:shadow_shaman:958248645193502771>",
        28 => "<:slardar:958248645214486578>",
        29 => "<:tidehunter:958249951228469269>",
        30 => "<:witch_doctor:958249951715004446>",
        31 => "<:lich:958248644992172032>",
        32 => "<:riki:958248645138980914>",
        33 => "<:enigma:958248644954456094>",
        34 => "<:tinker:958249951480127518>",
        35 => "<:sniper:958248645155762196>",
        36 => "<:necrophos:958248644698595379>",
        37 => "<:warlock:958249951740182569>",
        38 => "<:beastmaster:958248644581146644>",
        39 => "<:queen_of_pain:958248644736331829>",
        40 => "<:venomancer:958249951580815400>",
        41 => "<:faceless_void:958248644912484382>",
        42 => "<:wraith_king:958248645239664700>",
        43 => "<:death_prophet:958248644740517910>",
        44 => "<:phantom_assassin:958249951941500938>",
        45 => "<:pugna:958248644937662465>",
        46 => "<:templar_assassin:958249952050544691>",
        47 => "<:viper:958249951207497769>",
        48 => "<:luna:958249951966674995>",
        49 => "<:dragon_knight:958248644803436544>",
        50 => "<:dazzle:958248644476301324>",
        51 => "<:clockwerk:958248645210284032>",
        52 => "<:leshrac:958248644912504883>",
        53 => "<:natures_prophet:958248644560162888>",
        54 => "<:lifestealer:958248645084467240>",
        55 => "<:dark_seer:958248644644073502>",
        56 => "<:clinkz:958249951735980042>",
        57 => "<:omniknight:958248645080252426>",
        58 => "<:enchantress:958248644853764097>",
        59 => "<:huskar:958248644967022642>",
        60 => "<:night_stalker:958248645004767282>",
        61 => "<:broodmother:958248644702777364>",
        62 => "<:bounty_hunter:958248644627271690>",
        63 => "<:weaver:958249951429812266>",
        64 => "<:jakiro:958249951568220190>",
        65 => "<:batrider:958248644560191589>",
        66 => "<:chen:958248644644057149>",
        67 => "<:spectre:958248645235474473>",
        69 => "<:doom:958248644698591232>",
        68 => "<:ancient_apparition:958248644572762153>",
        70 => "<:ursa:958249951845027860>",
        71 => "<:spirit_breaker:958249951492730900>",
        72 => "<:gyrocopter:958249951983456276>",
        73 => "<:alchemist:958248644719558716>",
        74 => "<:invoker:958249951429800009>",
        75 => "<:silencer:958248645143199774>",
        76 => "<:outworld_destroyer:958249951702441994>",
        77 => "<:lycan:958249951958290432>",
        78 => "<:brewmaster:958249951840854026>",
        79 => "<:shadow_demon:958249951454982187>",
        80 => "<:lone_druid:958249951798886400>",
        81 => "<:chaos_knight:958249951840845894>",
        82 => "<:meepo:958249952218345482>",
        83 => "<:treant_protector:958249951626924073>",
        84 => "<:ogre_magi:958249952000233472>",
        85 => "<:undying:958249951987634176>",
        86 => "<:rubick:958249951895388192>",
        87 => "<:disruptor:958249952256086046>",
        88 => "<:nyx_assassin:958249952130240562>",
        89 => "<:naga_siren:958249952100904990>",
        90 => "<:keeper_of_the_light:958249952105095218>",
        91 => "<:io:958249952054759424>",
        92 => "<:visage:958249952113459321>",
        93 => "<:slark:958249952218325002>",
        94 => "<:medusa:958249952193155092>",
        95 => "<:troll_warlord:958249952201564210>",
        96 => "<:centaur_warrunner:958249952184782848>",
        97 => "<:magnus:958249952226738196>",
        98 => "<:timbersaw:958249952251904050>",
        99 => "<:bristleback:958251187243745280>",
        100 => "<:tusk:958251186950111253>",
        101 => "<:skywrath_mage:958251187260502036>",
        102 => "<:abaddon:958251187180806146>",
        103 => "<:elder_titan:958251187289878598>",
        104 => "<:legion_commander:958251187117908018>",
        105 => "<:techies:958251187222740992>",
        106 => "<:ember_spirit:958251187143065610>",
        107 => "<:earth_spirit:958251187172438046>",
        108 => "<:underlord:958251187369549844>",
        109 => "<:terrorblade:958251187382153226>",
        110 => "<:phoenix:958251187214381096>",
        111 => "<:oracle:958251187306627072>",
        112 => "<:winter_wyvern:958251187281489980>",
        113 => "<:arc_warden:958251187340197898>",
        114 => "<:monkey_king:958251187205992469>",
        119 => "<:dark_willow:958251187591868446>",
        120 => "<:pangolier:958251187470233631>",
        121 => "<:grimstroke:958251187709304862>",
        123 => "<:hoodwink:958251187856105532>",
        126 => "<:void_spirit:958251187772215386>",
        128 => "<:snapfire:958251188023873587>",
        129 => "<:mars:958251187696726016>",
        135 => "<:dawnbreaker:958251187608645633>",
        136 => "<:marci:958254609397334026>",
        137 => "<:primal_beast:958254609397342258>",
        _ => ":grey_question:",
    };

    if let Some(imp) = player.imp {
        trace!("IMP is available, displaying it...");
        Ok(format!("{} {} [{}/{}/{}] `{:+}`", &emoji, &name, &kills, &deaths, &assists, &imp))
    }
    else {
        trace!("IMP is not available, ignoring it...");
        Ok(format!("{} {} [{}/{}/{}]", &emoji, &name, &kills, &deaths, &assists))
    }
}