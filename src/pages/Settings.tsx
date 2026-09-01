import React, { useEffect, useState } from "react";

const Settings: React.FC = () => {
  const [compactRows, setCompactRows] = useState(
    () => localStorage.getItem("scrobblist-compact-rows") === "true",
  );
  useEffect(() => {
    localStorage.setItem("scrobblist-compact-rows", String(compactRows));
  }, [compactRows]);
  return (
    <div className="min-h-full bg-[#0a0a0a] px-5 py-6 sm:px-8 lg:px-10">
      <div className="mx-auto max-w-3xl">
        <header className="mb-8 border-b border-[#242424] pb-7">
          <p className="mb-2 text-xs font-semibold uppercase tracking-[0.22em] text-[#e31b12]">
            Preferences
          </p>
          <h1 className="text-3xl font-bold tracking-tight text-white">
            Settings
          </h1>
          <p className="mt-2 text-sm text-gray-500">
            Personalize how Scrobblist presents your listening history.
          </p>
        </header>
        <section className="divide-y divide-[#242424] border-y border-[#242424]">
          <label className="flex cursor-pointer items-center justify-between gap-6 py-5">
            <span>
              <span className="block font-semibold text-white">
                Compact track rows
              </span>
              <span className="mt-1 block text-sm text-gray-500">
                Use a denser layout for listening history.
              </span>
            </span>
            <input
              type="checkbox"
              checked={compactRows}
              onChange={(event) => setCompactRows(event.target.checked)}
              className="h-5 w-5 accent-[#e31b12]"
            />
          </label>
          <div className="py-5">
            <p className="font-semibold text-white">Connected account</p>
            <p className="mt-1 text-sm text-gray-500">
              Last.fm session is stored locally and used for authenticated
              requests.
            </p>
          </div>
        </section>
      </div>
    </div>
  );
};

export default Settings;
