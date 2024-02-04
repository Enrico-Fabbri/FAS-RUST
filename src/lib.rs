// TODO: add download option for episodes

pub mod animeunity;
pub mod animeworld;

/// Enumeration of different anime streaming sites.
#[derive(Debug)]
pub enum Sites {
    AnimeWorld,
    AnimeUnity,
    AnimeSaturn,
    AnimeItaly,
}

/// Struct representing an anime.
#[derive(Debug)]
pub struct Anime {
    site: Sites,
    name: String,
    link: String,
}

impl Anime {
    /// Constructs a new Anime instance.
    ///
    /// # Arguments
    ///
    /// * `site` - The site where the anime is available.
    /// * `name` - The name of the anime.
    /// * `link` - The link to the anime's page.
    pub fn new(site: Sites, name: String, link: String) -> Self {
        Self { site, name, link }
    }
}

/// Struct representing an episode of an anime.
#[derive(Debug)]
pub struct Episode {
    number: usize,
    episode_id: String,
}

impl Episode {
    /// Constructs a new Episode instance.
    ///
    /// # Arguments
    ///
    /// * `number` - The episode number.
    /// * `episode_id` - The unique identifier of the episode.
    pub fn new(number: usize, episode_id: String) -> Self {
        Self { number, episode_id }
    }
}

/// Struct representing episodes of an anime from a specific site.
#[derive(Debug)]
pub struct AnimeEpisodes {
    site: Sites,
    link: String,
    episodes: Vec<Episode>,
}

impl AnimeEpisodes {
    /// Constructs a new AnimeEpisodes instance.
    ///
    /// # Arguments
    ///
    /// * `site` - The site where the anime is available.
    /// * `link` - The link to the anime's page.
    /// * `episodes` - A vector containing episodes of the anime.
    pub fn new(site: Sites, link: String, episodes: Vec<Episode>) -> Self {
        Self {
            site,
            link,
            episodes,
        }
    }
}

/// Struct representing a video link.
#[derive(Debug)]
pub struct Video {
    link: String,
}

impl Video {
    pub fn new(link: String) -> Self {
        Self { link }
    }
}
