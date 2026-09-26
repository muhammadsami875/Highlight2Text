import { BrowserRouter, Routes, Route } from "react-router-dom";
import { MainLayout } from "@/components/layout/MainLayout";
import { HomePage } from "@/pages/HomePage";
import { ConvertPage } from "@/pages/ConvertPage";
import { BatchPage } from "@/pages/BatchPage";
import { HistoryPage } from "@/pages/HistoryPage";
import { FavoritesPage } from "@/pages/FavoritesPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { AboutPage } from "@/pages/AboutPage";

export default function App() {
  return (
    <BrowserRouter>
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
  );
}
