import { useEffect } from "react";
import { BrowserRouter, Routes, Route, useNavigate } from "react-router-dom";
import { MainLayout } from "@/components/layout/MainLayout";
import { ErrorBoundary } from "@/components/common/ErrorBoundary";
import { HomePage } from "@/pages/HomePage";
import { ConvertPage } from "@/pages/ConvertPage";
import { BatchPage } from "@/pages/BatchPage";
import { HistoryPage } from "@/pages/HistoryPage";
import { FavoritesPage } from "@/pages/FavoritesPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { AboutPage } from "@/pages/AboutPage";
import { useTheme } from "@/hooks/useTheme";

function KeyboardShortcuts() {
  const navigate = useNavigate();

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.ctrlKey || e.metaKey;
      if (mod && e.key === "o") {
        e.preventDefault();
        navigate("/convert");
      }
      if (mod && e.key === "b") {
        e.preventDefault();
        navigate("/batch");
      }
      if (mod && e.key === ",") {
        e.preventDefault();
        navigate("/settings");
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate]);

  return null;
}

export default function App() {
  useTheme();

  return (
    <ErrorBoundary>
      <BrowserRouter>
        <KeyboardShortcuts />
        <Routes>
          <Route element={<MainLayout />}>
            <Route path="/" element={<HomePage />} />
            <Route path="/convert" element={<ConvertPage />} />
            <Route path="/batch" element={<BatchPage />} />
            <Route path="/history" element={<HistoryPage />} />
            <Route path="/favorites" element={<FavoritesPage />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="/about" element={<AboutPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ErrorBoundary>
  );
}
