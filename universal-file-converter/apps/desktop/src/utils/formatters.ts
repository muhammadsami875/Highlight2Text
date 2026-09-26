export function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function truncateFilename(name: string, maxLength: number = 40): string {
  if (name.length <= maxLength) return name;
  const ext = name.lastIndexOf(".");
  if (ext === -1) return name.slice(0, maxLength - 3) + "...";
  const extension = name.slice(ext);
  const baseName = name.slice(0, ext);
  const maxBase = maxLength - extension.length - 3;
  if (maxBase <= 0) return name.slice(0, maxLength - 3) + "...";
  return baseName.slice(0, maxBase) + "..." + extension;
}

export function getFileExtension(filename: string): string {
  const dot = filename.lastIndexOf(".");
  return dot === -1 ? "" : filename.slice(dot).toLowerCase();
}

export function getFileBaseName(filename: string): string {
  const dot = filename.lastIndexOf(".");
  return dot === -1 ? filename : filename.slice(0, dot);
}
