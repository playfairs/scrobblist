import React from "react";

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
}
const ProfilePage: React.FC<{ profile: Profile | null }> = ({ profile }) =>
  profile ? (
    <div className="min-h-full bg-[#0a0a0a] px-5 py-6 sm:px-8 lg:px-10">
      <div className="mx-auto max-w-4xl">
        <header className="mb-8 border-b border-[#242424] pb-7">
          <p className="mb-2 text-xs font-semibold uppercase tracking-[0.22em] text-[#e31b12]">
            Account
          </p>
          <h1 className="text-3xl font-bold tracking-tight text-white">
            Profile
          </h1>
          <p className="mt-2 text-sm text-gray-500">
            Everything Last.fm shares about your account.
          </p>
        </header>
        <div className="flex flex-col gap-6 border border-[#242424] bg-[#111] p-6 sm:flex-row sm:items-center">
          <div className="flex h-24 w-24 shrink-0 items-center justify-center overflow-hidden rounded-full border-2 border-[#e31b12] bg-[#241313] text-3xl font-bold text-[#e31b12]">
            {profile.image_url ? (
              <img
                src={profile.image_url}
                alt={profile.username}
                className="h-full w-full object-cover"
                onError={(event) => {
                  event.currentTarget.style.display = "none";
                }}
              />
            ) : (
              profile.username.slice(0, 1).toUpperCase()
            )}
          </div>
          <div>
            <h2 className="text-2xl font-semibold text-white">
              {profile.realname || profile.username}
            </h2>
            <p className="mt-1 text-gray-500">@{profile.username}</p>
            <a
              href={profile.url}
              target="_blank"
              rel="noopener noreferrer"
              className="mt-4 inline-block text-sm text-[#e31b12] hover:text-white"
            >
              View on Last.fm
            </a>
          </div>
        </div>
        <div className="mt-4 grid gap-px bg-[#292929] sm:grid-cols-3">
          <div className="bg-[#151515] p-5">
            <p className="text-xs uppercase tracking-wider text-gray-500">
              Scrobbles
            </p>
            <p className="mt-2 text-2xl font-semibold text-white">
              {profile.playcount.toLocaleString()}
            </p>
          </div>
          <div className="bg-[#151515] p-5">
            <p className="text-xs uppercase tracking-wider text-gray-500">
              Loved songs
            </p>
            <p className="mt-2 text-2xl font-semibold text-white">
              {profile.loved_songs.toLocaleString()}
            </p>
          </div>
          <div className="bg-[#151515] p-5">
            <p className="text-xs uppercase tracking-wider text-gray-500">
              Country
            </p>
            <p className="mt-2 text-2xl font-semibold text-white">
              {profile.country || "—"}
            </p>
          </div>
        </div>
        <div className="mt-4 grid gap-px bg-[#292929] sm:grid-cols-3">
          <div className="bg-[#151515] p-5">
            <p className="text-xs uppercase tracking-wider text-gray-500">
              Member since
            </p>
            <p className="mt-2 text-sm font-semibold text-white">
              {profile.registered_at
                ? new Date(profile.registered_at).toLocaleDateString()
                : "—"}
            </p>
          </div>
          <div className="bg-[#151515] p-5">
            <p className="text-xs uppercase tracking-wider text-gray-500">
              Age / gender
            </p>
            <p className="mt-2 text-sm font-semibold capitalize text-white">
              {profile.age || "—"}
              {profile.gender ? ` · ${profile.gender}` : ""}
            </p>
          </div>
          <div className="bg-[#151515] p-5">
            <p className="text-xs uppercase tracking-wider text-gray-500">
              Account
            </p>
            <p className="mt-2 text-sm font-semibold text-white">
              {profile.subscriber ? "Subscriber" : "Free member"}
            </p>
          </div>
        </div>
      </div>
    </div>
  ) : null;

export default ProfilePage;
