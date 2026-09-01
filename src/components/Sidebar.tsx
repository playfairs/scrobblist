import React from "react";
import type { Page } from "../App";
import {
  Album,
  BarChart3,
  Disc3,
  Heart,
  ListMusic,
  Settings,
  UserRound,
  Users,
  LayoutDashboard,
} from "lucide-react";

interface SidebarProps {
  currentPage: Page;
  onPageChange: (page: Page) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ currentPage, onPageChange }) => {
  const navItems: { id: Page; label: string; icon: typeof LayoutDashboard }[] =
    [
      { id: "overview", label: "Overview", icon: LayoutDashboard },
      { id: "scrobbles", label: "Scrobbles", icon: ListMusic },
      { id: "artists", label: "Artists", icon: Users },
      { id: "albums", label: "Albums", icon: Album },
      { id: "tracks", label: "Tracks", icon: Disc3 },
      { id: "charts", label: "Charts", icon: BarChart3 },
      { id: "profile", label: "Profile", icon: UserRound },
      { id: "loved", label: "Loved", icon: Heart },
      { id: "settings", label: "Settings", icon: Settings },
    ];

  return (
    <aside className="flex w-64 shrink-0 flex-col border-r border-[#292929] bg-[#101010]">
      <div className="border-b border-[#292929] px-5 pb-5 pt-6">
        <div className="mb-5 flex items-center gap-2">
          <span className="h-2.5 w-2.5 rounded-full bg-[#e31b12] shadow-[0_0_14px_rgba(227,27,18,0.75)]" />
          <h1 className="text-xl font-bold tracking-tight text-white">
            Scrobblist
          </h1>
        </div>
        <p className="text-[10px] font-semibold uppercase tracking-[0.24em] text-gray-600">
          Last.fm companion
        </p>
      </div>
      <nav className="flex-1 px-3 py-5">
        <p className="mb-3 px-3 text-[10px] font-semibold uppercase tracking-[0.22em] text-gray-600">
          Library
        </p>
        <ul className="space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <li key={item.id}>
                <button
                  onClick={() => onPageChange(item.id)}
                  className={`group relative w-full rounded-md px-3 py-2.5 text-left text-sm transition-colors ${
                    currentPage === item.id
                      ? "bg-[#2a1110] font-semibold text-white"
                      : "text-gray-500 hover:bg-[#1b1b1b] hover:text-white"
                  }`}
                >
                  {currentPage === item.id && (
                    <span className="absolute bottom-2 left-0 top-2 w-0.5 rounded-full bg-[#e31b12]" />
                  )}
                  <Icon
                    size={16}
                    className={`mr-3 inline-block align-[-3px] ${currentPage === item.id ? "text-[#e31b12]" : "text-gray-600 group-hover:text-gray-300"}`}
                  />
                  {item.label}
                </button>
              </li>
            );
          })}
        </ul>
      </nav>
    </aside>
  );
};

export default Sidebar;
