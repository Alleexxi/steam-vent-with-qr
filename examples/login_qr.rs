use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;
use std::error::Error;
use steam_vent::auth::QrChallengeHandler;
use steam_vent::{Connection, ConnectionTrait, ServerList};
use steam_vent_proto::steammessages_player_steamclient::CPlayer_GetOwnedGames_Request;

struct AsciiQrHandler;

impl QrChallengeHandler for AsciiQrHandler {
    async fn handle_challenge_url(&mut self, challenge_url: &str) {
        let code = match QrCode::new(challenge_url.as_bytes()) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("failed to encode QR ({e}); URL: {challenge_url}");
                return;
            }
        };
        let rendered = code
            .render::<Dense1x2>()
            .dark_color(Dense1x2::Light)
            .light_color(Dense1x2::Dark)
            .quiet_zone(true)
            .build();
        println!("scan this QR with the Steam mobile app:\n");
        println!("{rendered}");
        println!("URL: {challenge_url}\n");
    }

    async fn handle_remote_interaction(&mut self) {
        println!("scanned, waiting for approval...");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let server_list = ServerList::discover().await?;
    let connection = Connection::login_qr(&server_list, AsciiQrHandler).await?;

    println!("logged in as {}", connection.steam_id().steam3());
    println!("requesting games");

    let req = CPlayer_GetOwnedGames_Request {
        steamid: Some(connection.steam_id().into()),
        include_appinfo: Some(true),
        include_played_free_games: Some(true),
        ..CPlayer_GetOwnedGames_Request::default()
    };
    let games = connection.service_method(req).await?;
    println!(
        "{} owns {} games",
        connection.steam_id().steam3(),
        games.game_count()
    );
    for game in games.games {
        println!(
            "{}: {} {}",
            game.appid(),
            game.name(),
            game.playtime_forever()
        );
    }

    Ok(())
}
