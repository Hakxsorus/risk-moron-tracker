//! Module for scanning the RISK lobby for players and determining whether they are likely blacklisted.
//!
//! The [`scan`] function performs the following steps:
//! 1. Finds the RISK window from all active windows.
//! 2. Screenshots and crops the player cards from the RISK window.
//! 3. Creates an OCR engine, loads the images, and extracts the text.
//! 4. Loads the blacklist.
//! 5. Fuzzy matches the detections against the blacklist.
//!
//! The module also contains utility functions for capturing screenshots, cropping player cards,
//! creating an OCR engine, and detecting text from images.
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::detector;
//!
//! # async fn example_usage() -> anyhow::Result<()> {
//! let scans = detector::scan()?;
//! for scan_info in scans {
//!     println!("Username: {}, Score: {}", scan_info.username, scan_info.score);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Warning
//!
//! - The [`crop_player_cards_1920_1080`] function assumes the dimension of the screenshot is 1920x1080 pixels.
//!   Adjustments might be necessary for different monitor aspect ratios.
//! - The `crop_and_save_player_cards_dynamic` function is a placeholder and not yet implemented.
//!

use std::path::PathBuf;
use std::result::Result::Ok;
use anyhow::bail;
use iced::subscription;
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use rten_tensor::AsView;
use xcap::Window;
use crate::{blacklist, paths};

#[derive(Debug)]
/// Information about a scan result, including the detected username and the matching score.
pub(crate) struct ScanInfo {
    /// The likely username match detected during the scan.
    pub username: String,
    /// The matching similarity between the detected text and the username in the blacklist.
    ///
    /// This similarity represents the degree of similarity between the detected text and the username
    /// in the blacklist. Higher similarities indicate stronger matches.
    pub similarity: u8,
}

/// Scans the RISK lobby for players and determines whether they are likely blacklisted.
pub fn scan() -> anyhow::Result<Vec<ScanInfo>> {
    let blacklist_path = paths::blacklist_path().ok_or(anyhow::anyhow!("Unable to construct blacklist path."))?;
    let blacklist = match blacklist::Blacklist::load(&blacklist_path) {
        Ok(blacklist) => blacklist,
        Err(err) => bail!(format!("Blacklist Error: {}", err.to_string()))
    };

    let risk_window = risk_window().ok_or(anyhow::anyhow!("Unable to find RISK window."))?;

    let scrshot_path = paths::scrshot_path()
        .ok_or(anyhow::anyhow!("Unable to construct screenshot path."))?;

    scrshot_window(&risk_window, &scrshot_path)?;
    crop_player_cards_1920_1080(&scrshot_path)?;

    let engine = create_ocr_engine()?;
    let mut detections: Vec<String> = Vec::new();
    for i in 0..6 {
        let player_scrshot_path = paths::player_scrshot_path(i)
            .ok_or(anyhow::anyhow!("Unable to construct player screenshot path."))?;

        dbg!(&player_scrshot_path);

        let text = detect_text(&engine, &player_scrshot_path)?;
        detections.extend(text);
    }

    let mut scans: Vec<ScanInfo> = Vec::new();
    for detection_text in detections.iter()
    {
        let detection_text_normalised = normalize(&detection_text);
        if detection_text_normalised.len() <= 1 {
            continue;
        }

        for moron in blacklist.morons.iter() {
            // 5.2. Ensure the moron's username is in lowercase.
            let username_normalised = normalize(&moron.username);
            let similarity = fuzzywuzzy::fuzz::ratio(&detection_text_normalised, &username_normalised);
            scans.push(ScanInfo {
                username: String::from(&moron.username),
                similarity: similarity
            });
        }
    }

    dbg!(&scans);

    Ok(scans)
}

/// Retrieves the window representing the game "RISK", if it exists.
pub(crate) fn risk_window() -> Option<Window> {
    let active_windows_result = xcap::Window::all();
    match active_windows_result {
        Ok(active_windows) => {
            active_windows
                .iter()
                .find(|w| w.title() == "RISK")
                .cloned()
        },
        Err(_) => None,
    }
}


/// Captures a screenshot of the specified window and saves it to a file.
///
/// # Arguments
/// * `window`: A reference to the [`xcap::Window`] to capture the screenshot from.
/// * `path`: A reference to the [`PathBuf`] representing the path where the screenshot will be saved.
pub(crate) fn scrshot_window(
    window: &xcap::Window,
    path: &PathBuf
) -> anyhow::Result<()> {
    let image = window.capture_image()?;
    Ok(image.save(&path)?)
}

/// Crops the player cards from the screenshot image and saves them individually to the app directory
/// with an indexed file name.
///
/// # Arguments
/// * `scrshot_path`: A reference to the [`PathBuf`] representing the path to the screenshot image to crop.
///
/// # Warning
/// This method assumes the dimension is 1920x1080px. Otherwise, it will not work.
pub(crate) fn crop_player_cards_1920_1080(scrshot_path: &PathBuf) -> anyhow::Result<()> {
    // Crop the surrounding space out of the player list.
    // =============================
    // ||| [Player 1] [Player 2] |||
    // ||| [Player 3] [Player 4] |||
    // ||| [Player 5] [Player 6] |||
    // =============================
    let mut image = image::io::Reader::open(&scrshot_path)?.decode()?;
    let player_list_width = 1200;
    let player_list_height = 550;
    let player_list_start_x = (image.width() - player_list_width) / 2;
    let player_list_start_y = (image.height() - player_list_height) / 2;
    let player_list_image = image.crop(
        player_list_start_x,
        player_list_start_y,
        player_list_width,
        player_list_height
    );
    // Crop the individual players cards out of the player list.
    // [Player 1] [Player 2]
    // [Player 3] [Player 4]
    // [Player 5] [Player 6]
    let player_card_width = 600;
    let player_card_height = 180;
    for row in 0..3 {
        for col in 0..2 {
            let player_card_start_x = col * player_card_width;
            let player_card_start_y = row * player_card_height;
            let player_card_image = player_list_image.clone().crop(
                player_card_start_x,
                player_card_start_y,
                player_card_width,
                player_card_height);
            let player_card_index = (row * 2 + col) as i32;
            let player_scrshot_path = paths::player_scrshot_path(player_card_index)
                .ok_or(anyhow::anyhow!("Unable to construct player screenshot path."))?;
            player_card_image.save(player_scrshot_path)?;
        }
    }

    Ok(())
}

/// Crops the player cards from the screenshot image adjusting for various monitor aspect ratios and
/// saves them individually to the app directory with an indexed file name.
///
/// # Arguments
/// * `scrshot_path`: A reference to the [`PathBuf`] representing the path to the screenshot image to crop.
fn crop_and_save_player_cards_dynamic(scrshot_path: &PathBuf) -> anyhow::Result<()> {
    todo!()
}

/// Creates an OCR engine using the detection and recognition models from the app directory.
pub(crate) fn create_ocr_engine() -> anyhow::Result<OcrEngine> {
    // Get the paths to the detection and recognition models
    let detection_model_path = paths::detection_model_path()
        .ok_or(anyhow::anyhow!("Unable to construct detection model path."))?;
    let recognition_model_path = paths::recognition_model_path()
        .ok_or(anyhow::anyhow!("Unable to construct recognition model path."))?;
    // Read the model data from the files
    let detection_model_data = std::fs::read(&detection_model_path)?;
    let recognition_model_data = std::fs::read(&recognition_model_path)?;
    // Load the detection and recognition models
    let detection_model = Model::load(&detection_model_data)?;
    let recognition_model = Model::load(&recognition_model_data)?;
    // Create an OCR engine using the loaded models
    let ocr_engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        debug: false,
        decode_method: Default::default(),
    })?;

    Ok(ocr_engine)
}

/// Detects text (transformed to lowercase) from an image using the provided OCR engine.
///
/// # Arguments
/// * `ocr_engine`: A reference to the OCR engine ([`OcrEngine`]) used for text detection.
/// * `scrshot_path`: A reference to the [`PathBuf`] representing the path to the image.
pub(crate) fn detect_text(
    ocr_engine: &OcrEngine,
    image_path: &PathBuf
) -> anyhow::Result<Vec<String>> {
    // Detect the text from the image.
    let image = rten_imageio::read_image(&image_path.display().to_string())
        .expect("Unable to read image into the tensor.");
    let ocr_input = ocr_engine.prepare_input(image.view())?;
    let text = ocr_engine.get_text(&ocr_input)?;
    // Split it on newlines to get an array of detected text chunks.
    Ok(text.split('\n')
        .map(|s| normalize(&s))
        .collect()
    )
}

/// Normalizes a string by converting it to lowercase and removing spaces.
///
/// # Arguments
/// * `input` - A reference to the input string that needs to be normalized.
fn normalize(input: &str) -> String {
    let prefix = "General ";
    let normalized_without_prefix = if input.starts_with(prefix) {
        &input[prefix.len()..]
    } else {
        input
    };

    normalized_without_prefix.to_lowercase().replace(" ", "")
}
