use once_cell::sync::Lazy;
use reqwest::Client;
use semver::Version;
use serde::Deserialize;

static GITHUB_API_URL: &str = "https://api.github.com/repos/titanom/bwenv/releases/latest";

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
}

pub async fn fetch_latest_version() -> Result<Version, Box<dyn std::error::Error>> {
    println!("did fetch --------------------------");
    let response = CLIENT
        .get(GITHUB_API_URL)
        .header("User-Agent", "bwenv")
        .send()
        .await?
        .json::<GithubRelease>()
        .await?;

    Ok(Version::parse(&response.tag_name.replace("v", ""))?)
}
