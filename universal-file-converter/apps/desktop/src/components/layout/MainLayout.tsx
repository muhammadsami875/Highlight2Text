import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { ToastContainer } from "@/components/common/ToastContainer";

export function MainLayout() {
  return (
    <div className="flex h-screen bg-surface-50 dark:bg-surface-950">
      <Sidebar />
      <main className="flex-1 overflow-y-auto">
        <div className="p-6 max-w-5xl mx-auto">
          <Outlet />
        </div>
      </main>
      <ToastContainer />
    </div>
  );
}
