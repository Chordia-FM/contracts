//! How a library's Discord bot lays out its messages, per bot, written by the owner.
//!
//! A message is a list of Discord's own parts: text, a section with a picture or a button beside
//! it, a gallery of pictures, a separator, a row of buttons, and, on the list messages, the line
//! each entry is written with. Every piece of text is a **template**: Discord markdown with
//! variables like `{track.title}`, `{channel}` (a real mention), `{player.progress_bar:12}` or
//! `{emoji:listening}`, which the library fills in from the live state when it sends. The
//! variables and what they mean are the library's to define (it is the one that renders them);
//! the dashboard asks it for the list.
//!
//! The defaults here are the design the bot ships with, written in the same language, so a bot
//! with no saved layout looks exactly as it always did and "reset" is these values.
//!
//! The rules a layout must keep to live here too, as [`ViewLayout::validate`], so the dashboard
//! and the library refuse the same things for the same reasons: Discord allows at most five
//! buttons per row and forty components per message, a control must not appear twice in one
//! message (its id would clash), and a gallery holds at most ten pictures.

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
    /// `/queue`: a page of what is coming up.
    Queue,
    /// `/history`: what was heard.
    History,
}

impl LayoutView {
    pub const ALL: [LayoutView; 6] = [
        LayoutView::NowPlaying,
        LayoutView::Idle,
        LayoutView::Queued,
        LayoutView::Left,
        LayoutView::Queue,
        LayoutView::History,
    ];

    /// Views that show a list of entries and therefore need a [`LayoutBlock::List`].
    pub fn is_list(self) -> bool {
        matches!(self, LayoutView::Queue | LayoutView::History)
    }
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

/// Where a picture comes from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ImageSource {
    /// The track's cover from its tags.
    Cover,
    /// The artist's picture from the Hub, when there is one.
    Artist,
    /// The bot's own avatar.
    BotAvatar,
    /// Any address, a template like the texts.
    Url { url: String },
}

/// A button: one of the bot's controls, or a link with your own label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ButtonSpec {
    Control {
        control: ControlButton,
    },
    /// `url` is a template, so `{album_link}` opens the album's page on the web client.
    Link {
        label: String,
        url: String,
    },
}

/// What sits beside a section's text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum Accessory {
    Image { source: ImageSource },
    Button { button: ButtonSpec },
}

/// One part of a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LayoutBlock {
    /// Text: Discord markdown with variables.
    Text { content: String },
    /// Text with a picture or a button beside it.
    Section {
        content: String,
        accessory: Accessory,
    },
    /// One to ten pictures, large.
    Gallery { images: Vec<ImageSource> },
    /// A gap, with or without a line.
    Separator {
        divider: bool,
        spacing: SeparatorSpacing,
    },
    /// Up to five buttons side by side.
    Row { buttons: Vec<ButtonSpec> },
    /// On a list message, the line every entry is written with; `empty` shows when there are
    /// none. Paging buttons are added after it when the list is longer than a page.
    List {
        item: String,
        empty: String,
        page_size: u8,
    },
}

impl LayoutBlock {
    /// The kind as it appears on the wire, for messages about it.
    pub fn kind(&self) -> &'static str {
        match self {
            LayoutBlock::Text { .. } => "text",
            LayoutBlock::Section { .. } => "section",
            LayoutBlock::Gallery { .. } => "gallery",
            LayoutBlock::Separator { .. } => "separator",
            LayoutBlock::Row { .. } => "row",
            LayoutBlock::List { .. } => "list",
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
    #[serde(default = "default_queue")]
    pub queue: ViewLayout,
    #[serde(default = "default_history")]
    pub history: ViewLayout,
}

impl Default for BotLayouts {
    fn default() -> Self {
        Self {
            now_playing: default_now_playing(),
            idle: default_idle(),
            queued: default_queued(),
            left: default_left(),
            queue: default_queue(),
            history: default_history(),
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
            LayoutView::Queue => &self.queue,
            LayoutView::History => &self.history,
        }
    }

    pub fn view_mut(&mut self, view: LayoutView) -> &mut ViewLayout {
        match view {
            LayoutView::NowPlaying => &mut self.now_playing,
            LayoutView::Idle => &mut self.idle,
            LayoutView::Queued => &mut self.queued,
            LayoutView::Left => &mut self.left,
            LayoutView::Queue => &mut self.queue,
            LayoutView::History => &mut self.history,
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
        LayoutView::Queue => "queue",
        LayoutView::History => "history",
    }
}

fn text(s: &str) -> LayoutBlock {
    LayoutBlock::Text {
        content: s.to_string(),
    }
}

fn separator(divider: bool, spacing: SeparatorSpacing) -> LayoutBlock {
    LayoutBlock::Separator { divider, spacing }
}

fn controls(buttons: &[ControlButton]) -> LayoutBlock {
    LayoutBlock::Row {
        buttons: buttons
            .iter()
            .map(|c| ButtonSpec::Control { control: *c })
            .collect(),
    }
}

/// The design the bot ships with: the controller as it has always looked, in the template
/// language, so it is also a worked example of the variables.
pub fn default_now_playing() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            text("### {icon} {heading}\n-# in {emoji:listening} {channel}"),
            separator(true, SeparatorSpacing::Small),
            LayoutBlock::Section {
                content: "{track}\n-# {file}".to_string(),
                accessory: Accessory::Image {
                    source: ImageSource::Cover,
                },
            },
            separator(false, SeparatorSpacing::Small),
            text("{player.progress_bar:12} {player.position} / {track.duration}\n-# {player.meta}"),
            separator(false, SeparatorSpacing::Large),
            controls(&[
                ControlButton::Previous,
                ControlButton::PlayPause,
                ControlButton::Skip,
                ControlButton::Stop,
                ControlButton::Shuffle,
            ]),
            controls(&[
                ControlButton::Loop,
                ControlButton::VolumeDown,
                ControlButton::VolumeUp,
                ControlButton::Queue,
                ControlButton::Autoplay,
            ]),
        ],
    }
}

pub fn default_idle() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            text("### {icon} {heading}\n-# in {emoji:listening} {channel}"),
            separator(true, SeparatorSpacing::Small),
            text("-# The queue is empty. `/play` something."),
        ],
    }
}

pub fn default_queued() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            text("### {icon} {heading}"),
            separator(true, SeparatorSpacing::Small),
            LayoutBlock::Section {
                content: "{added}\n-# {added.meta}".to_string(),
                accessory: Accessory::Image {
                    source: ImageSource::Cover,
                },
            },
        ],
    }
}

pub fn default_left() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            text("### {icon} {heading}\n-# {bot}"),
            separator(true, SeparatorSpacing::Small),
            text("-# {left.reason} · `/play` to bring me back"),
        ],
    }
}

pub fn default_queue() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            text("### {icon} {heading}\n-# {queue.tracks} · {queue.duration} · {bot}"),
            separator(true, SeparatorSpacing::Small),
            text("{player.line}"),
            separator(false, SeparatorSpacing::Small),
            LayoutBlock::List {
                item: "`{index}.` {track.line} · {track.duration} · {requester}".to_string(),
                empty: "-# The queue is empty.".to_string(),
                page_size: 10,
            },
        ],
    }
}

pub fn default_history() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            text("### {icon} {heading}\n-# {bot}"),
            separator(true, SeparatorSpacing::Small),
            LayoutBlock::List {
                item: "{track.line}\n-# {play.at} · {play.length} · {requester} · {play.counted}"
                    .to_string(),
                empty: "-# Nothing has played yet.".to_string(),
                page_size: 10,
            },
        ],
    }
}

/// A server's own versions of some of the bot's messages. A view that is absent means "the
/// bot's". Written by the library owner from the dashboard, per server, in the same language.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LayoutOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub now_playing: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<ViewLayout>,
}

impl LayoutOverrides {
    pub fn view(&self, view: LayoutView) -> Option<&ViewLayout> {
        match view {
            LayoutView::NowPlaying => self.now_playing.as_ref(),
            LayoutView::Idle => self.idle.as_ref(),
            LayoutView::Queued => self.queued.as_ref(),
            LayoutView::Left => self.left.as_ref(),
            LayoutView::Queue => self.queue.as_ref(),
            LayoutView::History => self.history.as_ref(),
        }
    }

    pub fn is_empty(&self) -> bool {
        LayoutView::ALL.iter().all(|v| self.view(*v).is_none())
    }

    /// Every present view, or the first rule one of them breaks.
    pub fn validate(&self) -> Result<(), String> {
        for view in LayoutView::ALL {
            if let Some(l) = self.view(view) {
                l.validate(view)
                    .map_err(|e| format!("{}: {e}", view_name(view)))?;
            }
        }
        Ok(())
    }
}

impl BotLayouts {
    /// The bot's layouts with a server's own versions laid over them.
    pub fn with_overrides(&self, overrides: &LayoutOverrides) -> BotLayouts {
        let mut out = self.clone();
        for view in LayoutView::ALL {
            if let Some(l) = overrides.view(view) {
                *out.view_mut(view) = l.clone();
            }
        }
        out
    }
}

/// The most blocks one view may hold. Well under Discord's forty components even when every
/// block is a section with a thumbnail and the list adds its paging row.
pub const MAX_BLOCKS: usize = 20;
pub const MAX_ROWS: usize = 5;
pub const MAX_PER_ROW: usize = 5;
pub const MAX_GALLERY: usize = 10;
pub const MAX_TEXT_CHARS: usize = 2000;
pub const MAX_ITEM_CHARS: usize = 400;
pub const MAX_LABEL_CHARS: usize = 80;
pub const MIN_PAGE: u8 = 3;
pub const MAX_PAGE: u8 = 25;

fn check_text(what: &str, s: &str, max: usize) -> Result<(), String> {
    if s.trim().is_empty() {
        return Err(format!("{what} needs some text"));
    }
    if s.chars().count() > max {
        return Err(format!("{what} holds at most {max} characters"));
    }
    Ok(())
}

fn check_url(url: &str) -> Result<(), String> {
    let u = url.trim();
    if u.starts_with("http://") || u.starts_with("https://") || u.starts_with('{') {
        Ok(())
    } else {
        Err("a link needs an http(s) address or a variable".into())
    }
}

fn check_button(
    view: LayoutView,
    b: &ButtonSpec,
    seen: &mut Vec<ControlButton>,
) -> Result<(), String> {
    match b {
        ButtonSpec::Control { control } => {
            if view != LayoutView::NowPlaying {
                return Err("the bot's controls belong on the now-playing message".into());
            }
            if seen.contains(control) {
                return Err(format!("{control:?} appears twice; each control once"));
            }
            seen.push(*control);
            Ok(())
        }
        ButtonSpec::Link { label, url } => {
            check_text("a link button's label", label, MAX_LABEL_CHARS)?;
            check_url(url)
        }
    }
}

fn check_image(source: &ImageSource) -> Result<(), String> {
    match source {
        ImageSource::Url { url } => check_url(url),
        _ => Ok(()),
    }
}

impl ViewLayout {
    /// The rules, in the order a person would want to hear them.
    pub fn validate(&self, view: LayoutView) -> Result<(), String> {
        if self.blocks.is_empty() {
            return Err("a message needs at least one block".into());
        }
        if self.blocks.len() > MAX_BLOCKS {
            return Err(format!("at most {MAX_BLOCKS} blocks"));
        }
        let mut rows = 0;
        let mut lists = 0;
        let mut seen: Vec<ControlButton> = Vec::new();
        for block in &self.blocks {
            match block {
                LayoutBlock::Text { content } => {
                    check_text("a text block", content, MAX_TEXT_CHARS)?
                }
                LayoutBlock::Section { content, accessory } => {
                    check_text("a section", content, MAX_TEXT_CHARS)?;
                    match accessory {
                        Accessory::Image { source } => check_image(source)?,
                        Accessory::Button { button } => {
                            rows += 1;
                            check_button(view, button, &mut seen)?;
                        }
                    }
                }
                LayoutBlock::Gallery { images } => {
                    if images.is_empty() || images.len() > MAX_GALLERY {
                        return Err(format!("a gallery holds one to {MAX_GALLERY} pictures"));
                    }
                    for i in images {
                        check_image(i)?;
                    }
                }
                LayoutBlock::Separator { .. } => {}
                LayoutBlock::Row { buttons } => {
                    rows += 1;
                    if rows > MAX_ROWS {
                        return Err(format!("at most {MAX_ROWS} rows of buttons"));
                    }
                    if buttons.is_empty() {
                        return Err("a row needs at least one button".into());
                    }
                    if buttons.len() > MAX_PER_ROW {
                        return Err(format!("at most {MAX_PER_ROW} buttons in a row"));
                    }
                    for b in buttons {
                        check_button(view, b, &mut seen)?;
                    }
                }
                LayoutBlock::List {
                    item,
                    empty,
                    page_size,
                } => {
                    if !view.is_list() {
                        return Err(format!(
                            "a list block does not belong in the {} message",
                            view_name(view)
                        ));
                    }
                    lists += 1;
                    if lists > 1 {
                        return Err("one list per message".into());
                    }
                    check_text("the list's line", item, MAX_ITEM_CHARS)?;
                    check_text("the list's empty text", empty, MAX_TEXT_CHARS)?;
                    if !(MIN_PAGE..=MAX_PAGE).contains(page_size) {
                        return Err(format!("a page holds {MIN_PAGE} to {MAX_PAGE} entries"));
                    }
                }
            }
        }
        if view.is_list() && lists == 0 {
            return Err(format!(
                "the {} message needs a list block",
                view_name(view)
            ));
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
            serde_json::from_str(r#"{"idle":{"blocks":[{"kind":"text","content":"hi"}]}}"#)
                .unwrap();
        assert_eq!(partial.now_playing, default_now_playing());
        assert_eq!(partial.idle.blocks.len(), 1);
    }

    #[test]
    fn the_rules_refuse_what_discord_would() {
        let mut v = default_now_playing();
        if let LayoutBlock::Row { buttons } = &mut v.blocks[6] {
            buttons.push(ButtonSpec::Control {
                control: ControlButton::Lyrics,
            });
        }
        assert!(v
            .validate(LayoutView::NowPlaying)
            .unwrap_err()
            .contains("at most 5"));

        let mut v = default_now_playing();
        if let LayoutBlock::Row { buttons } = &mut v.blocks[7] {
            buttons[0] = ButtonSpec::Control {
                control: ControlButton::Skip,
            };
        }
        assert!(v
            .validate(LayoutView::NowPlaying)
            .unwrap_err()
            .contains("twice"));

        let v = ViewLayout {
            blocks: vec![controls(&[ControlButton::PlayPause])],
        };
        assert!(v
            .validate(LayoutView::Idle)
            .unwrap_err()
            .contains("now-playing"));

        let v = ViewLayout {
            blocks: vec![text("### Queue")],
        };
        assert!(v
            .validate(LayoutView::Queue)
            .unwrap_err()
            .contains("needs a list"));

        let v = ViewLayout {
            blocks: vec![LayoutBlock::Row {
                buttons: vec![ButtonSpec::Link {
                    label: "Open".into(),
                    url: "javascript:alert(1)".into(),
                }],
            }],
        };
        assert!(v.validate(LayoutView::Left).unwrap_err().contains("http"));
        assert!(ViewLayout { blocks: vec![] }
            .validate(LayoutView::Left)
            .is_err());
    }
}
