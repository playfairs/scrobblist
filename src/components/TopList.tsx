import React, { useEffect, useState } from "react";
import { ExternalLink, Disc3, Music2, UserRound } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";

export type TopKind = "artists" | "albums" | "tracks";
export type TopPeriod = "7day" | "1month" | "12month" | "overall";

interface TopItem {
  name: string;
  artist: string | null;
  url: string;
  image_url: string | null;
  playcount: number;
}

const periods: { value: TopPeriod; label: string }[] = [
  { value: "7day", label: "Weekly" },
  { value: "1month", label: "Monthly" },
  { value: "12month", label: "Yearly" },
  { value: "overall", label: "All time" },
];

const placeholder =
  "https://lastfm.freetls.fastly.net/i/u/300x300/2a96cbd8b46e442fc41c2b86b821562f.png";
const icons = { artists: UserRound, albums: Disc3, tracks: Music2 };

const TopList: React.FC<{ kind: TopKind; title?: string }> = ({
  kind,
  title,
}) => {
  const [period, setPeriod] = useState<TopPeriod>("7day");
  const [items, setItems] = useState<TopItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const Icon = icons[kind];

  useEffect(() => {
    let active = true;
    setLoading(true);
    setError(null);
    invoke<TopItem[]>("get_top_items", { kind, period, limit: 50 })
      .then((nextItems) => {
        if (active) setItems(nextItems);
      })
      .catch((err) => {
        if (active) setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        if (active) setLoading(false);
      });

    return () => {
      active = false;
    };
  }, [kind, period]);

  return (
    <div className="min-h-full bg-[#0a0a0a] px-5 py-6 sm:px-8 lg:px-10">
      <div className="mx-auto max-w-5xl">
        <header className="mb-8 flex flex-wrap items-end justify-between gap-5 border-b border-[#242424] pb-7">
          <div>
            <p className="mb-2 text-xs font-semibold uppercase tracking-[0.22em] text-[#e31b12]">
              Top charts
            </p>
            <h1 className="text-3xl font-bold tracking-tight text-white">
              {title || kind[0].toUpperCase() + kind.slice(1)}
            </h1>
            <p className="mt-2 text-sm text-gray-500">
              {periods.find((option) => option.value === period)?.label} plays
              from Last.fm.
            </p>
          </div>
          <div className="flex rounded-md border border-[#292929] bg-[#111] p-1">
            {periods.map((option) => (
              <button
                key={option.value}
                onClick={() => setPeriod(option.value)}
                className={`px-3 py-2 text-xs font-semibold transition-colors ${period === option.value ? "bg-[#3a1311] text-white" : "text-gray-500 hover:text-white"}`}
              >
                {option.label}
              </button>
            ))}
          </div>
        </header>
        {error && (
          <div className="mb-5 border border-red-900/60 bg-red-950/30 px-4 py-3 text-sm text-red-300">
            {error}
          </div>
        )}
        {loading ? (
          <div className="py-16 text-center text-sm text-gray-500">
            Loading {kind}...
          </div>
        ) : items.length === 0 ? (
          <div className="border border-dashed border-[#303030] py-16 text-center text-gray-500">
            No {kind} found for this period.
          </div>
        ) : (
          <div className="divide-y divide-[#202020] border-y border-[#202020]">
            {items.map((item, index) => (
              <div
                key={`${item.name}-${item.artist || ""}`}
                className="group flex min-h-[78px] items-center gap-4 px-3 py-3 transition-colors hover:bg-[#151515]"
              >
                <span className="w-7 text-center text-sm text-gray-600">
                  {String(index + 1).padStart(2, "0")}
                </span>
                <img
                  src={item.image_url || placeholder}
                  alt=""
                  className="h-12 w-12 rounded-md object-cover"
                  onError={(event) => {
                    event.currentTarget.src = placeholder;
                  }}
                />
                <div className="min-w-0 flex-1">
                  <button
                    onClick={() => void open(item.url)}
                    className="flex max-w-full items-center gap-2 truncate text-left font-semibold text-white hover:text-[#e31b12]"
                  >
                    <Icon size={15} className="shrink-0 text-[#e31b12]" />
                    {item.name}
                    <ExternalLink
                      size={13}
                      className="shrink-0 text-gray-700"
                    />
                  </button>
                  {item.artist && (
                    <p className="mt-1 truncate text-sm text-gray-500">
                      {item.artist}
                    </p>
                  )}
                </div>
                <span className="whitespace-nowrap text-xs text-gray-600">
                  {item.playcount.toLocaleString()} plays
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default TopList;
