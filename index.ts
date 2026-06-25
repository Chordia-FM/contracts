// @chordia/contracts - TypeScript entry point.
//
// Generated bindings from the Rust crate (via ts-rs / typeshare):
export * from "./bindings/AccessClaims";
export * from "./bindings/Album";
export * from "./bindings/AlbumDetail";
export * from "./bindings/AlbumOverrideInput";
export * from "./bindings/AlbumOverrideView";
export * from "./bindings/ArtOption";
export * from "./bindings/ArtOptions";
export * from "./bindings/Artist";
export * from "./bindings/ArtistDetail";
export * from "./bindings/ArtistLabel";
export * from "./bindings/ArtistRef";
export * from "./bindings/ArtistRelation";
export * from "./bindings/ArtistOverrideInput";
export * from "./bindings/ArtistOverrideView";
export * from "./bindings/AudioProperties";
export * from "./bindings/BrowseAlbum";
export * from "./bindings/BrowseArtist";
export * from "./bindings/BrowseTrack";
export * from "./bindings/BucketGranularity";
export * from "./bindings/CatalogPruneRequest";
export * from "./bindings/CatalogSyncRequest";
export * from "./bindings/CatalogSyncResponse";
export * from "./bindings/CapabilityAction";
export * from "./bindings/CapabilityClaims";
export * from "./bindings/ClientMessage";
export * from "./bindings/ClientType";
export * from "./bindings/Compatibility";
export * from "./bindings/DailyMix";
export * from "./bindings/DailyMixDetail";
export * from "./bindings/DiscoveryResult";
export * from "./bindings/EntityKind";
export * from "./bindings/EntityStats";
export * from "./bindings/EqBand";
export * from "./bindings/EqConfig";
export * from "./bindings/EqPreset";
export * from "./bindings/FriendNowPlaying";
export * from "./bindings/FriendRequest";
export * from "./bindings/FriendScrobble";
export * from "./bindings/Friendship";
export * from "./bindings/FriendshipStatus";
export * from "./bindings/HeartbeatRequest";
export * from "./bindings/HeartbeatResponse";
export * from "./bindings/HistoryEntry";
export * from "./bindings/HistoryPage";
export * from "./bindings/LabelDetail";
export * from "./bindings/LabelSummary";
export * from "./bindings/LibraryShare";
export * from "./bindings/LibrarySummary";
export * from "./bindings/ListeningCharts";
export * from "./bindings/ListeningEvent";
export * from "./bindings/LoginRequest";
export * from "./bindings/Lyrics";
export * from "./bindings/LyricsEditInput";
export * from "./bindings/LyricsLine";
export * from "./bindings/LyricsSyncType";
export * from "./bindings/MatchQuery";
export * from "./bindings/MatchResult";
export * from "./bindings/MatchStrength";
export * from "./bindings/NowPlaying";
export * from "./bindings/NowPlayingReport";
export * from "./bindings/Period";
export * from "./bindings/PermissionLevel";
export * from "./bindings/PinKind";
export * from "./bindings/PinnedItem";
export * from "./bindings/PlaybackSource";
export * from "./bindings/Playlist";
export * from "./bindings/PlaylistDetail";
export * from "./bindings/PublicProfile";
export * from "./bindings/PublicUser";
export * from "./bindings/QualityPreferences";
export * from "./bindings/QualityProfile";
export * from "./bindings/QueueItem";
export * from "./bindings/RecentItem";
export * from "./bindings/RecentKind";
export * from "./bindings/RecentPlay";
export * from "./bindings/RegisterRequest";
export * from "./bindings/RelayRequest";
export * from "./bindings/ResolvedServer";
export * from "./bindings/ResourceRef";
export * from "./bindings/RoomState";
export * from "./bindings/ScrobbleBatch";
export * from "./bindings/ScrobbleBatchResponse";
export * from "./bindings/ScrobblePrivacy";
export * from "./bindings/SearchResults";
export * from "./bindings/ServerEndpoint";
export * from "./bindings/ServerMessage";
export * from "./bindings/SessionInfo";
export * from "./bindings/ShareRequest";
export * from "./bindings/SimilarUser";
export * from "./bindings/SmartCondition";
export * from "./bindings/SmartField";
export * from "./bindings/SmartMatch";
export * from "./bindings/SmartOp";
export * from "./bindings/SmartPlaylist";
export * from "./bindings/SmartPlaylistDetail";
export * from "./bindings/SmartRules";
export * from "./bindings/SmartSort";
export * from "./bindings/StreamQuery";
export * from "./bindings/SyncTrack";
export * from "./bindings/TimeBucket";
export * from "./bindings/TokenPair";
export * from "./bindings/TopItem";
export * from "./bindings/Track";
export * from "./bindings/TrackFingerprint";
export * from "./bindings/TrackOverrideInput";
export * from "./bindings/TrackOverrideView";
export * from "./bindings/TranscodeTarget";
export * from "./bindings/Trending";
export * from "./bindings/UpdateProfile";
export * from "./bindings/UserProfile";
export * from "./bindings/UserSettings";
export * from "./bindings/WrappedReport";

// ── Hand-declared types not yet in the Rust bindings ─────────────────────────

/** Hub auth response: user + token pair. */
export interface AuthResponse {
	user: import("./bindings/UserProfile").UserProfile;
	tokens: import("./bindings/TokenPair").TokenPair;
}

/** Hub capability grant request - asks the Hub for a short-lived streaming token. */
export interface GrantRequest {
	library_id: string;
	resource?: import("./bindings/ResourceRef").ResourceRef;
	action?: import("./bindings/CapabilityAction").CapabilityAction;
	room_id?: string;
}

/** Hub capability grant response. */
export interface GrantResponse {
	token: string;
	/** The resolved library server to connect to directly. */
	server: import("./bindings/ServerEndpoint").ServerEndpoint;
	expires_at: number;
}

/** Refresh token request. */
export interface RefreshRequest {
	refresh_token: string;
}
