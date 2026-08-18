//! Plans, entitlements and profile badges.
//!
//! Chordia is self-hostable and AGPL, so every one of these is optional by construction: a Hub with
//! no payment provider configured reports [`Entitlements::billing_enabled`] `false` and hands every
//! feature to every account. Nothing here gates the product's identity — playback quality, EQ,
//! offline, libraries and the social graph are the same on every tier. What a plan buys is
//! expression, depth of insight, and ownership of your own history.
//!
//! ## Where the truth lives
//!
//! [`Entitlements`] is the ONE thing a client should branch on, and it is served on `GET /v1/me`
//! alongside the profile so a gate never needs its own request. The client must not derive a tier
//! from a badge, a subscription row or a price: those describe billing, while entitlements describe
//! what the account may do, and admin/staff comps make the two legitimately disagree.

use serde::{Deserialize, Serialize};

use crate::EpochMillis;

/// A subscription tier. Ordered: every tier includes everything below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum PlanTier {
    #[default]
    Free,
    Sonic,
    SuperSonic,
}

/// How often a subscription bills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum BillingInterval {
    #[default]
    Monthly,
    Yearly,
}

/// The lifecycle state of a subscription, as far as the account is concerned.
///
/// `Canceled` is NOT the same as `Expired`: a canceled subscription keeps its entitlements until
/// the period already paid for runs out, which is the difference between "I turned off renewal" and
/// "I no longer have this".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum BillingStatus {
    /// No subscription has ever been started, or the last one has fully lapsed.
    #[default]
    None,
    Active,
    Trialing,
    /// Payment failed; access continues through a short grace period.
    PastDue,
    /// Will not renew. Access continues until `current_period_end`.
    Canceled,
    Paused,
    Expired,
}

/// A capability an account may or may not have.
///
/// Deliberately a flat list rather than a tier comparison at the call site: `has(SmartPlaylists)`
/// survives a tier being renamed, split or comped, where `tier >= Sonic` does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum Feature {
    /// Any hex colour as the accent, beyond the built-in presets.
    CustomAccent,
    /// Accents that change over time or with the artwork.
    DynamicAccent,
    /// The account's display name renders in its accent wherever it appears.
    NameAccent,
    /// The account's profile page paints in its accent for visitors.
    ProfileAccent,
    /// Avatars keep their animation instead of being flattened to a still.
    AnimatedAvatar,
    /// Creating, editing and refreshing rule-based playlists.
    SmartPlaylists,
    /// The detailed breakdown behind a friend's compatibility score.
    TasteMatchSummary,
    /// Arbitrary date ranges on charts and Wrapped, rather than the fixed periods.
    DeepAnalytics,
    /// Downloading listening history as CSV.
    CsvExport,
    /// Correcting or deleting individual plays.
    ScrobbleEditing,
    /// Importing listening history from Spotify or Last.fm exports.
    HistoryImport,
}

/// Everything a client needs to decide what to show. Served with the profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Entitlements {
    #[serde(default)]
    pub tier: PlanTier,
    /// The capabilities this account actually has. Branch on this, not on `tier`.
    #[serde(default)]
    pub features: Vec<Feature>,
    /// How far back listening history is readable, in days. `None` = no limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_days: Option<u32>,
    /// The oldest timestamp history queries will return, derived from `retention_days`. Lets the UI
    /// say "plays before this date are hidden" instead of silently showing a truncated chart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_from: Option<EpochMillis>,
    /// Start of the current unbroken paid streak. Survives switching tier and switching interval,
    /// and survives cancelling then resubscribing before the period runs out; resets only when the
    /// subscription actually lapses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_since: Option<EpochMillis>,
    /// These entitlements come from an admin/staff comp rather than a payment. No badge is granted:
    /// the tier badges mean "supporter", and comping one would make it mean nothing.
    #[serde(default)]
    pub complimentary: bool,
    /// False when this Hub has no payment provider configured. Everything is unlocked, and clients
    /// must render no plan UI at all — a self-hoster should never see an upsell.
    #[serde(default)]
    pub billing_enabled: bool,
}

/// One purchasable plan, for the pricing table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlanInfo {
    pub tier: PlanTier,
    pub features: Vec<Feature>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_days: Option<u32>,
    /// Minor units (cents), so no float ever touches a price.
    pub monthly_price_cents: u32,
    pub yearly_price_cents: u32,
    /// ISO 4217, e.g. `USD`.
    pub currency: String,
    /// Absent when this Hub has not configured a product for that interval, which is also how a
    /// client knows not to offer it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monthly_product_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yearly_product_id: Option<String>,
}

/// `GET /v1/billing/plans`. Public: the pricing table is readable signed out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlansResponse {
    pub billing_enabled: bool,
    /// Paid plans only — Free is the absence of a subscription, not a product.
    pub plans: Vec<PlanInfo>,
}

/// `GET /v1/billing/me`. The billing view of the account, beside its entitlements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BillingMe {
    pub entitlements: Entitlements,
    #[serde(default)]
    pub status: BillingStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<BillingInterval>,
    /// When the current paid period ends: the renewal date, or the date access stops if cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_period_end: Option<EpochMillis>,
    #[serde(default)]
    pub cancel_at_period_end: bool,
    /// Whether a billing-portal link can be minted for this account.
    #[serde(default)]
    pub has_customer: bool,
    /// A checkout started but not yet confirmed by a webhook, so the plan page can say "activating"
    /// after a reload rather than looking like the payment did nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_checkout: Option<EpochMillis>,
}

/// `POST /v1/billing/checkout`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CheckoutRequest {
    pub tier: PlanTier,
    pub interval: BillingInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CheckoutResponse {
    /// Send the browser here. Hosted by the payment provider; card details never reach the Hub.
    pub checkout_url: String,
    /// Our idempotency key, echoed back on the return URL so the page can poll for activation.
    pub request_id: crate::Uuid,
}

/// `POST /v1/billing/portal`: a one-time link into the provider's customer portal, where cancelling,
/// resuming and payment methods live.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PortalResponse {
    pub url: String,
}

/// `POST /v1/billing/change`: move an existing subscription to another tier or interval.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChangePlanRequest {
    pub tier: PlanTier,
    pub interval: BillingInterval,
}

/// A staff member's role, shown on their badge so moderation actions have a face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum StaffRole {
    Support,
    Moderator,
    Admin,
}

/// A badge on a profile.
///
/// Tagged by `kind` with the detail inline, rather than a flat struct of nullable fields: a badge's
/// detail is exactly what makes it that badge, and every renderer has to switch on the kind anyway.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ProfileBadge {
    /// The person who builds this. Title and tagline are configured per account, so it can say
    /// something true rather than a generic label.
    Developer {
        title: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tagline: Option<String>,
    },
    Staff {
        role: StaffRole,
    },
    /// Joined before this instance's early-bird cutoff.
    EarlyBird {
        joined_at: EpochMillis,
    },
    /// One of the first hundred accounts to ever start a paid subscription. Never revoked, and the
    /// rank is never reissued — including after the holder deletes their account.
    EarlySupporter {
        rank: u16,
        since: EpochMillis,
    },
    Sonic {
        since: EpochMillis,
        /// Whole months of the current unbroken streak. The client derives any visual stage from
        /// this, so changing the stage thresholds needs no server deploy.
        streak_months: u32,
    },
    SuperSonic {
        since: EpochMillis,
        streak_months: u32,
    },
}

/// `PATCH /v1/admin/users/{id}/badges`.
///
/// Three-valued per field, matching `UpdateProfile`: absent leaves it alone, `""` clears it, a value
/// sets it. Without that distinction an admin could not remove a developer title without also
/// clearing the staff role in the same request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminBadgeUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_tagline: Option<String>,
    /// `support` | `moderator` | `admin`, or `""` to remove the staff badge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff_role: Option<String>,
}
