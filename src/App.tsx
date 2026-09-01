import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./components/Sidebar";
import Overview from "./pages/Overview";
import Scrobbles from "./pages/Scrobbles";
import Artists from "./pages/Artists";
import Albums from "./pages/Albums";
import Tracks from "./pages/Tracks";
import Charts from "./pages/Charts";
import ProfilePage from "./pages/Profile";
import Settings from "./pages/Settings";
import LoadingState from "./components/LoadingState";
import ErrorState from "./components/ErrorState";

export type Page =
  | "overview"
  | "scrobbles"
  | "artists"
  | "albums"
  | "tracks"
  | "charts"
  | "profile"
  | "loved"
  | "settings";

interface Profile {
  username: string;
  realname: string | null;
  url: string;
  image_url: string | null;
  country: string | null;
  playcount: number;
  loved_songs: number;
  age: number | null;
  gender: string | null;
  subscriber: boolean;
  registered_at: string;
  weekly_scrobbles: number;
}

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

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("overview");
  const [isAuthenticated, setIsAuthenticated] = useState<boolean | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [profile, setProfile] = useState<Profile | null>(null);
  const [recentTracks, setRecentTracks] = useState<RecentTracksResponse | null>(
    null,
  );

  useEffect(() => {
    checkAuth();
  }, []);

  useEffect(() => {
    if (!isAuthenticated) return;

    const refresh = () => {
      void loadRecentTracks();
    };
    const interval = window.setInterval(refresh, 30000);
    window.addEventListener("focus", refresh);

    return () => {
      window.clearInterval(interval);
      window.removeEventListener("focus", refresh);
    };
  }, [isAuthenticated]);

  const checkAuth = async () => {
    try {
      const session = await invoke<{ username: string; has_session: boolean }>(
        "get_session",
      );
      setIsAuthenticated(session.has_session);

      if (session.has_session) {
        await loadProfile();
        await loadRecentTracks();
      }
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const loadProfile = async () => {
    const profileData = await invoke<Profile>("get_profile");
    setProfile(profileData);
  };

  const loadRecentTracks = async () => {
    const tracksData = await invoke<RecentTracksResponse>("get_recent_tracks", {
      limit: 20,
      refresh: true,
    });
    setRecentTracks(tracksData);
  };

  const handleAuth = async () => {
    if (isAuthenticating) return;

    setIsAuthenticating(true);
    setError(null);

    try {
      const session = await invoke<{ username: string; has_session: boolean }>(
        "start_lastfm_auth",
      );

      if (session.has_session) {
        setIsAuthenticated(true);
        await loadProfile();
        await loadRecentTracks();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsAuthenticating(false);
    }
  };

  if (isLoading) {
    return <LoadingState message="Loading Scrobblist..." />;
  }

  if (error) {
    return (
      <ErrorState message={error} onRetry={() => window.location.reload()} />
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-[#0a0a0a]">
        <div className="text-center">
          <h1 className="text-4xl font-bold mb-4 text-white">Scrobblist</h1>
          <p className="text-gray-400 mb-8">A Last.fm desktop client</p>
          <button
            onClick={handleAuth}
            disabled={isAuthenticating}
            className={`px-6 py-3 rounded-lg transition-colors ${
              isAuthenticating
                ? "bg-gray-700 text-gray-300 cursor-not-allowed"
                : "bg-[#d51007] hover:bg-[#b50d05] text-white"
            }`}
          >
            {isAuthenticating ? "Signing in…" : "Sign in with Last.fm"}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-[#0a0a0a]">
      <Sidebar currentPage={currentPage} onPageChange={setCurrentPage} />
      <main className="flex-1 overflow-auto">
        {currentPage === "overview" && (
          <Overview profile={profile} recentTracks={recentTracks} />
        )}
        {currentPage === "scrobbles" && (
          <Scrobbles recentTracks={recentTracks} onRefresh={loadRecentTracks} />
        )}
        {currentPage === "artists" && <Artists />}
        {currentPage === "albums" && <Albums />}
        {currentPage === "tracks" && <Tracks />}
        {currentPage === "charts" && <Charts />}
        {currentPage === "profile" && <ProfilePage profile={profile} />}
        {currentPage === "settings" && <Settings />}
        {currentPage !== "overview" &&
          ![
            "scrobbles",
            "artists",
            "albums",
            "tracks",
            "charts",
            "profile",
            "settings",
          ].includes(currentPage) && (
            <div className="flex items-center justify-center h-full text-gray-400">
              <p>Coming soon</p>
            </div>
          )}
      </main>
    </div>
  );
}

export default App;
