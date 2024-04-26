//! This module contains functions for managing files, directories, and asynchronous file downloads.
//!
//! It includes functions for:
//! - Getting paths to various files and directories within the application directory.
//! - Creating the application directory and blacklist file if they don't exist.
//! - Asynchronously downloading required RTEN (Real-Time Entity Recognition) models.
//! - Asynchronously downloading files from URLs and saving them to specified paths.
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::paths::{
//!     download_rten_models,
//!     create_app_dir_and_blacklist_file
//! };
//!
//! async fn initialize_app() -> anyhow::Result<()> {
//!     // Ensure the app directory and blacklist file are created
//!     create_app_dir_and_blacklist_file()?;
//!     // Download required RTEN models asynchronously
//!     download_rten_models().await?;
//!     Ok(())
//! }
//! ```

use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use crate::blacklist;

/// The URL to report bugs and issues to.
pub(crate) const SUPPORT_URL: &str = "https://github.com/Hakxsorus/blitz/tree/master";

/// The download URL for the OCRS detection model.
const DETECTION_MODEL_URL: &str = "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten";

/// The download URL for the OCRS recognition model.
const RECOGNITION_MODEL_URL: &str = "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten";

/// The download URL for the application banner.
const BANNER_PNG_URL: &str = "https://i.imgur.com/6wno5lb.png";

/// The file name for the OCRS detection model.
const DETECTION_MODEL_FILE_NAME: &str = "text-detection.rten";

/// The file name for the OCRS recognition model.
const RECOGNITION_MODEL_FILE_NAME: &str = "text-recognition.rten";

/// The file name for the application banner.
const BANNER_PNG_FILE_NAME: &str = "banner.png";

/// Gets the [`PathBuf`] to the app directory.
pub(crate) fn app_dir_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home_dir_path| home_dir_path.join("blitz-app"))
}

/// Gets the [`PathBuf`] to the init file.
pub(crate) fn init_path() -> Option<PathBuf> {
    join_to_app_dir_path("init")
}

/// Gets the [`PathBuf`] to the blacklist file.
pub(crate) fn blacklist_path() -> Option<PathBuf> {
    join_to_app_dir_path("blacklist.json")
}

/// Gets the [`PathBuf`] to the screenshot file.
pub(crate) fn scrshot_path() -> Option<PathBuf> {
    join_to_app_dir_path("players.png")
}

/// Gets the [`PathBuf`] to a cropped screenshot file.
pub(crate) fn player_scrshot_path(n: i32) -> Option<PathBuf> {
    join_to_app_dir_path(format!("player-crop-{n}.png").as_str())
}

/// Gets the [`PathBuf`] to the detection model file.
pub(crate) fn detection_model_path() -> Option<PathBuf> {
    join_to_app_dir_path(DETECTION_MODEL_FILE_NAME)
}

/// Gets the [`PathBuf`] to the recognition model file.
pub(crate) fn recognition_model_path() -> Option<PathBuf> {
    join_to_app_dir_path(RECOGNITION_MODEL_FILE_NAME)
}

/// Gets the [`PathBuf`] to the application banner file.
pub(crate) fn banner_path() -> Option<PathBuf> {
    join_to_app_dir_path(BANNER_PNG_FILE_NAME)
}

/// Joins a file name to the app directory path and returns it as a [`PathBuf`].
///
/// # Arguments
/// * `filename` - The name of the file to join.
fn join_to_app_dir_path(filename: &str) -> Option<PathBuf> {
    app_dir_path().map(|app_dir_path| app_dir_path.join(&filename))
}

/// Creates the app directory if it does not exist.
pub(crate) fn create_app_dir() -> anyhow::Result<()> {
    let app_dir_path = app_dir_path().ok_or(anyhow::anyhow!("Unable to construct the app directory path"))?;
    std::fs::create_dir_all(app_dir_path)?;
    Ok(())
}

/// Creates the init marker file if it does not exist to the app directory.
pub(crate) fn create_init_file_if_not_exists() -> anyhow::Result<()> {
    let init_path = init_path().ok_or(anyhow::anyhow!("Unable to construct the init path."))?;
    if !init_path.exists() {
        std::fs::File::create(&init_path)?;
    }

    Ok(())
}

/// Creates the blacklist file (with default data) file if it does not exist to the app directory.
pub(crate) fn create_blacklist_file_if_not_exists() -> anyhow::Result<()> {
    let blacklist_path = blacklist_path().ok_or(anyhow::anyhow!("Unable construct the blacklist file path"))?;
    if !blacklist_path.exists() {
        let default_blacklist = blacklist::Blacklist::default();
        let default_blacklist_json = serde_json::to_string_pretty(&default_blacklist)?;
        let mut default_blacklist_file = std::fs::File::create(&blacklist_path)?;
        default_blacklist_file.write_all(&default_blacklist_json.as_ref())?;
    }

    Ok(())
}

/// Asynchronously downloads required RTEN (Real-Time Entity Recognition) models if they don't already
/// exist locally. This function downloads both the detection and recognition models used for real-time
/// entity recognition.
pub(crate) async fn download_rten_models() -> Result<(), Box<dyn Error>> {
    download_if_not_exists(DETECTION_MODEL_URL, DETECTION_MODEL_FILE_NAME).await?;
    download_if_not_exists(RECOGNITION_MODEL_URL, RECOGNITION_MODEL_FILE_NAME).await?;
    Ok(())
}

/// Asynchronously downloads the application banner if it doesn't already exist locally.
pub(crate) async fn download_banner_file() -> anyhow::Result<()> {
    Ok(download_if_not_exists(BANNER_PNG_URL, BANNER_PNG_FILE_NAME).await?)
}

/// Asynchronously downloads a file from the specified URL if it doesn't already exist locally to
/// the app directory.
///
/// # Parameters
/// * `url`: A string slice representing the URL from which to download the file.
/// * `filename`: A string slice representing the name of the file to be downloaded.
async fn download_if_not_exists(
    url: &str,
    filename: &str,
) -> anyhow::Result<()> {
    let file_path = join_to_app_dir_path(&filename).ok_or(anyhow::anyhow!("Unable to construct the download path."))?;
    if !file_path.exists() {
        download_file(&url, &file_path).await?;
    }

    Ok(())
}


/// Asynchronously downloads a file from the given URL and saves it to the specified path.
///
/// # Arguments
/// * `url`: A string slice representing the URL from which to download the file.
/// * `path`: A [`PathBuf`] representing the path where the downloaded file should be saved.
async fn download_file(
    url: &str,
    path: &PathBuf
) -> anyhow::Result<()> {
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let mut file = std::fs::File::create(path)?;
        let bytes = response.bytes().await?;
        std::io::copy(&mut bytes.as_ref(), &mut file)?;
    } else {
        response.error_for_status()?;
    }

    Ok(())
}