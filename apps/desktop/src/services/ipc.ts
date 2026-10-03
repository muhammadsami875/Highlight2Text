import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { AppInfo, ImportedPage } from "../types/bindings";

export type { AppInfo, ImportedPage } from "../types/bindings";

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function importImage(path: string): Promise<ImportedPage> {
  return invoke<ImportedPage>("import_image", { path });
}

export async function pickImages(): Promise<string[]> {
  const picked = await open({
    multiple: true,
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "tif", "tiff", "bmp", "webp"],
      },
    ],
  });
  if (!picked) return [];
  return Array.isArray(picked) ? picked : [picked];
}

/// Resolve a native file path to a URL the webview can render.
export function assetUrl(absPath: string): string {
  return convertFileSrc(absPath);
}
