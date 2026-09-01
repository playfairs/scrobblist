import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import TrackRow from "../components/TrackRow";

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

interface RecentTracksResponse {
  tracks: Track[];
  now_playing: Track | null;
}

interface ScrobblesProps {
  recentTracks: RecentTracksResponse | null;
  onRefresh: () => Promise<void>;
}

const Scrobbles: React.FC<ScrobblesProps> = ({ recentTracks, onRefresh }) => {
  const [tracks, setTracks] = useState<Track[]>([]);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadPage = async (nextPage: number) => {
    setLoading(true);
    setError(null);
    try {
      const response = await invoke<RecentTracksResponse>("get_recent_tracks", {
        limit: 30,
        page: nextPage,
        refresh: true,
      });
      setTracks(response.tracks.filter((track) => !track.now_playing));
      setPage(nextPage);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (page === 1 && recentTracks) {
      setTracks(recentTracks.tracks.filter((track) => !track.now_playing));
      setLoading(false);
    }
  }, [page, recentTracks]);

  useEffect(() => {
    const refresh = () => {
      if (page === 1) void onRefresh();
      else void loadPage(page);
    };
    const interval = window.setInterval(refresh, 30000);
    window.addEventListener("focus", refresh);

    return () => {
      window.clearInterval(interval);
      window.removeEventListener("focus", refresh);
    };
  }, [onRefresh, page]);

  const refreshCurrentPage = async () => {
    setRefreshing(true);
    try {
      if (page === 1) await onRefresh();
      else await loadPage(page);
    } finally {
      setRefreshing(false);
    }
  };

  return (
    <div className="min-h-full bg-[#0a0a0a] px-5 py-6 sm:px-8 lg:px-10">
      <div className="mx-auto max-w-5xl">
        <header className="mb-8 flex flex-wrap items-end justify-between gap-4 border-b border-[#242424] pb-7">
          <div>
            <p className="mb-2 text-xs font-semibold uppercase tracking-[0.22em] text-[#d51007]">
              History
            </p>
            <h1 className="text-3xl font-bold tracking-tight text-white">
              Scrobbles
            </h1>
            <p className="mt-2 text-sm text-gray-500">
              Everything you have listened to, newest first.
            </p>
          </div>
          <button
            onClick={() => void refreshCurrentPage()}
            disabled={loading || refreshing}
            className="border border-[#343434] px-4 py-2 text-sm text-gray-300 transition-colors hover:border-[#d51007] hover:text-white disabled:cursor-wait disabled:opacity-50"
          >
            {loading || refreshing ? "Refreshing..." : "Refresh"}
          </button>
        </header>

        {error && (
          <div className="mb-5 border border-red-900/60 bg-red-950/30 px-4 py-3 text-sm text-red-300">
            {error}
          </div>
        )}
        {loading && tracks.length === 0 ? (
          <div className="py-16 text-center text-sm text-gray-500">
            Loading your listening history...
          </div>
        ) : tracks.length === 0 ? (
          <div className="border border-dashed border-[#303030] py-16 text-center text-gray-500">
            No scrobbles found.
          </div>
        ) : (
          <div className="divide-y divide-[#202020] border-y border-[#202020]">
            {tracks.map((track, index) => (
              <TrackRow
                key={`${track.name}-${track.artist}-${track.scrobbled_at}-${index}`}
                track={track}
              />
            ))}
          </div>
        )}

        <div className="mt-6 flex items-center justify-between">
          <button
            onClick={() => void loadPage(page - 1)}
            disabled={page === 1 || loading}
            className="border border-[#292929] px-4 py-2 text-sm text-gray-400 transition-colors hover:text-white disabled:cursor-not-allowed disabled:opacity-30"
          >
            Previous
          </button>
          <span className="text-xs uppercase tracking-wider text-gray-600">
            Page {page}
          </span>
          <button
            onClick={() => void loadPage(page + 1)}
            disabled={tracks.length < 30 || loading}
            className="border border-[#292929] px-4 py-2 text-sm text-gray-400 transition-colors hover:text-white disabled:cursor-not-allowed disabled:opacity-30"
          >
            Next
          </button>
        </div>
      </div>
    </div>
  );
};

export default Scrobbles;
