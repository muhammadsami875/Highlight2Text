import { NavLink } from "react-router-dom";
import { clsx } from "clsx";
import {
  Home,
  ArrowRightLeft,
  Layers,
  Clock,
  Star,
  Settings,
  Info,
} from "lucide-react";
import { t } from "@/i18n";

const NAV_ITEMS = [
  { to: "/", icon: Home, label: "nav.home" },
  { to: "/convert", icon: ArrowRightLeft, label: "nav.convert" },
  { to: "/batch", icon: Layers, label: "nav.batch" },
  { to: "/history", icon: Clock, label: "nav.history" },
  { to: "/favorites", icon: Star, label: "nav.favorites" },
  { to: "/settings", icon: Settings, label: "nav.settings" },
  { to: "/about", icon: Info, label: "nav.about" },
];

export function Sidebar() {
  return (
    <aside className="w-56 h-screen flex flex-col border-r border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-900/50">
      <div className="px-5 py-4 border-b border-surface-200 dark:border-surface-700">
        <h1 className="text-sm font-bold text-surface-900 dark:text-surface-100 tracking-tight">
          Universal File
          <br />
          Converter
        </h1>
      </div>
      <nav className="flex-1 px-3 py-3 space-y-0.5 overflow-y-auto">
        {NAV_ITEMS.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              clsx("nav-item", isActive && "nav-item-active")
            }
          >
            <Icon size={18} />
            <span>{t(label)}</span>
          </NavLink>
        ))}
      </nav>
      <div className="px-4 py-3 border-t border-surface-200 dark:border-surface-700">
        <p className="text-xs text-surface-400 dark:text-surface-500">
          v{t("app.version")}
        </p>
      </div>
    </aside>
  );
}
