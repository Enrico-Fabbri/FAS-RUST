use super::*;
//use tauri_plugin_http::reqwest;

const LINK: &str = "https://aniplay.co";

/// Asynchronously searches for anime titles matching the provided keywords on AniPlay.
///
/// This function takes a reference to a [`reqwest::Client`] and a string of keywords to search for.
/// It sends a GET request to AniPlay's search endpoint with the provided keywords, extracts
/// relevant information from the JSON response, and returns a vector of [`Anime`] objects containing
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
pub async fn search(client: &reqwest::Client, keywords: &str) -> Vec<Anime> {
    let mut names = Vec::new();

    // Define the CSS selector to extract relevant information from the JSON response.
    let json_selector = scraper::Selector::parse("body").unwrap();

    // Construct the URL for the search query on AniPlay.
    let url = format!(
        "https://api.aniplay.co/api/series/advancedSearch?sort=1&page=1&_q={}",
        keywords
    );

    // Send a GET request to the URL and handle the response.
    let resp = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Unable to fetch webpage: {}", err);
            return names;
        }
    };

    // If the response is successful, attempt to extract JSON content.
    let html = match resp.text().await {
        Ok(html) => html,
        Err(err) => {
            eprintln!("Unable to get response value: {}", err);
            return names;
        }
    };

    // Parse the JSON document.
    let document = scraper::Html::parse_document(&html);

    // Extract the JSON body from the HTML document.
    let body = document
        .select(&json_selector)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<&str>>()[0];

    // Parse the JSON data into a serde_json::Value.
    let json = match serde_json::from_str::<serde_json::Value>(body) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("Unable to parse json: {}", err);
            return names;
        }
    };

    // Check if the JSON data contains anime information.
    let json_pre = match json.get("data") {
        Some(json_pre) => json_pre,
        None => {
            eprintln!("Value not found");
            return names;
        }
    };

    // Check if the anime information is in an array.
    let json_array = match json_pre.as_array() {
        Some(j) => j,
        None => {
            eprintln!("Not an array");
            return names;
        }
    };

    // Iterate over JSON objects in the array.
    for json_obj in json_array {
        let mut id = String::new();

        let mut name = String::new();
        let mut year = String::new();
        let mut state = AnimeState::NonValido;
        let mut description = String::new();
        let mut genres = Vec::<String>::new();
        let mut studio = String::new();
        let mut stars = String::new();
        let mut cover = String::new();
        let mut cover_full = String::new();

        // Extract ID and title from each JSON object.
        if let Some(id_value) = json_obj.get("id") {
            id = id_value.as_u64().unwrap().to_string();
        }
        if let Some(name_value) = json_obj.get("title") {
            name = name_value.as_str().unwrap().to_string();
        }
        if let Some(year_value) = json_obj.get("release_date") {
            year = year_value.as_str().unwrap().to_string();
        }
        if let Some(state_value) = json_obj.get("status") {
            state = match state_value.as_str().unwrap() {
                "In corso" => AnimeState::InCorso,
                "Completato" => AnimeState::Finito,
                _ => AnimeState::NonValido,
            };
        }
        if let Some(genres_value) = json_obj.get("genres") {
            for genre in genres_value.as_array().unwrap() {
                genres.push(genre.get("name").unwrap().as_str().unwrap().to_string());
            }
        }
        if let Some(studios_value) = json_obj.get("studios") {
            for s in studios_value.as_array().unwrap() {
                studio = s.get("name").unwrap().as_str().unwrap().to_string();
                break;
            }
        }
        if let Some(stars_value) = json_obj.get("score") {
            stars = stars_value.as_f64().unwrap_or(std::f64::NAN).to_string();
        }
        if let Some(description_value) = json_obj.get("description") {
            description = description_value.as_str().unwrap().to_string();
        }
        if let Some(cover_value) = json_obj.get("cover") {
            cover = cover_value.as_str().unwrap().to_string();
        }
        if let Some(cover_full_value) = json_obj.get("main_image") {
            cover_full = cover_full_value.as_str().unwrap_or("").to_string();
        }
        let banner = "".to_string();

        // Create a new Anime instance and add it to the vector.
        names.push(Anime::new(
            Sites::AniPlay,
            format!("/series/{}", id),
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

    names
}

/// Asynchronously retrieves information about anime episodes from AniPlay.
///
/// This function takes a reference to a [`reqwest::Client`], an [`Anime`] object representing
/// the anime to fetch episodes for, and an optional range of episode indices. It sends a GET
/// request to AniPlay's endpoint for the specified anime, extracts episode information from the
/// HTML response, and returns an [`AnimeEpisodes`] object containing the fetched episodes.
///
/// # Arguments
///
/// * `client` - A reference to a [`reqwest::Client`] used to make HTTP requests.
/// * `anime` - An [`Anime`] object representing the anime for which to fetch episodes.
/// * `range` - An optional range of episode indices (inclusive) for which to retrieve episodes.
///
/// # Returns
///
/// An [`AnimeEpisodes`] object containing information about the fetched episodes.
pub async fn get_anime_episodes(
    client: &reqwest::Client,
    anime: Anime,
    range: Option<std::ops::RangeInclusive<usize>>,
) -> AnimeEpisodes {
    let mut episodes = Vec::<Episode>::new();

    // Define the CSS selector to extract relevant information from the HTML response.
    let selector = scraper::Selector::parse("script").unwrap();

    // Construct the URL for the anime page on AniPlay.
    let url = format!("{}{}", aniplay::LINK, anime.link);

    // Send a GET request to the URL and handle the response.
    if let Ok(resp) = client.get(&url).send().await {
        // If the response is successful, attempt to extract HTML content.
        if let Ok(html) = resp.text().await {
            // Parse the HTML document.
            let document = scraper::Html::parse_document(&html);

            // Extract the script element containing episode information.
            let script_item = document.select(&selector).last().unwrap();
            let script = script_item.text().collect::<Vec<&str>>()[0];

            // Define regular expressions for extracting episode data.
            let re = regex::Regex::new(r#"episodes:\s*\[(.*?)\]"#).unwrap();
            let reg = regex::Regex::new(r#"id:(\d+)"#).unwrap();
            let rege = regex::Regex::new(r#"number:"(\d+)""#).unwrap();

            // Extract and process episode information and episode number using regular expressions.
            if let Some(captures) = re.captures(script) {
                if let Some(data) = captures.get(1) {
                    let data = data.as_str().replace("episodes:[", "");
                    let _ = data
                        .split("},{")
                        .map(|s| {
                            if let Some(id_capture) = reg.captures(s) {
                                if let Some(id) = id_capture.get(1) {
                                    if let Some(number_capture) = rege.captures(s) {
                                        if let Some(number) = number_capture.get(1) {
                                            let n = number.as_str().parse::<usize>().unwrap();

                                            if range.as_ref().map_or(true, |r| r.contains(&n)) {
                                                episodes
                                                    .push(Episode::new(n, id.as_str().to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        })
                        .collect::<Vec<_>>();
                }
            }
        }
    }

    // Create and return an AnimeEpisodes instance containing fetched episodes.
    AnimeEpisodes::new(Sites::AniPlay, anime.link, episodes)
}

/// Asynchronously retrieves video links for a range of anime episodes from AniPlay.
///
/// This function takes a reference to a [`reqwest::Client`], an [`AnimeEpisodes`] object containing
/// information about all episodes of the anime, and a range of episode indices. It sends a GET request
/// to AniPlay's endpoint for the last episode of the anime, extracts episode information from the HTML
/// response, and returns a vector of [`Video`] objects containing the retrieved video links for episodes
/// within the specified range.
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

    // Define the CSS selector to extract relevant information from the HTML response.
    let selector = scraper::Selector::parse("script").unwrap();

    // Construct the URL for the last episode of the anime on AniPlay.
    let url = format!(
        "{}/watch/{}",
        aniplay::LINK,
        anime_episodes.episodes.last().unwrap().episode_id
    );

    // Define regular expressions for extracting episode data.
    let re = regex::Regex::new(r#"episodes:\s*\[(.*?)\]"#).unwrap();
    let reg = regex::Regex::new(r#"streaming_link:"([^"]+)""#).unwrap();
    let rege = regex::Regex::new(r#"number:"(\d+)""#).unwrap();

    // Send a GET request to the URL and handle the response.
    if let Ok(resp) = client.get(&url).send().await {
        // If the response is successful, attempt to extract HTML content.
        if let Ok(html) = resp.text().await {
            // Parse the HTML document.
            let document = scraper::Html::parse_document(&html);

            // Extract the script element containing episode information.
            if let Some(script_item) = document.select(&selector).last() {
                let script = script_item.text().collect::<Vec<&str>>()[0];

                // Extract and process episode information and episode number using regular expressions.
                if let Some(captures) = re.captures(script) {
                    if let Some(data) = captures.get(1) {
                        let data = data.as_str().replace("episodes:[", "");
                        // Split the data into individual episode strings and process each one.
                        let _ = data
                            .split("},{")
                            .map(|s| {
                                // Extract the link, episode number, and add video links for episodes within the specified range.
                                if let Some(link_capture) = reg.captures(s) {
                                    if let Some(link) = link_capture.get(1) {
                                        if let Some(number_capture) = rege.captures(s) {
                                            if let Some(number) = number_capture.get(1) {
                                                let n = number.as_str().parse::<usize>().unwrap();
                                                if range.contains(&n) {
                                                    videos.push(Video::new(
                                                        link.as_str().to_string(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            })
                            .collect::<Vec<_>>();
                    }
                }
            }
        }
    }

    videos
}
