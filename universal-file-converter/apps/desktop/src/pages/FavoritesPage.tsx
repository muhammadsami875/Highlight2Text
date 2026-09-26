import { Star, ArrowRight, Trash2 } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useHistoryStore } from "@/stores/historyStore";
import { getFormatById } from "@/types/formats";
import { t } from "@/i18n";

export function FavoritesPage() {
  const navigate = useNavigate();
  const { favorites, removeFavorite } = useHistoryStore();

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-bold text-surface-900 dark:text-surface-100">
        {t("favorites.title")}
      </h2>

      {favorites.length === 0 ? (
        <div className="text-center py-16">
          <Star
            size={48}
            className="mx-auto mb-3 text-surface-300 dark:text-surface-600"
          />
          <p className="text-surface-400 dark:text-surface-500">
            {t("favorites.noFavorites")}
          </p>
          <p className="text-sm text-surface-300 dark:text-surface-600 mt-1">
            {t("favorites.addHint")}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          {favorites.map((fav) => {
            const fromInfo = getFormatById(fav.fromFormat);
            const toInfo = getFormatById(fav.toFormat);
            return (
              <div key={fav.id} className="card p-4 flex items-center gap-3">
                <button
                  onClick={() => navigate("/convert")}
                  className="flex-1 flex items-center gap-3 text-left hover:opacity-80"
                >
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-surface-700 dark:text-surface-300">
                      {fromInfo?.name ?? fav.fromFormat.toUpperCase()}
                    </span>
                    <ArrowRight
                      size={14}
                      className="text-surface-400 dark:text-surface-500"
                    />
                    <span className="text-sm font-medium text-surface-700 dark:text-surface-300">
                      {toInfo?.name ?? fav.toFormat.toUpperCase()}
                    </span>
                  </div>
                </button>
                <button
                  onClick={() => removeFavorite(fav.id)}
                  className="btn-ghost p-1.5"
                >
                  <Trash2 size={14} className="text-surface-400" />
                </button>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
