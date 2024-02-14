use super::*;
//use tauri_plugin_http::reqwest;

/// The base URL for AnimeUnity site.
const LINK: &str = "https://www.animeunity.to";

/// Asynchronously searches for anime titles matching the provided keywords on AnimeUnity.
///
/// This function takes a reference to a [`reqwest::Client`] and a string of keywords to search for.
/// It sends a GET request to AnimeUnity's search endpoint with the provided keywords, extracts
/// relevant information from the HTML response, and returns a vector of [`Anime`] objects containing
/// titles and links of the matching anime.
///
/// # Arguments
///
/// * `client` - A reference to a [`reqwest::Client`] used to make HTTP requests.
/// * `keywords` - A string containing the keywords to search for anime titles.
///
/// # Returns
///
/// A vector of [`Anime`] objects containing titles and links of the matching anime.
///
/// # Errors
///
/// This function returns an empty vector if there is an error in sending the request,
/// handling the response, or parsing the JSON data.
pub async fn search(client: &reqwest::Client, keywords: &str) -> Vec<Anime> {
    // Initialize a vector to store Anime instances.
    let mut names = Vec::<Anime>::new();

    // Define the CSS selector to extract relevant information from the HTML response.
    let items_selector = scraper::Selector::parse("archivio").unwrap();

    // Construct the URL for the search query on AnimeUnity.
    let url = format!("{}/archivio?title={}", animeunity::LINK, keywords);

    // Send a GET request to the URL and handle the response.
    let resp = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Unable to fetch webpage: {}", err);
            return names;
        }
    };

    // If the response is successful, attempt to extract HTML content.
    let html = match resp.text().await {
        Ok(html) => html,
        Err(err) => {
            eprintln!("Unable to get response value: {}", err);
            return names;
        }
    };

    // Parse the HTML document.
    let document = scraper::Html::parse_document(&html);

    // Extract JSON data from the HTML document and parse it.
    let records_attr = match document
        .select(&items_selector)
        .next()
        .and_then(|elem| elem.attr("records"))
    {
        Some(r) => r,
        None => {
            eprintln!("Unable to find any element that match");
            return names;
        }
    };

    // Parse the JSON data into a serde_json::Value.
    let json = match serde_json::from_str::<serde_json::Value>(records_attr) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("Unable to parse json: {}", err);
            return names;
        }
    };

    // Check if the JSON data is an array.
    let json_array = match json.as_array() {
        Some(j) => j,
        None => {
            eprintln!("Not an array");
            return names;
        }
    };

    // Iterate over JSON objects in the array.
    for json_obj in json_array {
        // Extract ID, slug, and title_eng from each JSON object.
        let mut id = String::new();
        let mut slug = String::new();

        let mut name = String::new();
        let mut year = String::new();
        let mut state = AnimeState::NonValido;
        let mut description = String::new();
        let mut genres = Vec::<String>::new();
        let mut studio = String::new();
        let mut stars = String::new();
        let mut cover = String::new();
        let mut cover_full = String::new();
        let mut banner = String::new();

        if let Some(id_value) = json_obj.get("id") {
            id = id_value.as_u64().unwrap().to_string();
        }
        if let Some(slug_value) = json_obj.get("slug") {
            slug = slug_value.as_str().unwrap().to_string();
        }
        if let Some(name_value) = json_obj.get("title_eng") {
            name = name_value.as_str().unwrap().to_string();
        }
        if let Some(year_value) = json_obj.get("date") {
            year = year_value.as_str().unwrap().to_string();
        }
        if let Some(state_value) = json_obj.get("status") {
            state = match state_value.as_str().unwrap() {
                "In Corso" => AnimeState::InCorso,
                "Terminato" => AnimeState::Finito,
                _ => AnimeState::NonValido,
            };
        }
        if let Some(genres_value) = json_obj.get("genres") {
            for genre in genres_value.as_array().unwrap() {
                genres.push(genre.get("name").unwrap().as_str().unwrap().to_string());
            }
        }
        if let Some(studio_value) = json_obj.get("studio") {
            studio = studio_value.as_str().unwrap().to_string();
        }
        if let Some(stars_value) = json_obj.get("score") {
            stars = stars_value.as_str().unwrap().to_string();
        }
        if let Some(description_value) = json_obj.get("plot") {
            description = description_value.as_str().unwrap().to_string();
        }
        if let Some(cover_value) = json_obj.get("imageurl") {
            cover = cover_value.as_str().unwrap().to_string();
        }
        if let Some(cover_full_value) = json_obj.get("cover") {
            cover_full = cover_full_value.as_str().unwrap_or("").to_string();
        }
        if let Some(banner_value) = json_obj.get("imageurl_cover") {
            banner = banner_value.as_str().unwrap_or("").to_string();
        }

        // Create a new Anime instance and add it to the vector.
        names.push(Anime::new(
            Sites::AnimeUnity,
            format!("/anime/{}-{}", id, slug),
            AnimeInfo::new(
                name,
                year,
                state,
                description,
                genres,
                studio,
                stars,
                cover,
                cover_full,
                banner,
            ),
        ));
    }

    // Return the vector containing Anime instances.
    names
}

// TODO: add function to set update progress

/// Fetches episodes of a specific anime from AnimeUnity.
///
/// # Arguments
///
/// * `client` - The reqwest client to make HTTP requests.
/// * `anime` - The Anime instance to fetch episodes for.
/// * `range` - An optional range of episode numbers to fetch.
///
/// # Returns
///
/// An AnimeEpisodes instance containing episodes of the anime.
pub async fn get_anime_episodes(
    client: &reqwest::Client,
    anime: Anime,
    range: Option<std::ops::RangeInclusive<usize>>,
) -> AnimeEpisodes {
    // Initialize a vector to store Episode instances.
    let mut episodes = Vec::<Episode>::new();

    // Define the CSS selector to extract relevant information from the HTML response.
    let body_selector = scraper::Selector::parse("body").unwrap();

    // Extract the anime ID from the anime link.
    let anime_id = anime
        .link
        .matches(char::is_numeric)
        .collect::<Vec<&str>>()
        .join("");

    // Construct the URL for fetching information about the anime.
    let mut url = format!("{}/info_api/{}/", animeunity::LINK, anime_id);

    // Send a GET request to the URL and handle the response.
    match client.get(&url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(html) => {
                // Parse the HTML document.
                let document = scraper::Html::parse_document(&html);

                // Define a regex pattern to extract episode count.
                let re = regex::Regex::new(r#""episodes_count":(\d+)"#).unwrap();

                // Extract episode count from the HTML document.
                if let Some(captures) = re.captures(document.html().as_str()) {
                    if let Some(episodes_count) = captures.get(1) {
                        // Calculate the number of pages needed to fetch all episodes.
                        let max = ((episodes_count.as_str().parse::<usize>().unwrap() + 119) / 120)
                            as usize;

                        // Iterate over each page to fetch episodes.
                        for i in 1..=max {
                            // Construct URL for fetching episodes for a specific page.
                            url = format!(
                                "{}/info_api/{}/1?start_range={}&end_range={}",
                                animeunity::LINK,
                                anime_id,
                                (i - 1) * 120 + 1,
                                i * 120
                            );

                            // Send a GET request to the URL and handle the response.
                            match client.get(&url).send().await {
                                Ok(resp) => match resp.text().await {
                                    Ok(html) => {
                                        // Parse the HTML document.
                                        let document = scraper::Html::parse_document(&html);

                                        // Extract JSON data from the HTML document.
                                        let json_str = document
                                            .select(&body_selector)
                                            .next()
                                            .unwrap()
                                            .text()
                                            .collect::<Vec<&str>>()[0];

                                        // Parse the JSON data into a serde_json::Value.
                                        if let Ok(json) =
                                            serde_json::from_str::<serde_json::Value>(json_str)
                                        {
                                            // Extract episodes from the JSON data.
                                            if let Some(json_pre) = json.get("episodes") {
                                                if let Some(json_array) = json_pre.as_array() {
                                                    // Iterate over each episode JSON object.
                                                    for json_obj in json_array {
                                                        // Extract episode number and ID.
                                                        let number = json_obj
                                                            .get("number")
                                                            .and_then(|v| v.as_str())
                                                            .and_then(|s| s.parse::<usize>().ok())
                                                            .unwrap_or(0);

                                                        let episode_id = json_obj
                                                            .get("id")
                                                            .and_then(|v| v.as_u64())
                                                            .map(|id| id.to_string())
                                                            .unwrap_or_default();

                                                        // Check if the episode number is within the specified range.
                                                        if range
                                                            .as_ref()
                                                            .map_or(true, |r| r.contains(&number))
                                                        {
                                                            // Create a new Episode instance and add it to the vector.
                                                            episodes.push(Episode::new(
                                                                number, episode_id,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }

    // Create and return an AnimeEpisodes instance containing fetched episodes.
    AnimeEpisodes::new(Sites::AnimeUnity, anime.link, episodes)
}

/// Asynchronously fetches video links for a range of anime episodes.
///
/// This function takes a reference to a [`reqwest::Client`], an [`AnimeEpisodes`] object containing
/// information about all episodes of the anime, and a range of episode indices. It filters episodes
/// to retain only those within the specified range, fetches video links for each episode asynchronously,
/// and returns a vector of [`Video`] objects containing the retrieved links.
///
/// # Arguments
///
/// * `client` - A reference to a [`reqwest::Client`] used to make HTTP requests.
/// * `anime_episodes` - An [`AnimeEpisodes`] struct containing information about all episodes of the anime.
/// * `range` - A range of episode indices (inclusive) for which to retrieve video links.
///
/// # Returns
///
/// A vector of [`Video`] objects containing links to the requested episodes.
pub async fn get_episodes_link(
    client: &reqwest::Client,
    anime_episodes: AnimeEpisodes,
    range: std::ops::RangeInclusive<usize>,
) -> Vec<Video> {
    let mut videos = Vec::<Video>::new();

    // Filter episodes to retain only those within the specified range
    let episodes = anime_episodes
        .episodes
        .iter()
        .filter_map(|episode| {
            if range.contains(&episode.number) {
                Some(episode)
            } else {
                None
            }
        })
        .collect::<Vec<&Episode>>();

    // Define the CSS selector to extract video information from the HTML response.
    let video_selector = scraper::Selector::parse("video-player").unwrap();

    // Iterate over the filtered episodes and fetch video links asynchronously.
    for episode in episodes {
        let url = format!(
            "{}{}/{}",
            animeunity::LINK,
            anime_episodes.link,
            episode.episode_id
        );

        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(html) = resp.text().await {
                let document = scraper::Html::parse_document(&html);

                // Extract the video link from the HTML document and create a Video instance.
                if let Some(embed_url) = document
                    .select(&video_selector)
                    .next()
                    .and_then(|elem| elem.attr("embed_url"))
                {
                    videos.push(Video::new(embed_url.to_string()));
                }
            }
        }
    }

    videos
}
