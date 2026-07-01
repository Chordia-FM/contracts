//! User-submitted metadata suggestions (the artist-page Report flow) + the admin review queue. Each
//! field a user suggests is stored and reviewed independently; approving one applies it to the
//! canonical entity exactly like an admin edit.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One field a user proposes a new value for. `value_text` carries name/bio/genres (genres as a
/// comma-separated list); `value_image_hash` carries an uploaded image/banner hash (from `POST
/// /v1/images`). Exactly one is set per `field`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SuggestionFieldInput {
    /// `image` | `banner` | `name` | `bio` | `genres`.
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_image_hash: Option<String>,
}

/// Body of `POST /v1/suggestions`: one Report-modal submission. Fans out into one stored, separately
/// reviewable suggestion per item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MetadataSuggestionInput {
    /// Only `artist` is supported today.
    pub entity_type: String,
    pub entity_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub items: Vec<SuggestionFieldInput>,
}

/// One pending field-suggestion in the admin review queue, with the entity's current value so the
/// admin can compare before approving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminSuggestion {
    pub id: Uuid,
    pub reporter_handle: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_name: String,
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_image_hash: Option<String>,
    /// The entity's current text value for this field (name/bio/genres), for before/after comparison.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_text: Option<String>,
    /// The entity's current image/banner hash for this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_image_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub created_at_ms: i64,
}
