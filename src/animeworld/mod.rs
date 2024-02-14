use super::*;
//use tauri_plugin_http::reqwest;

/// The base URL for AnimeWorld site.
const LINK: &str = "https://animeworld.so";

/// Asynchronously searches for anime titles matching the provided keywords.
///
/// This function takes a reference to a [`reqwest::Client`] and a string of keywords to search for.
/// It sends a GET request to AnimeWorld's search endpoint with the provided keywords, extracts
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
pub async fn search(client: &reqwest::Client, keywords: &str) -> Vec<Anime> {
    // Initialize a vector to store Anime instances found matching the keywords.
    let mut names = Vec::<Anime>::new();

    // Define the CSS selector to extract relevant information from the HTML response.
    let items_selector = scraper::Selector::parse(".widget-body .film-list .item .name").unwrap();

    let info_selector = scraper::Selector::parse(".widget.info .widget-body .row").unwrap();
    let image_selector = scraper::Selector::parse(".thumb img").unwrap();
    let name_selector = scraper::Selector::parse(".head .title").unwrap();
    let desc_selector = scraper::Selector::parse(".desc").unwrap();
    let other_selector = scraper::Selector::parse(".row").unwrap();

    // Define regex patterns to extract information
    let year_regex = regex::Regex::new(r"Data di Uscita:\s*([\w\s]+)\n").unwrap();
    let state_regex = regex::Regex::new(r"Stato:\s*(.*)").unwrap();
    let genres_regex = regex::Regex::new(r"Genere:\s*([\w\s,]+)\n").unwrap();
    let studio_regex = regex::Regex::new(r"Studio:\s*([\w\s]+)\n").unwrap();
    let stars_regex = regex::Regex::new(r"Voto:\s*([\d.]+)").unwrap();

    // Construct the URL for the search query on AnimeWorld.
    let mut url = format!("{}/search?keyword={}", animeworld::LINK, keywords);

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

    // Iterate over items matching the specified CSS selector.
    for item in document.select(&items_selector) {
        // Extract the link of the anime item.
        let link = item.attr("href").unwrap().to_owned();

        url = format!("{}{}", animeworld::LINK, link);

        let respo = match client.get(&url).send().await {
            Ok(respo) => respo,
            Err(err) => {
                eprintln!("Unable to fetch webpage: {}", err);
                return names;
            }
        };

        let htm = match respo.text().await {
            Ok(htm) => htm,
            Err(err) => {
                eprintln!("Unable to get response value: {}", err);
                return names;
            }
        };

        let document = scraper::Html::parse_document(&htm);

        let info = document.select(&info_selector).next().unwrap();

        let name = info
            .select(&name_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let data = info
            .select(&other_selector)
            .map(|e| e.text().collect::<String>())
            .collect::<String>();

        let year = year_regex
            .captures(&data)
            .map(|captures| captures.get(1).unwrap().as_str().trim().to_string())
            .unwrap();

        let state = match state_regex
            .captures(&data)
            .map(|captures| captures.get(1).unwrap().as_str().trim())
            .unwrap()
        {
            "In corso" => AnimeState::InCorso,
            "Finito" => AnimeState::Finito,
            _ => AnimeState::NonValido,
        };

        let genres = genres_regex
            .captures(&data)
            .and_then(|captures| captures.get(1))
            .map_or_else(Vec::new, |genre_match| {
                genre_match
                    .as_str()
                    .split(',')
                    .map(|genre| genre.trim().to_string())
                    .collect()
            });

        let studio = studio_regex
            .captures(&data)
            .map(|captures| captures.get(1).unwrap().as_str().trim().to_string())
            .unwrap();

        let stars = stars_regex
            .captures(&data)
            .map(|captures| captures.get(1).unwrap().as_str().trim().to_string())
            .unwrap();

        let description = info
            .select(&desc_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let cover = info
            .select(&image_selector)
            .next()
            .unwrap()
            .attr("src")
            .unwrap()
            .to_string();

        let cover_full = "".to_string();

        let banner = "".to_string();

        // Create a new Anime instance and add it to the vector.
        names.push(Anime::new(
            Sites::AnimeWorld,
            link,
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

/// Asynchronously fetches episodes of a specific anime from a website.
///
/// This function takes a reference to a [`reqwest::Client`], an [`Anime`] object representing the anime,
/// and an optional range of episode indices, and returns an [`AnimeEpisodes`] struct containing
/// information about the fetched episodes.
///
/// # Arguments
///
/// * `client` - A reference to a [`reqwest::Client`] used to make HTTP requests.
/// * `anime` - An [`Anime`] object representing the anime for which episodes are to be fetched.
/// * `range` - An optional range of episode indices (inclusive) to limit the fetched episodes. If `None`,
///   all episodes will be fetched.
///
/// # Returns
///
/// An [`AnimeEpisodes`] struct containing information about the fetched episodes.
pub async fn get_anime_episodes(
    client: &reqwest::Client,
    anime: Anime,
    range: Option<std::ops::RangeInclusive<usize>>,
) -> AnimeEpisodes {
    // Initialize a vector to store Episode instances.
    let mut episodes = Vec::<Episode>::new();

    // Define the CSS selector to extract episode information from the HTML response.
    let episode_selector = scraper::Selector::parse(".server.active .episodes .episode a").unwrap();

    // Construct the URL to fetch episodes for the specified anime.
    let url = format!("{}{}", animeworld::LINK, anime.link);

    // Send a GET request to the URL and handle the response.
    if let Ok(resp) = client.get(&url).send().await {
        // If the response is successful, attempt to extract HTML content.
        if let Ok(html) = resp.text().await {
            // Parse the HTML document.
            let document = scraper::Html::parse_document(&html);

            // Define a closure to add video IDs to the episodes vector.
            let mut add_video_id = |episode: &scraper::ElementRef<'_>, index: usize| {
                episodes.push(Episode::new(
                    index,
                    episode.attr("data-id").unwrap().to_owned(),
                ));
            };

            // Iterate over episodes matching the specified CSS selector.
            for (index, episode) in document.select(&episode_selector).enumerate() {
                // Check if the episode index is within the specified range, if provided.
                if range.as_ref().map_or(true, |r| r.contains(&(index + 1))) {
                    // Add the video ID to the episodes vector.
                    add_video_id(&episode, index + 1);
                }
            }
        }
    }

    // Create and return an AnimeEpisodes instance containing fetched episodes.
    AnimeEpisodes::new(Sites::AnimeWorld, anime.link, episodes)
}

/// Generates a list of video links for a range of anime episodes.
///
/// This function takes in an [`AnimeEpisodes`] object and a range of episode indices
/// and returns a vector of [`Video`] objects containing links to episodes within
/// the specified range. If the episode range is valid and episodes are found,
/// their corresponding links are formatted and added to the result vector.
///
/// # Arguments
///
/// * `anime_episodes` - An [`AnimeEpisodes`] struct containing information about all episodes of the anime.
/// * `range` - A range of episode indices (inclusive) for which to retrieve video links.
///
/// # Returns
///
/// A vector of [`Video`] objects containing links to the requested episodes.
pub fn get_episodes_link(
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

    // Generate video links for the filtered episodes
    for episode in episodes {
        // Format the video link using the episode ID
        videos.push(Video::new(format!(
            "{}/api/episode/serverPlayerAnimeWorld?id={}",
            animeworld::LINK,
            episode.episode_id
        )));
    }

    // Return a vec instance containing generated links
    videos
}
