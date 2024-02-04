use fas::*;

#[tokio::test]
async fn test_search_animeworld() {
    let client = reqwest::Client::new();
    let names = animeworld::search(&client, "one piece").await;

    //println!("{:#?}", names);
}

#[tokio::test]
async fn test_get_anime_episodes_no_range_animeworld() {
    let client = reqwest::Client::new();
    let episodes = animeworld::get_anime_episodes(
        &client,
        Anime::new(
            Sites::AnimeWorld,
            "".to_string(),
            "/play/one-piece-subita.qzG-LE".to_string(),
        ),
        None,
    )
    .await;

    //println!("{:#?}", episodes)
}

#[tokio::test]
async fn test_get_anime_episodes_range_animeworld() {
    let client = reqwest::Client::new();
    let episodes = animeworld::get_anime_episodes(
        &client,
        Anime::new(
            Sites::AnimeWorld,
            "".to_string(),
            "/play/one-piece-subita.qzG-LE".to_string(),
        ),
        Some(1090..=2000),
    )
    .await;

    //println!("{:#?}", episodes)
}

#[tokio::test]
async fn test_search_animeunity() {
    let client = reqwest::Client::new();
    let names = animeunity::search(&client, "one piece").await;

    //println!("{:#?}", names);
}

#[tokio::test]
async fn test_get_anime_episodes_no_range_animeunity() {
    let client = reqwest::Client::new();
    let episodes = animeunity::get_anime_episodes(
        &client,
        Anime::new(
            Sites::AnimeUnity,
            "".to_string(),
            "/anime/12-one-piece".to_string(),
        ),
        None,
    )
    .await;

    //println!("{:#?}", episodes)
}

#[tokio::test]
async fn test_get_anime_episodes_range_animeunity() {
    let client = reqwest::Client::new();
    let episodes = animeunity::get_anime_episodes(
        &client,
        Anime::new(
            Sites::AnimeUnity,
            "".to_string(),
            "/anime/12-one-piece".to_string(),
        ),
        Some(1090..=1800),
    )
    .await;

    //println!("{:#?}", episodes)
}

#[tokio::test]
async fn test_get_episodes_link_animeunity() {
    let client = reqwest::Client::new();
    let anime_episodes = AnimeEpisodes::new(
        Sites::AnimeUnity,
        "/anime/12-one-piece".to_string(),
        vec![
            Episode::new(1090, "76414".to_string()),
            Episode::new(1091, "76514".to_string()),
            Episode::new(1092, "76612".to_string()),
        ],
    );

    let videos = animeunity::get_episodes_link(&client, anime_episodes, 1090..=1092).await;

    //println!("{:#?}", videos);
}
