//! How a library's Discord bot lays out its messages, per bot, written by the owner.
//!
//! A message is a list of Discord's own parts: text, a section with a picture or a button beside
//! it, a gallery of pictures, a separator, a row of buttons, a container (the box with the accent
//! bar, holding any of the others) and, on the list messages, the line each entry is written
//! with. A container is a part like any other: a message may have none, one, or several, and each
//! takes the bot's colour, a colour of its own, or no bar. Every piece of text is a **template**: Discord markdown with
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
//! buttons per row and forty components per message, a container cannot hold another, a control
//! must not appear twice in one message (its id would clash), and a gallery holds at most ten
//! pictures.
//!
//! [`LAYOUT_VERSION`] stamps what is saved, so the library can bring an older layout up to date
//! (renamed variables, the container that used to be implied) exactly once.

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
    /// `/play` of an album: the toast for the tracks added.
    QueuedAlbum,
    /// `/play` of an artist: the toast for the tracks added.
    QueuedArtist,
    /// `/playlist`: the toast for the tracks added from a Chordia playlist.
    QueuedPlaylist,
    /// `/lyrics`: a page of the current track's lyrics.
    Lyrics,
    /// A command did what it was asked: "Skipped", "Paused", "Volume".
    Done,
    /// A heads-up rather than a failure: "Nothing is playing", "Join a voice channel first".
    Notice,
    /// Something went wrong.
    Error,
    /// A listener voted to skip: the tally so far.
    Vote,
    /// The vote passed and the track was skipped.
    VotePassed,
}

impl LayoutView {
    pub const ALL: [LayoutView; 15] = [
        LayoutView::NowPlaying,
        LayoutView::Idle,
        LayoutView::Queued,
        LayoutView::Left,
        LayoutView::Queue,
        LayoutView::History,
        LayoutView::QueuedAlbum,
        LayoutView::QueuedArtist,
        LayoutView::QueuedPlaylist,
        LayoutView::Lyrics,
        LayoutView::Done,
        LayoutView::Notice,
        LayoutView::Error,
        LayoutView::Vote,
        LayoutView::VotePassed,
    ];

    /// Views that show a list of entries and therefore need a [`LayoutBlock::List`].
    pub fn is_list(self) -> bool {
        matches!(self, LayoutView::Queue | LayoutView::History)
    }

    /// Views that turn pages, where a [`LayoutBlock::Pager`] may go.
    pub fn is_paged(self) -> bool {
        self.is_list() || matches!(self, LayoutView::Lyrics)
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

/// One of the bot's controls, as a button. Each may appear once on a message; they work on any
/// message, and one pressed on a queue, history or lyrics page redraws that page.
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
    Mute,
    SeekBack,
    SeekForward,
    Queue,
    Clear,
    Autoplay,
    Lyrics,
    History,
    Leave,
    /// Opens the equalizer panel.
    Equalizer,
}

impl ControlButton {
    pub const ALL: [ControlButton; 18] = [
        ControlButton::Previous,
        ControlButton::PlayPause,
        ControlButton::Skip,
        ControlButton::Stop,
        ControlButton::Shuffle,
        ControlButton::Loop,
        ControlButton::VolumeDown,
        ControlButton::VolumeUp,
        ControlButton::Mute,
        ControlButton::SeekBack,
        ControlButton::SeekForward,
        ControlButton::Queue,
        ControlButton::Clear,
        ControlButton::Autoplay,
        ControlButton::Lyrics,
        ControlButton::History,
        ControlButton::Leave,
        ControlButton::Equalizer,
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
    /// The artist's wide banner from the Hub, when there is one.
    ArtistBanner,
    /// The bot's own avatar.
    BotAvatar,
    /// The server's icon.
    ServerIcon,
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

/// The bar down a container's side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ContainerAccent {
    /// The bot's colour, and what the message means: grey while paused or idle, red for an error.
    Bot,
    /// One colour whatever happens, `#rrggbb`.
    Fixed { hex: String },
    /// No bar.
    None,
}

/// One part of a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LayoutBlock {
    /// Text: Discord markdown with variables.
    Text { content: String },
    /// One to three texts with a picture or a button beside them.
    Section {
        /// Older saves wrote one `content`; it reads as a single text.
        #[serde(alias = "content", deserialize_with = "one_or_many")]
        texts: Vec<String>,
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
    /// none. A [`Pager`](LayoutBlock::Pager) somewhere on the message pages it.
    List {
        item: String,
        empty: String,
        page_size: u8,
    },
    /// On a list message, the first / back / page / next / last buttons, drawn where this sits
    /// when the list runs past one page. Without one the list shows its first page only.
    Pager,
    /// The box with a bar down its side, holding any of the other parts.
    Container {
        accent: ContainerAccent,
        blocks: Vec<LayoutBlock>,
    },
}

/// A string where a list is expected reads as a list of one.
fn one_or_many<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(de)? {
        OneOrMany::One(s) => vec![s],
        OneOrMany::Many(v) => v,
    })
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
            LayoutBlock::Pager => "pager",
            LayoutBlock::Container { .. } => "container",
        }
    }
}

/// What saved layouts are stamped with; older ones are brought up to date by the library.
pub const LAYOUT_VERSION: u32 = 6;

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
    /// [`LAYOUT_VERSION`] when saved by this code; 0 for anything older.
    #[serde(default)]
    pub version: u32,
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
    #[serde(default = "default_queued_album")]
    pub queued_album: ViewLayout,
    #[serde(default = "default_queued_artist")]
    pub queued_artist: ViewLayout,
    #[serde(default = "default_queued_playlist")]
    pub queued_playlist: ViewLayout,
    #[serde(default = "default_lyrics")]
    pub lyrics: ViewLayout,
    #[serde(default = "default_done")]
    pub done: ViewLayout,
    #[serde(default = "default_notice")]
    pub notice: ViewLayout,
    #[serde(default = "default_error")]
    pub error: ViewLayout,
    #[serde(default = "default_vote")]
    pub vote: ViewLayout,
    #[serde(default = "default_vote_passed")]
    pub vote_passed: ViewLayout,
}

impl Default for BotLayouts {
    fn default() -> Self {
        Self {
            version: LAYOUT_VERSION,
            now_playing: default_now_playing(),
            idle: default_idle(),
            queued: default_queued(),
            left: default_left(),
            queue: default_queue(),
            history: default_history(),
            queued_album: default_queued_album(),
            queued_artist: default_queued_artist(),
            queued_playlist: default_queued_playlist(),
            lyrics: default_lyrics(),
            done: default_done(),
            notice: default_notice(),
            error: default_error(),
            vote: default_vote(),
            vote_passed: default_vote_passed(),
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
            LayoutView::QueuedAlbum => &self.queued_album,
            LayoutView::QueuedArtist => &self.queued_artist,
            LayoutView::QueuedPlaylist => &self.queued_playlist,
            LayoutView::Lyrics => &self.lyrics,
            LayoutView::Done => &self.done,
            LayoutView::Notice => &self.notice,
            LayoutView::Error => &self.error,
            LayoutView::Vote => &self.vote,
            LayoutView::VotePassed => &self.vote_passed,
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
            LayoutView::QueuedAlbum => &mut self.queued_album,
            LayoutView::QueuedArtist => &mut self.queued_artist,
            LayoutView::QueuedPlaylist => &mut self.queued_playlist,
            LayoutView::Lyrics => &mut self.lyrics,
            LayoutView::Done => &mut self.done,
            LayoutView::Notice => &mut self.notice,
            LayoutView::Error => &mut self.error,
            LayoutView::Vote => &mut self.vote,
            LayoutView::VotePassed => &mut self.vote_passed,
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
        LayoutView::QueuedAlbum => "added album",
        LayoutView::QueuedArtist => "added artist",
        LayoutView::QueuedPlaylist => "added playlist",
        LayoutView::Lyrics => "lyrics",
        LayoutView::Done => "done",
        LayoutView::Notice => "notice",
        LayoutView::Error => "error",
        LayoutView::Vote => "vote",
        LayoutView::VotePassed => "vote passed",
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

/// The shipped design puts everything in one container in the bot's colour.
fn boxed(blocks: Vec<LayoutBlock>) -> ViewLayout {
    ViewLayout {
        blocks: vec![LayoutBlock::Container {
            accent: ContainerAccent::Bot,
            blocks,
        }],
    }
}

/// The design the bot ships with: the controller as it has always looked, in the template
/// language, so it is also a worked example of the variables.
/// The box with the bot's colour down its side, holding some of a message's parts.
fn container(blocks: Vec<LayoutBlock>) -> LayoutBlock {
    LayoutBlock::Container {
        accent: ContainerAccent::Bot,
        blocks,
    }
}

/// The controller: the header and everything beside the art in the box, the two rows of
/// controls under it.
pub fn default_now_playing() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            container(vec![
                text("### {icon} {heading} in {channel}"),
                separator(true, SeparatorSpacing::Small),
                LayoutBlock::Section {
                    texts: vec!["{track}\n-# Requested by {requester}\n{player.progress_bar:12} {player.position} / {track.duration}\n-# {queue.count} tracks in queue ({queue.duration}) · Volume: {player.volume}%".to_string()],
                    accessory: Accessory::Image {
                        source: ImageSource::Cover,
                    },
                },
            ]),
            controls(&[ControlButton::Loop, ControlButton::Previous, ControlButton::PlayPause, ControlButton::Skip, ControlButton::Shuffle]),
            controls(&[ControlButton::Queue, ControlButton::History, ControlButton::Autoplay, ControlButton::Lyrics, ControlButton::Leave]),
        ],
    }
}

pub fn default_idle() -> ViewLayout {
    boxed(vec![
        text("### {icon} {heading} in {channel}"),
        separator(true, SeparatorSpacing::Small),
        text("-# The queue is empty. `/play` something."),
    ])
}

/// Added a track: what it is, how long it runs and when it plays, beside its cover.
pub fn default_queued() -> ViewLayout {
    boxed(vec![
        text("### {icon} {heading}"),
        separator(true, SeparatorSpacing::Small),
        LayoutBlock::Section {
            texts: vec!["{added}\n-# {added.meta}".to_string()],
            accessory: Accessory::Image {
                source: ImageSource::Cover,
            },
        },
    ])
}

pub fn default_left() -> ViewLayout {
    boxed(vec![
        text("### {icon} {heading}"),
        separator(true, SeparatorSpacing::Small),
        text("-# {left.reason} · `/play` to bring me back"),
    ])
}

/// The album, artist and playlist toasts start as the track toast: what was added reads as
/// the album, the artist or the playlist.
pub fn default_queued_album() -> ViewLayout {
    default_queued()
}

pub fn default_queued_artist() -> ViewLayout {
    default_queued()
}

pub fn default_queued_playlist() -> ViewLayout {
    default_queued()
}

/// A page of lyrics under the title, the page buttons under the box.
pub fn default_lyrics() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            container(vec![
                text("### {icon} {track.title}\n-# {track.artist}"),
                separator(true, SeparatorSpacing::Small),
                text("{lyrics}"),
            ]),
            LayoutBlock::Pager,
        ],
    }
}

/// A reply: the title line, then the detail under a divider when there is any (the renderer
/// drops the divider with it, so a bare "Paused" is one line).
pub fn default_done() -> ViewLayout {
    boxed(vec![
        text("### {icon} {heading}"),
        separator(true, SeparatorSpacing::Small),
        text("{detail}"),
    ])
}

pub fn default_notice() -> ViewLayout {
    boxed(vec![
        text("### {icon} {heading}"),
        separator(true, SeparatorSpacing::Small),
        text("{detail}"),
    ])
}

pub fn default_error() -> ViewLayout {
    boxed(vec![
        text("### {icon} {heading}"),
        separator(true, SeparatorSpacing::Small),
        text("{detail}"),
    ])
}

/// A vote to skip: who wants it, and how far along it is.
pub fn default_vote() -> ViewLayout {
    boxed(vec![
        text("### {icon} {heading}\n{vote.by} wants to skip {track.line}\n-# {vote.count} of {vote.needed} votes · {vote.percent}% of {vote.listeners} listening"),
    ])
}

pub fn default_vote_passed() -> ViewLayout {
    boxed(vec![text(
        "### {icon} {heading}\n{track.line}\n-# {vote.count} votes",
    )])
}

/// The queue: a Clear button beside the header, the entries, the page buttons under the box.
pub fn default_queue() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            container(vec![
                LayoutBlock::Section {
                    texts: vec![
                        "### {icon} {heading}\n-# {queue.tracks} · {queue.duration} · {bot}"
                            .to_string(),
                    ],
                    accessory: Accessory::Button {
                        button: ButtonSpec::Control {
                            control: ControlButton::Clear,
                        },
                    },
                },
                separator(true, SeparatorSpacing::Small),
                text("{player.line}"),
                separator(false, SeparatorSpacing::Small),
                LayoutBlock::List {
                    item: "`{index}.` {track.line} · {track.duration} · {requester}".to_string(),
                    empty: "-# The queue is empty.".to_string(),
                    page_size: 10,
                },
                separator(false, SeparatorSpacing::Large),
            ]),
            LayoutBlock::Pager,
        ],
    }
}

pub fn default_history() -> ViewLayout {
    ViewLayout {
        blocks: vec![
            container(vec![
                text("### {icon} {heading}"),
                separator(true, SeparatorSpacing::Small),
                LayoutBlock::List {
                    item:
                        "{track.line}\n-# {play.at} · {play.length} · {requester} · {play.counted}"
                            .to_string(),
                    empty: "-# Nothing has played yet.".to_string(),
                    page_size: 10,
                },
            ]),
            LayoutBlock::Pager,
        ],
    }
}

/// A server's own versions of some of the bot's messages. A view that is absent means "the
/// bot's". Written by the library owner from the dashboard, per server, in the same language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LayoutOverrides {
    /// [`LAYOUT_VERSION`] when saved by this code; 0 for anything older.
    #[serde(default)]
    pub version: u32,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued_album: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued_artist: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued_playlist: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vote: Option<ViewLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vote_passed: Option<ViewLayout>,
}

impl Default for LayoutOverrides {
    fn default() -> Self {
        Self {
            version: LAYOUT_VERSION,
            now_playing: None,
            idle: None,
            queued: None,
            left: None,
            queue: None,
            history: None,
            queued_album: None,
            queued_artist: None,
            queued_playlist: None,
            lyrics: None,
            done: None,
            notice: None,
            error: None,
            vote: None,
            vote_passed: None,
        }
    }
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
            LayoutView::QueuedAlbum => self.queued_album.as_ref(),
            LayoutView::QueuedArtist => self.queued_artist.as_ref(),
            LayoutView::QueuedPlaylist => self.queued_playlist.as_ref(),
            LayoutView::Lyrics => self.lyrics.as_ref(),
            LayoutView::Done => self.done.as_ref(),
            LayoutView::Notice => self.notice.as_ref(),
            LayoutView::Error => self.error.as_ref(),
            LayoutView::Vote => self.vote.as_ref(),
            LayoutView::VotePassed => self.vote_passed.as_ref(),
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
/// block is a section with a thumbnail or a row of page buttons.
pub const MAX_BLOCKS: usize = 20;
pub const MAX_ROWS: usize = 5;
pub const MAX_PER_ROW: usize = 5;
pub const MAX_GALLERY: usize = 10;
/// Discord's cap on the texts beside a section's accessory.
pub const MAX_SECTION_TEXTS: usize = 3;
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

fn check_button(b: &ButtonSpec, seen: &mut Vec<ControlButton>) -> Result<(), String> {
    match b {
        ButtonSpec::Control { control } => {
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

fn is_hex6(s: &str) -> bool {
    let h = s.strip_prefix('#').unwrap_or(s);
    h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit())
}

/// What one pass over the blocks keeps count of.
#[derive(Default)]
struct Tally {
    blocks: usize,
    rows: usize,
    lists: usize,
    seen: Vec<ControlButton>,
}

impl ViewLayout {
    /// The rules, in the order a person would want to hear them.
    pub fn validate(&self, view: LayoutView) -> Result<(), String> {
        if self.blocks.is_empty() {
            return Err("a message needs at least one block".into());
        }
        let mut tally = Tally::default();
        check_blocks(view, &self.blocks, false, &mut tally)?;
        if view.is_list() && tally.lists == 0 {
            return Err(format!(
                "the {} message needs a list block",
                view_name(view)
            ));
        }
        Ok(())
    }

    /// Every block, containers opened up, in order.
    pub fn flat(&self) -> Vec<&LayoutBlock> {
        let mut out = Vec::new();
        for b in &self.blocks {
            out.push(b);
            if let LayoutBlock::Container { blocks, .. } = b {
                out.extend(blocks.iter());
            }
        }
        out
    }
}

fn check_blocks(
    view: LayoutView,
    blocks: &[LayoutBlock],
    inside: bool,
    tally: &mut Tally,
) -> Result<(), String> {
    for block in blocks {
        tally.blocks += 1;
        if tally.blocks > MAX_BLOCKS {
            return Err(format!("at most {MAX_BLOCKS} blocks"));
        }
        let rows = &mut tally.rows;
        let lists = &mut tally.lists;
        let seen = &mut tally.seen;
        {
            match block {
                LayoutBlock::Container { accent, blocks } => {
                    if inside {
                        return Err("a container cannot hold another container".into());
                    }
                    if blocks.is_empty() {
                        return Err("a container needs something in it".into());
                    }
                    if let ContainerAccent::Fixed { hex } = accent {
                        if !is_hex6(hex) {
                            return Err("a container's colour is #rrggbb".into());
                        }
                    }
                    check_blocks(view, blocks, true, tally)?;
                    continue;
                }
                LayoutBlock::Text { content } => {
                    check_text("a text block", content, MAX_TEXT_CHARS)?
                }
                LayoutBlock::Section { texts, accessory } => {
                    if texts.is_empty() || texts.len() > MAX_SECTION_TEXTS {
                        return Err(format!("a section holds one to {MAX_SECTION_TEXTS} texts"));
                    }
                    for t in texts {
                        check_text("a section's text", t, MAX_TEXT_CHARS)?;
                    }
                    match accessory {
                        Accessory::Image { source } => check_image(source)?,
                        Accessory::Button { button } => {
                            *rows += 1;
                            check_button(button, seen)?;
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
                    *rows += 1;
                    if *rows > MAX_ROWS {
                        return Err(format!("at most {MAX_ROWS} rows of buttons"));
                    }
                    if buttons.is_empty() {
                        return Err("a row needs at least one button".into());
                    }
                    if buttons.len() > MAX_PER_ROW {
                        return Err(format!("at most {MAX_PER_ROW} buttons in a row"));
                    }
                    for b in buttons {
                        check_button(b, seen)?;
                    }
                }
                LayoutBlock::Pager => {
                    if !view.is_paged() {
                        return Err(format!(
                            "page buttons do not belong in the {} message",
                            view_name(view)
                        ));
                    }
                    *rows += 1;
                    if *rows > MAX_ROWS {
                        return Err(format!("at most {MAX_ROWS} rows of buttons"));
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
                    *lists += 1;
                    if *lists > 1 {
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
    }
    Ok(())
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
        // A missing view falls back to its default; an old save has no version.
        let partial: BotLayouts =
            serde_json::from_str(r#"{"idle":{"blocks":[{"kind":"text","content":"hi"}]}}"#)
                .unwrap();
        assert_eq!(partial.now_playing, default_now_playing());
        assert_eq!(partial.idle.blocks.len(), 1);
        assert_eq!(partial.version, 0);
        assert_eq!(BotLayouts::default().version, LAYOUT_VERSION);
        assert_eq!(LayoutOverrides::default().version, LAYOUT_VERSION);
    }

    #[test]
    fn the_rules_refuse_what_discord_would() {
        // The controller's two rows sit under its box.
        let mut v = default_now_playing();
        if let LayoutBlock::Row { buttons } = &mut v.blocks[1] {
            buttons.push(ButtonSpec::Control {
                control: ControlButton::Stop,
            });
        }
        assert!(v
            .validate(LayoutView::NowPlaying)
            .unwrap_err()
            .contains("at most 5"));

        let mut v = default_now_playing();
        if let LayoutBlock::Row { buttons } = &mut v.blocks[2] {
            buttons[0] = ButtonSpec::Control {
                control: ControlButton::Skip,
            };
        }
        assert!(v
            .validate(LayoutView::NowPlaying)
            .unwrap_err()
            .contains("twice"));

        // A section holds one to three texts, and an older save's single `content` still reads.
        let four = ViewLayout {
            blocks: vec![LayoutBlock::Section {
                texts: vec!["a".into(), "b".into(), "c".into(), "d".into()],
                accessory: Accessory::Image {
                    source: ImageSource::ArtistBanner,
                },
            }],
        };
        assert!(four
            .validate(LayoutView::Idle)
            .unwrap_err()
            .contains("one to 3"));
        let old: LayoutBlock = serde_json::from_str(
            r#"{"kind":"section","content":"hi","accessory":{"kind":"image","source":{"kind":"cover"}}}"#,
        )
        .unwrap();
        assert!(matches!(&old, LayoutBlock::Section { texts, .. } if texts == &["hi".to_string()]));
        let json = serde_json::to_string(&old).unwrap();
        assert!(json.contains(r#""texts":["hi"]"#) && !json.contains("content"));

        // A container is a block like the others: none, several, but never one in another, and
        // never empty; a fixed colour is a hex triple.
        let plain = ViewLayout {
            blocks: vec![text("no box"), separator(true, SeparatorSpacing::Small)],
        };
        plain.validate(LayoutView::Idle).unwrap();
        let nested = ViewLayout {
            blocks: vec![LayoutBlock::Container {
                accent: ContainerAccent::None,
                blocks: vec![LayoutBlock::Container {
                    accent: ContainerAccent::Bot,
                    blocks: vec![text("x")],
                }],
            }],
        };
        assert!(nested
            .validate(LayoutView::Idle)
            .unwrap_err()
            .contains("another container"));
        let empty = ViewLayout {
            blocks: vec![LayoutBlock::Container {
                accent: ContainerAccent::Bot,
                blocks: vec![],
            }],
        };
        assert!(empty
            .validate(LayoutView::Idle)
            .unwrap_err()
            .contains("needs something"));
        let bad_hex = ViewLayout {
            blocks: vec![LayoutBlock::Container {
                accent: ContainerAccent::Fixed { hex: "pink".into() },
                blocks: vec![text("x")],
            }],
        };
        assert!(bad_hex
            .validate(LayoutView::Idle)
            .unwrap_err()
            .contains("#rrggbb"));
        // A list inside a container still counts for the list message.
        let boxed_list = ViewLayout {
            blocks: vec![LayoutBlock::Container {
                accent: ContainerAccent::Fixed {
                    hex: "#f2258c".into(),
                },
                blocks: vec![LayoutBlock::List {
                    item: "{track.line}".into(),
                    empty: "-# nothing".into(),
                    page_size: 5,
                }],
            }],
        };
        boxed_list.validate(LayoutView::Queue).unwrap();
        assert_eq!(boxed_list.flat().len(), 2);

        // A control goes on any message: a Clear button on the queue page, say.
        ViewLayout {
            blocks: vec![controls(&[ControlButton::Clear])],
        }
        .validate(LayoutView::Idle)
        .unwrap();

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

    #[test]
    fn page_buttons_belong_on_lists_and_count_as_rows() {
        let v = ViewLayout {
            blocks: vec![text("x"), LayoutBlock::Pager],
        };
        assert!(v
            .validate(LayoutView::NowPlaying)
            .unwrap_err()
            .contains("page buttons"));
        let list = LayoutBlock::List {
            item: "{track.line}".into(),
            empty: "-".into(),
            page_size: 10,
        };
        // Anywhere on the message, and more than once.
        ViewLayout {
            blocks: vec![LayoutBlock::Pager, list.clone(), LayoutBlock::Pager],
        }
        .validate(LayoutView::Queue)
        .unwrap();
        // Without one a list message is still whole: it shows its first page.
        ViewLayout {
            blocks: vec![list.clone()],
        }
        .validate(LayoutView::History)
        .unwrap();
        // Lyrics turn pages without a list.
        ViewLayout {
            blocks: vec![text("{lyrics}"), LayoutBlock::Pager],
        }
        .validate(LayoutView::Lyrics)
        .unwrap();
        let mut blocks = vec![list];
        blocks.extend(vec![LayoutBlock::Pager; MAX_ROWS + 1]);
        assert!(ViewLayout { blocks }
            .validate(LayoutView::Queue)
            .unwrap_err()
            .contains("rows of buttons"));
    }
}
