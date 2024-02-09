pub mod animeunity;
pub mod animeworld;
pub mod aniplay;

/// Enumeration of different anime streaming sites.
#[derive(Debug)]
pub enum Sites {
    AnimeWorld,
    AnimeUnity,
    AniPlay,
}

#[derive(Debug)]
pub enum AnimeState {
    InCorso,
    Finito,
    NonValido,
}

#[derive(Debug)]
pub struct AnimeInfo {
    name: String,
    year: String,
    state: AnimeState,
    description: String,
    genres: Vec<String>,
    studio: String,
    stars: String,
    cover: String,
    cover_full: String,
    banner: String,
}

impl AnimeInfo {
    pub fn new(
        name: String,
        year: String,
        state: AnimeState,
        description: String,
        genres: Vec<String>,
        studio: String,
        stars: String,
        cover: String,
        cover_full: String,
        banner: String,
    ) -> Self {
        Self {
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
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_year(&self) -> &String {
        &self.year
    }

    pub fn get_description(&self) -> &String {
        &self.description
    }

    pub fn get_genres(&self) -> &Vec<String> {
        &self.genres
    }

    pub fn get_cover(&self) -> &String {
        &self.cover
    }

    pub fn get_cover_full(&self) -> &String {
        &self.cover_full
    }

    pub fn get_banner(&self) -> &String {
        &self.banner
    }

    pub fn get_state(&self) -> &AnimeState {
        &self.state
    }

    pub fn get_stars(&self) -> &String {
        &self.stars
    }

    pub fn get_studio(&self) -> &String {
        &self.studio
    }
}

/// Struct representing an anime.
#[derive(Debug)]
pub struct Anime {
    site: Sites,
    link: String,
    info: AnimeInfo,
}

impl Anime {
    /// Constructs a new Anime instance.
    ///
    /// # Arguments
    ///
    /// * `site` - The site where the anime is available.
    /// * `name` - The name of the anime.
    /// * `link` - The link to the anime's page.
    pub fn new(site: Sites, link: String, info: AnimeInfo) -> Self {
        Self { site, link, info }
    }

    pub fn get_site(&self) -> &Sites {
        &self.site
    }

    pub fn get_info(&self) -> &AnimeInfo {
        &self.info
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

    pub fn get_number(&self) -> usize {
        self.number
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

    pub fn get_site(&self) -> &Sites {
        &self.site
    }

    pub fn get_episodes(&self) -> &Vec<Episode> {
        &self.episodes
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

    pub fn get_link(&self) -> &String {
        &self.link
    }
}
