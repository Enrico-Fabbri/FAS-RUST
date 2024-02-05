use super::*;

// TODO: finish this mod

const LINK: &str = "https://aniplay.co";

/// Asynchronously searches for anime titles matching the provided keywords on AniPlay.
///
/// This function takes a reference to a reqwest [`Client`] and a string of keywords to search for.
/// It sends a GET request to AniPlay's search endpoint with the provided keywords, extracts
/// relevant information from the JSON response, and returns a vector of [`Anime`] objects containing
/// titles and links of the matching anime.
///
/// # Arguments
///
/// * `client` - A reference to a reqwest [`Client`] used to make HTTP requests.
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
    if let Ok(resp) = client.get(&url).send().await {
        // If the response is successful, attempt to extract JSON content.
        if let Ok(html) = resp.text().await {
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
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                // Check if the JSON data contains anime information.
                if let Some(json_pre) = json.get("data") {
                    // Check if the anime information is in an array.
                    if let Some(json_array) = json_pre.as_array() {
                        // Iterate over JSON objects in the array.
                        for json_obj in json_array {
                            // Extract ID and title from each JSON object.
                            let id = json_obj
                                .get("id")
                                .unwrap()
                                .as_u64()
                                .map(|f| f.to_string())
                                .unwrap();
                            let title =
                                json_obj.get("title").unwrap().as_str().unwrap().to_string();

                            // Create a new Anime instance and add it to the vector.
                            names.push(Anime::new(
                                Sites::AniPlay,
                                title,
                                format!("/series/{}", id),
                            ));
                        }
                    }
                }
            }
        }
    }

    names
}

/// Asynchronously retrieves information about anime episodes from AniPlay.
///
/// This function takes a reference to a reqwest [`Client`], an [`Anime`] object representing
/// the anime to fetch episodes for, and an optional range of episode indices. It sends a GET
/// request to AniPlay's endpoint for the specified anime, extracts episode information from the
/// HTML response, and returns an [`AnimeEpisodes`] object containing the fetched episodes.
///
/// # Arguments
///
/// * `client` - A reference to a reqwest [`Client`] used to make HTTP requests.
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
/// This function takes a reference to a reqwest [`Client`], an [`AnimeEpisodes`] object containing
/// information about all episodes of the anime, and a range of episode indices. It sends a GET request
/// to AniPlay's endpoint for the last episode of the anime, extracts episode information from the HTML
/// response, and returns a vector of [`Video`] objects containing the retrieved video links for episodes
/// within the specified range.
///
/// # Arguments
///
/// * `client` - A reference to a reqwest [`Client`] used to make HTTP requests.
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
