//! How a library's Discord bot lays out its messages, per bot, editable from the dashboard.
//!
//! A message is a list of **blocks**. The library renders each block into Discord's components
//! from the live state (the track, the queue, who asked); the owner decides which blocks appear,
//! in what order, and with which options. The defaults here are the design the bot ships with,
//! so a bot with no saved layout looks exactly as it always did, and "reset" is these values.
//!
//! The rules a layout must keep to live here too, as [`ViewLayout::validate`], so the dashboard
//! and the library refuse the same things for the same reasons: Discord allows at most five
//! buttons per row and forty components per message, and a control must not appear twice in one
//! message (its id would clash).

use serde::{Deserialize, Serialize};

/// Which of the bot's messages a layout describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LayoutView {
    /// The public controller while a track plays.
    NowPlaying,
    /// The controller when nothing plays.
    Idle,
    /// The confirmation after `/play`.
    Queued,
    /// What the controller turns into when the bot leaves.
    Left,
}

impl LayoutView {
    pub const ALL: [LayoutView; 4] = [
        LayoutView::NowPlaying,
        LayoutView::Idle,
        LayoutView::Queued,
        LayoutView::Left,
    ];
}

/// What the small line under a header says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum HeaderSubtitle {
    /// "in #channel", where the bot is.
    Channel,
    /// The bot's name.
    Bot,
    /// Nothing.
    None,
}

/// Where a picture goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ArtPlacement {
    /// Small, beside the text.
    Thumbnail,
    /// Large, below the text.
    Gallery,
    /// Not shown.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SeparatorSpacing {
    Small,
    Large,
}

/// A control on the now-playing message. Each may appear once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ControlButton {
    Previous,
    PlayPause,
    Skip,
    Stop,
    Shuffle,
    Loop,
    VolumeDown,
    VolumeUp,
    Queue,
    Autoplay,
    Lyrics,
}

impl ControlButton {
    pub const ALL: [ControlButton; 11] = [
        ControlButton::Previous,
        ControlButton::PlayPause,
        ControlButton::Skip,
        ControlButton::Stop,
        ControlButton::Shuffle,
        ControlButton::Loop,
        ControlButton::VolumeDown,
        ControlButton::VolumeUp,
        ControlButton::Queue,
        ControlButton::Autoplay,
        ControlButton::Lyrics,
    ];
}

/// The small line under the progress bar: which facts it carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MetaLine {
    /// "Requested by @user", or "Autoplay".
    pub requested_by: bool,
    /// "3 in queue".
    pub queue: bool,
    /// "vol 80%", only when not 100.
    pub volume: bool,
    /// "loop: queue", "shuffle", "autoplay", each only when on.
    pub modes: bool,
}

impl Default for MetaLine {
    fn default() -> Self {
        Self {
            requested_by: true,
            queue: true,
            volume: true,
            modes: true,
        }
    }
}

/// One part of a message. Which kinds a view accepts is checked by [`ViewLayout::validate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LayoutBlock {
    /// The title line with the view's icon, and a subtitle under it.
    Header { subtitle: HeaderSubtitle },
    /// The track at hand: title, artist and album, with its art and the file's badges.
    Track {
        art: ArtPlacement,
        album: bool,
        badges: bool,
    },
    /// The progress bar with the times, and the meta line under it.
    Progress {
        /// Bar segments; four to twenty. Twelve is about the width of a title on a phone.
        cells: u8,
        times: bool,
        meta: MetaLine,
    },
    /// Rows of controls, up to five per row, each control at most once in the message.
    Controls { rows: Vec<Vec<ControlButton>> },
    /// A gap, with or without a line.
    Separator {
        divider: bool,
        spacing: SeparatorSpacing,
    },
    /// Your own words, Discord markdown, with variables: `{title}`, `{artist}`, `{album}`,
    /// `{channel}`, `{bot}`, `{requested_by}`, `{queue_count}`, `{volume}`, `{position}`,
    /// `{duration}`.
    Text { content: String },
    /// The toast's summary: what was added, where it sits in the queue, who asked.
    Summary { art: ArtPlacement },
}

impl LayoutBlock {
    /// The kind as it appears on the wire, for messages about it.
    pub fn kind(&self) -> &'static str {
        match self {
            LayoutBlock::Header { .. } => "header",
            LayoutBlock::Track { .. } => "track",
            LayoutBlock::Progress { .. } => "progress",
            LayoutBlock::Controls { .. } => "controls",
            LayoutBlock::Separator { .. } => "separator",
            LayoutBlock::Text { .. } => "text",
            LayoutBlock::Summary { .. } => "summary",
        }
    }
}

/// The blocks of one view, in order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ViewLayout {
    pub blocks: Vec<LayoutBlock>,
}

/// Every view's layout for one bot. A missing view means its default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BotLayouts {
    #[serde(default = "default_now_playing")]
    pub now_playing: ViewLayout,
    #[serde(default = "default_idle")]
    pub idle: ViewLayout,
    #[serde(default = "default_queued")]
    pub queued: ViewLayout,
    #[serde(default = "default_left")]
    pub left: ViewLayout,
}

impl Default for BotLayouts {
    fn default() -> Self {
        Self {
            now_playing: default_now_playing(),
            idle: default_idle(),
            queued: default_queued(),
            left: default_left(),
        }
    }
}

impl BotLayouts {
    pub fn view(&self, view: LayoutView) -> &ViewLayout {
        match view {
            LayoutView::NowPlaying => &self.now_playing,
            LayoutView::Idle => &self.idle,
            LayoutView::Queued => &self.queued,
            LayoutView::Left => &self.left,
        }
    }

    /// Every view's layout, or the first rule one of them breaks.
    pub fn validate(&self) -> Result<(), String> {
        for view in LayoutView::ALL {
            self.view(view)
                .validate(view)
                .map_err(|e| format!("{}: {e}", view_name(view)))?;
        }
        Ok(())
    }
}

fn view_name(view: LayoutView) -> &'static str {
    match view {
        LayoutView::NowPlaying => "now playing",
        LayoutView::Idle => "idle",
        LayoutView::Queued => "queued",
        LayoutView::Left => "left",
    }
}

/// The design the bot ships with: the controller as it has always looked.
pub fn default_now_playing() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            LayoutBlock::Header {
                subtitle: HeaderSubtitle::Channel,
            },
            LayoutBlock::Track {
                art: ArtPlacement::Thumbnail,
                album: true,
                badges: true,
            },
            LayoutBlock::Separator {
                divider: false,
                spacing: SeparatorSpacing::Small,
            },
            LayoutBlock::Progress {
                cells: 12,
                times: true,
                meta: MetaLine::default(),
            },
            LayoutBlock::Separator {
                divider: false,
                spacing: SeparatorSpacing::Large,
            },
            LayoutBlock::Controls {
                rows: vec![
                    vec![
                        ControlButton::Previous,
                        ControlButton::PlayPause,
                        ControlButton::Skip,
                        ControlButton::Stop,
                        ControlButton::Shuffle,
                    ],
                    vec![
                        ControlButton::Loop,
                        ControlButton::VolumeDown,
                        ControlButton::VolumeUp,
                        ControlButton::Queue,
                        ControlButton::Autoplay,
                    ],
                ],
            },
        ],
    }
}

pub fn default_idle() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            LayoutBlock::Header {
                subtitle: HeaderSubtitle::Channel,
            },
            LayoutBlock::Text {
                content: "-# The queue is empty. `/play` something.".to_string(),
            },
        ],
    }
}

pub fn default_queued() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            LayoutBlock::Header {
                subtitle: HeaderSubtitle::None,
            },
            LayoutBlock::Summary {
                art: ArtPlacement::Thumbnail,
            },
        ],
    }
}

pub fn default_left() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            LayoutBlock::Header {
                subtitle: HeaderSubtitle::Bot,
            },
            LayoutBlock::Text {
                content: "-# {reason} · `/play` to bring me back".to_string(),
            },
        ],
    }
}

/// The most blocks one view may hold. Well under Discord's forty components even when every
/// block is a section with a thumbnail.
pub const MAX_BLOCKS: usize = 16;
pub const MAX_ROWS: usize = 5;
pub const MAX_PER_ROW: usize = 5;
pub const MAX_TEXT_CHARS: usize = 2000;
pub const MIN_CELLS: u8 = 4;
pub const MAX_CELLS: u8 = 20;

impl ViewLayout {
    /// The rules, in the order a person would want to hear them.
    pub fn validate(&self, view: LayoutView) -> Result<(), String> {
        if self.blocks.is_empty() {
            return Err("a view needs at least one block".into());
        }
        if self.blocks.len() > MAX_BLOCKS {
            return Err(format!("at most {MAX_BLOCKS} blocks"));
        }
        let mut headers = 0;
        let mut rows = 0;
        let mut seen: Vec<ControlButton> = Vec::new();
        for block in &self.blocks {
            let allowed = match block {
                LayoutBlock::Header { .. }
                | LayoutBlock::Separator { .. }
                | LayoutBlock::Text { .. } => true,
                LayoutBlock::Track { .. }
                | LayoutBlock::Progress { .. }
                | LayoutBlock::Controls { .. } => view == LayoutView::NowPlaying,
                LayoutBlock::Summary { .. } => view == LayoutView::Queued,
            };
            if !allowed {
                return Err(format!(
                    "a {} block does not belong in the {} view",
                    block.kind(),
                    view_name(view)
                ));
            }
            match block {
                LayoutBlock::Header { .. } => {
                    headers += 1;
                    if headers > 1 {
                        return Err("one header per view".into());
                    }
                }
                LayoutBlock::Progress { cells, .. } => {
                    if !(MIN_CELLS..=MAX_CELLS).contains(cells) {
                        return Err(format!(
                            "the progress bar has {MIN_CELLS} to {MAX_CELLS} cells"
                        ));
                    }
                }
                LayoutBlock::Controls { rows: r } => {
                    for row in r {
                        rows += 1;
                        if rows > MAX_ROWS {
                            return Err(format!("at most {MAX_ROWS} rows of controls"));
                        }
                        if row.is_empty() {
                            return Err("a row of controls cannot be empty".into());
                        }
                        if row.len() > MAX_PER_ROW {
                            return Err(format!("at most {MAX_PER_ROW} controls in a row"));
                        }
                        for b in row {
                            if seen.contains(b) {
                                return Err(format!("{b:?} appears twice; each control once"));
                            }
                            seen.push(*b);
                        }
                    }
                }
                LayoutBlock::Text { content } => {
                    if content.trim().is_empty() {
                        return Err("a text block needs some text".into());
                    }
                    if content.chars().count() > MAX_TEXT_CHARS {
                        return Err(format!(
                            "a text block holds at most {MAX_TEXT_CHARS} characters"
                        ));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_valid_and_round_trip() {
        let layouts = BotLayouts::default();
        layouts.validate().unwrap();
        let json = serde_json::to_string(&layouts).unwrap();
        let back: BotLayouts = serde_json::from_str(&json).unwrap();
        assert_eq!(back, layouts);
        // A missing view falls back to its default.
        let partial: BotLayouts =
            serde_json::from_str(r#"{"idle":{"blocks":[{"kind":"header","subtitle":"bot"}]}}"#)
                .unwrap();
        assert_eq!(partial.now_playing, default_now_playing());
        assert_eq!(partial.idle.blocks.len(), 1);
    }

    #[test]
    fn the_rules_refuse_what_discord_would() {
        let mut v = default_now_playing();
        if let LayoutBlock::Controls { rows } = &mut v.blocks[5] {
            rows[0].push(ControlButton::Lyrics);
        }
        assert!(v
            .validate(LayoutView::NowPlaying)
            .unwrap_err()
            .contains("at most 5"));

        let mut v = default_now_playing();
        if let LayoutBlock::Controls { rows } = &mut v.blocks[5] {
            rows[1][0] = ControlButton::Skip;
        }
        assert!(v
            .validate(LayoutView::NowPlaying)
            .unwrap_err()
            .contains("twice"));

        let v = ViewLayout {
            blocks: vec![LayoutBlock::Track {
                art: ArtPlacement::None,
                album: true,
                badges: false,
            }],
        };
        assert!(v
            .validate(LayoutView::Idle)
            .unwrap_err()
            .contains("does not belong"));

        let v = ViewLayout {
            blocks: vec![LayoutBlock::Progress {
                cells: 40,
                times: true,
                meta: MetaLine::default(),
            }],
        };
        assert!(v.validate(LayoutView::NowPlaying).is_err());
        assert!(ViewLayout { blocks: vec![] }
            .validate(LayoutView::Left)
            .is_err());
    }
}
