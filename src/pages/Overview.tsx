import React from "react";
import TrackRow from "../components/TrackRow";

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

interface OverviewProps {
  profile: Profile | null;
  recentTracks: RecentTracksResponse | null;
}

const Overview: React.FC<OverviewProps> = ({ profile, recentTracks }) => {
  if (!profile) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-[#d51007]"></div>
      </div>
    );
  }

  const nowPlaying = recentTracks?.now_playing;
  const recentScrobbles = recentTracks?.tracks || [];
  const lastPlayed = recentScrobbles.find((track) => !track.now_playing);
  const daysListening = profile.registered_at
    ? Math.max(
        1,
        Math.floor(
          (Date.now() - new Date(profile.registered_at).getTime()) / 86400000,
        ),
      )
    : 1;
  const averagePerDay = Math.round(profile.playcount / daysListening);

  return (
    <div className="min-h-full bg-[#0a0a0a] px-5 py-6 sm:px-8 lg:px-10">
      <div className="mx-auto max-w-6xl">
        <div className="mb-8 flex flex-wrap items-end justify-between gap-4 border-b border-[#242424] pb-7">
          <div className="flex items-center gap-4">
            {profile.image_url && (
              <img
                src={profile.image_url}
                alt={profile.realname || profile.username}
                className="h-20 w-20 rounded-full border-2 border-[#d51007] object-cover"
              />
            )}
            <div>
              <p className="mb-1 text-xs font-semibold uppercase tracking-[0.22em] text-[#d51007]">
                Your listening desk
              </p>
              <h1 className="mb-1 text-3xl font-bold tracking-tight text-white">
                {profile.realname || profile.username}
              </h1>
              <p className="text-sm text-gray-500">
                @{profile.username}
                {profile.country ? ` · ${profile.country}` : ""}
              </p>
            </div>
          </div>
          <div className="grid min-w-[280px] grid-cols-2 gap-px overflow-hidden rounded-lg border border-[#292929] bg-[#292929]">
            <div className="bg-[#151515] px-5 py-3">
              <p className="text-xs uppercase tracking-wider text-gray-500">
                Scrobbles
              </p>
              <p className="mt-1 text-xl font-semibold text-white">
                {profile.playcount.toLocaleString()}
              </p>
            </div>
            <div className="bg-[#151515] px-5 py-3">
              <p className="text-xs uppercase tracking-wider text-gray-500">
                Loved songs
              </p>
              <p className="mt-1 text-xl font-semibold text-white">
                {profile.loved_songs.toLocaleString()}
              </p>
            </div>
          </div>
        </div>

        {(nowPlaying || lastPlayed) && (
          <div className="mb-8 border-l-2 border-[#d51007] bg-[#151515] px-5 py-4 shadow-[0_10px_35px_rgba(0,0,0,0.22)]">
            <div className="mb-2 flex items-center justify-between">
              <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#d51007]">
                {nowPlaying ? "Now playing" : "Last played"}
              </h2>
              <span className="text-xs text-gray-600">
                {nowPlaying ? "LIVE" : "RECENT"}
              </span>
            </div>
            <TrackRow
              track={nowPlaying || lastPlayed!}
              isNowPlaying={Boolean(nowPlaying)}
            />
          </div>
        )}

        <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_280px]">
          <section>
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-lg font-semibold text-white">
                Recent scrobbles
              </h2>
              <span className="text-xs uppercase tracking-wider text-gray-600">
                Latest activity
              </span>
            </div>
            {recentScrobbles.length > 0 ? (
              <div className="divide-y divide-[#202020] border-y border-[#202020]">
                {recentScrobbles.map((track, index) => (
                  <TrackRow
                    key={`${track.name}-${track.artist}-${index}`}
                    track={track}
                  />
                ))}
              </div>
            ) : (
              <div className="border border-dashed border-[#303030] py-12 text-center text-gray-500">
                <p>No recent scrobbles yet.</p>
              </div>
            )}
          </section>
          <aside className="hidden border-l border-[#242424] pl-6 lg:block">
            <p className="mb-4 text-xs font-semibold uppercase tracking-[0.2em] text-gray-500">
              Listening snapshot
            </p>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-3xl font-semibold tracking-tight text-white">
                  {averagePerDay.toLocaleString()}
                </p>
                <p className="mt-1 text-sm text-gray-500">average per day</p>
              </div>
              <div>
                <p className="text-3xl font-semibold tracking-tight text-white">
                  {profile.weekly_scrobbles.toLocaleString()}
                </p>
                <p className="mt-1 text-sm text-gray-500">this week</p>
              </div>
            </div>
            <div className="mt-8 h-px bg-[#242424]" />
            <p className="mt-6 text-sm leading-6 text-gray-500">
              Based on {profile.playcount.toLocaleString()} total scrobbles
              since {new Date(profile.registered_at).toLocaleDateString()}.
            </p>
          </aside>
        </div>
      </div>
    </div>
  );
};

export default Overview;
