//! Canonical pipeline definition.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Import,
    Fingerprint,
    MatchTrack,
    MatchAlbum,
    TagTrack,
    Index,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write the same snake-case strings Serde uses
        write!(f, "{}", serde_json::to_string(self).unwrap().trim_matches('"'))
    }
}

impl Stage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Stage::Import      => "import",
            Stage::Fingerprint => "fingerprint",
            Stage::MatchTrack  => "match_track",
            Stage::MatchAlbum  => "match_album",
            Stage::TagTrack    => "tag_track",
            Stage::Index       => "index",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobEnvelope {
    pub album_id: Option<Uuid>,
    pub track_id: Option<Uuid>,
    pub file_id:  Option<Uuid>,
    pub stage:    Stage,
}

