/// Image moderation results
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Debug, Clone, PartialEq)]
pub struct ImageModeration {
    /// Whether the image is considered NSFW
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nsfw: Option<bool>,
    /// The most likely NSFW label
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nsfw_label: Option<String>,
    /// Score for the most likely NSFW label (0.0 - 1.0)
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nsfw_score: Option<f32>,
    /// Whether OCR detected profane text
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub profanity: Option<bool>,
    /// Matched profane terms
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub profanity_matches: Option<Vec<String>>,
}

/// File
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    /// Unique Id
    #[cfg_attr(feature = "serde", serde(rename = "_id"))]
    pub id: String,
    /// Tag / bucket this file was uploaded to
    pub tag: String,
    /// Original filename
    pub filename: String,
    /// Parsed metadata of this file
    pub metadata: Metadata,
    /// Raw content type of this file
    pub content_type: String,
    /// Size of this file (in bytes)
    pub size: isize,

    /// Image moderation results (if applicable)
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub image_moderation: Option<ImageModeration>,

    /// Whether this file was deleted
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub deleted: Option<bool>,
    /// Whether this file was reported
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub reported: Option<bool>,

    // TODO: migrate this mess to having:
    // - author_id
    // - parent: Parent { Message(id), User(id), etc }
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub message_id: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub user_id: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub server_id: Option<String>,

    /// Id of the object this file is associated with
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub object_id: Option<String>,
}

/// Metadata associated with a file
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Metadata {
    /// File is just a generic uncategorised file
    #[default]
    File,
    /// File contains textual data and should be displayed as such
    Text,
    /// File is an image with specific dimensions
    Image { width: usize, height: usize },
    /// File is a video with specific dimensions
    Video { width: usize, height: usize },
    /// File is audio
    Audio,
}
