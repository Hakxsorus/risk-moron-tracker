//! This module provides structures and methods for managing a blacklist of users.
//!
//! The [`Blacklist`] struct represents a list of blacklisted users, where each user is represented
//! by a [`Moron`] struct containing their username and the reason for blacklisting.
//!
//! # Examples
//!
//! ```rust
//! use crate::blacklist::{Blacklist, Moron};
//!
//! fn main() -> anyhow::Result<()> {
//!     // Load existing blacklist from file
//!     let blacklist_path = std::path::PathBuf::from("blacklist.json");
//!     let blacklist = Blacklist::load(&blacklist_path)?;
//!
//!     // Add a new moron to the blacklist
//!     let new_moron = Moron {
//!         username: String::from("New Moron"),
//!         reason: String::from("Repeated spamming"),
//!     };
//!     blacklist.add_moron(new_moron);
//!
//!     // Save the updated blacklist to file
//!     blacklist.save(&blacklist_path)?;
//!
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Blacklist containing a list of [`Moron`].
#[derive(Serialize, Deserialize, Debug)]
pub struct Blacklist {
    /// The list of blacklisted morons.
    pub morons: Vec<Moron>,
}

/// A blacklisted moron.
#[derive(Serialize, Deserialize, Debug)]
pub struct Moron {
    /// The moron's username.
    pub username: String,
    /// Why the moron is blacklisted.
    pub reason: String
}

impl Blacklist {
    /// Loads and deserializes an existing [`Blacklist`] JSON file into a new [`Blacklist`].
    ///
    /// # Arguments
    /// * `blacklist_path` - A reference to the [`PathBuf`] representing the path to the blacklist file.
    pub fn load(blacklist_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(blacklist_path)?;
        let blacklist: Blacklist = serde_json::from_str(&content)?;
        Ok(blacklist)
    }
}

impl Default for Blacklist {
    /// Creates a new [`Blacklist`] that contains a two example entries.
    fn default() -> Self {
        Blacklist {
            morons: vec![Moron {
                username: String::from("Example User #1"),
                reason: "Copy and paste the { } block to add more entries".to_string(),
            }, Moron {
                username: String::from("Example User #2"),
                reason: "Don't forget the comma at the end of the block.".to_string(),
            }]
        }
    }
}