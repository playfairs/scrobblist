use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_i64_from_number_or_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberOrString {
        Number(i64),
        String(String),
    }

    match NumberOrString::deserialize(deserializer)? {
        NumberOrString::Number(value) => Ok(value),
        NumberOrString::String(value) => value.parse::<i64>().map_err(serde::de::Error::custom),
    }
}

fn deserialize_u64_from_number_or_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberOrString {
        Number(u64),
        String(String),
    }

    match NumberOrString::deserialize(deserializer)? {
        NumberOrString::Number(value) => Ok(value),
        NumberOrString::String(value) => value.parse::<u64>().map_err(serde::de::Error::custom),
    }
}

fn deserialize_u32_from_number_or_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_u64_from_number_or_string(deserializer)
        .and_then(|value| u32::try_from(value).map_err(serde::de::Error::custom))
}

fn deserialize_optional_u32_from_number_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberOrString {
        Number(u32),
        String(String),
    }

    let value = Option::<NumberOrString>::deserialize(deserializer)?;
    value
        .map(|value| match value {
            NumberOrString::Number(number) => Ok(number),
            NumberOrString::String(string) => {
                string.parse::<u32>().map_err(serde::de::Error::custom)
            }
        })
        .transpose()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub realname: Option<String>,
    pub url: String,
    pub image: Vec<Image>,
    pub country: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_u32_from_number_or_string"
    )]
    pub age: Option<u32>,
    pub gender: Option<String>,
    #[serde(deserialize_with = "deserialize_u32_from_number_or_string")]
    pub subscriber: u32,
    #[serde(deserialize_with = "deserialize_u64_from_number_or_string")]
    pub playcount: u64,
    #[serde(deserialize_with = "deserialize_u32_from_number_or_string")]
    pub playlists: u32,
    #[serde(deserialize_with = "deserialize_u32_from_number_or_string")]
    pub bootstrap: u32,
    pub registered: Registered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registered {
    #[serde(
        rename = "unixtime",
        deserialize_with = "deserialize_i64_from_number_or_string"
    )]
    pub unixtime: i64,
    #[serde(rename = "#text")]
    pub text: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "#text")]
    pub text: String,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    #[serde(default)]
    pub artist: Artist,
    #[serde(default)]
    pub url: String,
    pub mbid: Option<String>,
    pub album: Option<Album>,
    pub image: Vec<Image>,
    #[serde(rename = "@attr")]
    pub attr: Option<TrackAttr>,
    pub date: Option<TrackDate>,
    pub streamable: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Artist {
    #[serde(default, rename = "#text", alias = "name")]
    pub name: String,
    pub mbid: Option<String>,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    #[serde(default)]
    pub artist: String,
    #[serde(default, rename = "#text")]
    pub text: String,
    pub mbid: Option<String>,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackAttr {
    pub nowplaying: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackDate {
    #[serde(deserialize_with = "deserialize_i64_from_number_or_string")]
    pub uts: i64,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTracksResponse {
    pub recenttracks: RecentTracks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTracks {
    #[serde(rename = "@attr")]
    pub attr: RecentTracksAttr,
    pub track: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTracksAttr {
    pub user: String,
    pub total: String,
    pub page: String,
    pub perPage: String,
    pub totalPages: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session: Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub key: String,
    pub subscriber: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastFmError {
    pub error: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopArtistsResponse {
    pub topartists: TopArtists,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopArtists {
    pub artist: Vec<TopArtist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopArtist {
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub image: Vec<Image>,
    #[serde(default)]
    pub playcount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistInfoResponse {
    pub artist: ArtistInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistInfo {
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub image: Vec<Image>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAlbumsResponse {
    pub topalbums: TopAlbums,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAlbums {
    pub album: Vec<TopAlbum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAlbum {
    pub name: String,
    pub artist: Artist,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub image: Vec<Image>,
    #[serde(default)]
    pub playcount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopTracksResponse {
    pub toptracks: TopTracks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopTracks {
    pub track: Vec<TopTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopTrack {
    pub name: String,
    pub artist: Artist,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub image: Vec<Image>,
    #[serde(default)]
    pub playcount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LovedTracksResponse {
    pub lovedtracks: LovedTracks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LovedTracks {
    #[serde(rename = "@attr")]
    pub attr: LovedTracksAttr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LovedTracksAttr {
    #[serde(default)]
    pub total: String,
}
