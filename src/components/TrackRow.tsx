import React from "react";
import { open } from "@tauri-apps/plugin-shell";

interface Track {
  name: string;
  artist: string;
  artist_url: string;
  album: string | null;
  album_url: string | null;
  url: string;
  image_url: string | null;
  now_playing: boolean;
  scrobbled_at: string | null;
}

interface TrackRowProps {
  track: Track;
  isNowPlaying?: boolean;
}

const lastFmPlaceholder =
  "https://lastfm.freetls.fastly.net/i/u/300x300/2a96cbd8b46e442fc41c2b86b821562f.png";

const TrackRow: React.FC<TrackRowProps> = ({ track, isNowPlaying = false }) => {
  const artistUrl =
    track.artist_url ||
    `https://www.last.fm/music/${encodeURIComponent(track.artist)}`;
  const trackUrl =
    track.url ||
    `https://www.last.fm/music/${encodeURIComponent(track.artist)}/_/${encodeURIComponent(track.name)}`;
  const albumUrl =
    track.album_url ||
    (track.album
      ? `https://www.last.fm/music/${encodeURIComponent(track.artist)}/${encodeURIComponent(track.album)}`
      : "");

  const openLastFm = async (url: string) => {
    if (!url) return;
    try {
      await open(url);
    } catch (error) {
      console.error("Failed to open Last.fm link:", error);
    }
  };

  const formatTimestamp = (timestamp: string | null) => {
    if (!timestamp) return null;
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  return (
    <div className="group flex min-h-[74px] items-center gap-4 px-3 py-3 transition-colors hover:bg-[#151515]">
      {track.image_url ? (
        <img
          src={track.image_url}
          alt={track.album || track.name}
          className="h-12 w-12 rounded-md object-cover shadow-lg"
          onError={(event) => {
            event.currentTarget.src = lastFmPlaceholder;
          }}
        />
      ) : (
        <img
          src={lastFmPlaceholder}
          alt=""
          className="h-12 w-12 rounded-md object-cover shadow-lg"
        />
      )}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          {isNowPlaying && (
            <span className="inline-block w-2 h-2 bg-[#d51007] rounded-full animate-pulse"></span>
          )}
          <button
            onClick={() => void openLastFm(trackUrl)}
            className="block max-w-full truncate text-left text-sm font-semibold text-white transition-colors hover:text-[#e31b12]"
            title={`Open ${track.name} on Last.fm`}
          >
            {track.name}
          </button>
        </div>
        <p className="truncate text-sm text-gray-500">
          <button
            onClick={() => void openLastFm(artistUrl)}
            className="transition-colors hover:text-[#e31b12]"
            title={`Open ${track.artist} on Last.fm`}
          >
            {track.artist}
          </button>
          {track.album && (
            <>
              <span className="text-gray-600"> · </span>
              <button
                onClick={() => void openLastFm(albumUrl)}
                className="transition-colors hover:text-[#e31b12]"
                title={`Open ${track.album} on Last.fm`}
              >
                {track.album}
              </button>
            </>
          )}
        </p>
      </div>
      {track.scrobbled_at && (
        <div className="whitespace-nowrap text-right text-xs text-gray-600">
          {formatTimestamp(track.scrobbled_at)}
        </div>
      )}
      {isNowPlaying && (
        <div className="whitespace-nowrap text-right text-xs font-semibold uppercase tracking-wider text-[#e31b12]">
          Now Playing
        </div>
      )}
    </div>
  );
};

export default TrackRow;
