//! Refresh tokens aren't meant to expire, so they can be used as a persistent
//! authentication method without the need for user's interaction for
//! oauth requests. You still need to authenticate the usual way at least
//! once to obtain the refresh token, and you may need to obtain a new one
//! if you change the required scope.
//!
//! The cache generated by `get_token` uses the refresh token under the hood
//! to automatically authenticate the user. This example shows how it's done
//! because sometimes it's not possible to use this cache file (a web server
//! for example).
//!
//! *Note*: refresh tokens can actually expire, [as the OAuth2 spec
//! indicates](https://tools.ietf.org/html/rfc6749#section-6),
//! but this [hasn't actually happened in months with some
//! tokens](https://github.com/felix-hilden/tekore/issues/86),
//! so in the case of Spotify it doesn't seem to revoke them at all.

use dotenv::dotenv;

use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
use rspotify::util::get_token_without_cache;

async fn get_refresh_token(oauth: &mut SpotifyOAuth) -> String {
    let token = get_token_without_cache(oauth)
        .await
        .expect("couldn't authenticate successfully");
    token
        .refresh_token
        .expect("couldn't obtain a refresh token")
}

async fn client_from_refresh_token(oauth: &SpotifyOAuth, refresh_token: &str) -> Spotify {
    let token_info = oauth
        .refresh_access_token_without_cache(refresh_token)
        .await
        .expect("couldn't refresh access token with the refresh token");

    // Building the client credentials, now with the access token.
    let client_credential = SpotifyClientCredentials::default()
        .token_info(token_info)
        .build();

    // Initializing the Spotify client finally.
    Spotify::default()
        .client_credentials_manager(client_credential)
        .build()
}

// Sample request that will follow some artists, print the user's
// followed artists, and then unfollow the artists.
async fn do_things(spotify: Spotify) {
    let artists = vec![
        "3RGLhK1IP9jnYFH4BRFJBS".to_owned(), // The Clash
        "0yNLKJebCb8Aueb54LYya3".to_owned(), // New Order
        "2jzc5TC5TVFLXQlBNiIUzE".to_owned(), // a-ha
    ];
    spotify
        .user_follow_artists(&artists)
        .await
        .expect("couldn't follow artists");
    println!("Followed {} artists successfully.", artists.len());

    // Printing the followed artists
    let followed = spotify
        .current_user_followed_artists(None, None)
        .await
        .expect("couldn't get user followed artists");
    println!(
        "User currently follows at least {} artists.",
        followed.artists.items.len()
    );

    spotify
        .user_unfollow_artists(&artists)
        .await
        .expect("couldn't unfollow artists");
    println!("Unfollowed {} artists successfully.", artists.len());
}

#[tokio::main]
async fn main() {
    // The default credentials from the `.env` file will be used by default.
    dotenv().ok();
    let mut oauth = SpotifyOAuth::default()
        .scope("user-follow-read user-follow-modify")
        .build();

    // In the first session of the application we only authenticate and obtain
    // the refresh token.
    println!(">>> Session one, obtaining refresh token:");
    let refresh_token = get_refresh_token(&mut oauth).await;

    // At a different time, the refresh token can be used to refresh an access
    // token directly and run requests:
    println!(">>> Session two, running some requests:");
    let spotify = client_from_refresh_token(&mut oauth, &refresh_token).await;
    do_things(spotify).await;

    // This process can now be repeated multiple times by using only the
    // refresh token that was obtained at the beginning.
    println!(">>> Session three, running some requests:");
    let spotify = client_from_refresh_token(&mut oauth, &refresh_token).await;
    do_things(spotify).await;
}