export type FormatCategory =
  | "documents"
  | "spreadsheets"
  | "presentations"
  | "pdf"
  | "images"
  | "text"
  | "web"
  | "code"
  | "data"
  | "audio"
  | "video"
  | "archives";

export interface FileFormatInfo {
  id: string;
  name: string;
  extensions: string[];
  mimeTypes: string[];
  category: FormatCategory;
  description: string;
  icon: string;
}

export type SupportLevel = "supported" | "experimental" | "unsupported";

export interface ConversionRoute {
  from: string;
  to: string;
  level: SupportLevel;
  engines: string[];
  isMultiStep: boolean;
  intermediates?: string[];
  notes?: string;
}

export const FORMAT_CATEGORIES: Record<FormatCategory, string> = {
  documents: "Documents",
  spreadsheets: "Spreadsheets",
  presentations: "Presentations",
  pdf: "PDF",
  images: "Images",
  text: "Text",
  web: "Web",
  code: "Source Code",
  data: "Data",
  audio: "Audio",
  video: "Video",
  archives: "Archives",
};

export const FORMATS: Record<string, FileFormatInfo> = {
  pdf: {
    id: "pdf",
    name: "PDF",
    extensions: [".pdf"],
    mimeTypes: ["application/pdf"],
    category: "pdf",
    description: "Portable Document Format",
    icon: "file-text",
  },
  docx: {
    id: "docx",
    name: "DOCX",
    extensions: [".docx"],
    mimeTypes: [
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ],
    category: "documents",
    description: "Microsoft Word Document",
    icon: "file-text",
  },
  doc: {
    id: "doc",
    name: "DOC",
    extensions: [".doc"],
    mimeTypes: ["application/msword"],
    category: "documents",
    description: "Microsoft Word Document (Legacy)",
    icon: "file-text",
  },
  odt: {
    id: "odt",
    name: "ODT",
    extensions: [".odt"],
    mimeTypes: ["application/vnd.oasis.opendocument.text"],
    category: "documents",
    description: "OpenDocument Text",
    icon: "file-text",
  },
  rtf: {
    id: "rtf",
    name: "RTF",
    extensions: [".rtf"],
    mimeTypes: ["application/rtf"],
    category: "documents",
    description: "Rich Text Format",
    icon: "file-text",
  },
  xlsx: {
    id: "xlsx",
    name: "XLSX",
    extensions: [".xlsx"],
    mimeTypes: [
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ],
    category: "spreadsheets",
    description: "Microsoft Excel Spreadsheet",
    icon: "table",
  },
  xls: {
    id: "xls",
    name: "XLS",
    extensions: [".xls"],
    mimeTypes: ["application/vnd.ms-excel"],
    category: "spreadsheets",
    description: "Microsoft Excel Spreadsheet (Legacy)",
    icon: "table",
  },
  ods: {
    id: "ods",
    name: "ODS",
    extensions: [".ods"],
    mimeTypes: ["application/vnd.oasis.opendocument.spreadsheet"],
    category: "spreadsheets",
    description: "OpenDocument Spreadsheet",
    icon: "table",
  },
  csv: {
    id: "csv",
    name: "CSV",
    extensions: [".csv"],
    mimeTypes: ["text/csv"],
    category: "data",
    description: "Comma-Separated Values",
    icon: "table",
  },
  tsv: {
    id: "tsv",
    name: "TSV",
    extensions: [".tsv"],
    mimeTypes: ["text/tab-separated-values"],
    category: "data",
    description: "Tab-Separated Values",
    icon: "table",
  },
  pptx: {
    id: "pptx",
    name: "PPTX",
    extensions: [".pptx"],
    mimeTypes: [
      "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    ],
    category: "presentations",
    description: "Microsoft PowerPoint Presentation",
    icon: "presentation",
  },
  ppt: {
    id: "ppt",
    name: "PPT",
    extensions: [".ppt"],
    mimeTypes: ["application/vnd.ms-powerpoint"],
    category: "presentations",
    description: "Microsoft PowerPoint Presentation (Legacy)",
    icon: "presentation",
  },
  odp: {
    id: "odp",
    name: "ODP",
    extensions: [".odp"],
    mimeTypes: ["application/vnd.oasis.opendocument.presentation"],
    category: "presentations",
    description: "OpenDocument Presentation",
    icon: "presentation",
  },
  png: {
    id: "png",
    name: "PNG",
    extensions: [".png"],
    mimeTypes: ["image/png"],
    category: "images",
    description: "Portable Network Graphics",
    icon: "image",
  },
  jpg: {
    id: "jpg",
    name: "JPG",
    extensions: [".jpg", ".jpeg"],
    mimeTypes: ["image/jpeg"],
    category: "images",
    description: "JPEG Image",
    icon: "image",
  },
  webp: {
    id: "webp",
    name: "WEBP",
    extensions: [".webp"],
    mimeTypes: ["image/webp"],
    category: "images",
    description: "WebP Image",
    icon: "image",
  },
  tiff: {
    id: "tiff",
    name: "TIFF",
    extensions: [".tiff", ".tif"],
    mimeTypes: ["image/tiff"],
    category: "images",
    description: "Tagged Image File Format",
    icon: "image",
  },
  bmp: {
    id: "bmp",
    name: "BMP",
    extensions: [".bmp"],
    mimeTypes: ["image/bmp"],
    category: "images",
    description: "Bitmap Image",
    icon: "image",
  },
  html: {
    id: "html",
    name: "HTML",
    extensions: [".html", ".htm"],
    mimeTypes: ["text/html"],
    category: "web",
    description: "HyperText Markup Language",
    icon: "globe",
  },
  md: {
    id: "md",
    name: "Markdown",
    extensions: [".md", ".markdown"],
    mimeTypes: ["text/markdown"],
    category: "web",
    description: "Markdown Document",
    icon: "file-text",
  },
  txt: {
    id: "txt",
    name: "TXT",
    extensions: [".txt"],
    mimeTypes: ["text/plain"],
    category: "text",
    description: "Plain Text",
    icon: "file-text",
  },
  json: {
    id: "json",
    name: "JSON",
    extensions: [".json"],
    mimeTypes: ["application/json"],
    category: "data",
    description: "JavaScript Object Notation",
    icon: "braces",
  },
  xml: {
    id: "xml",
    name: "XML",
    extensions: [".xml"],
    mimeTypes: ["application/xml", "text/xml"],
    category: "data",
    description: "Extensible Markup Language",
    icon: "code",
  },
  py: {
    id: "py",
    name: "Python",
    extensions: [".py"],
    mimeTypes: ["text/x-python"],
    category: "code",
    description: "Python Source Code",
    icon: "code",
  },
  js: {
    id: "js",
    name: "JavaScript",
    extensions: [".js", ".mjs"],
    mimeTypes: ["text/javascript"],
    category: "code",
    description: "JavaScript Source Code",
    icon: "code",
  },
  ts: {
    id: "ts",
    name: "TypeScript",
    extensions: [".ts", ".tsx"],
    mimeTypes: ["text/typescript"],
    category: "code",
    description: "TypeScript Source Code",
    icon: "code",
  },
  java: {
    id: "java",
    name: "Java",
    extensions: [".java"],
    mimeTypes: ["text/x-java-source"],
    category: "code",
    description: "Java Source Code",
    icon: "code",
  },
  cpp: {
    id: "cpp",
    name: "C++",
    extensions: [".cpp", ".cc", ".cxx", ".hpp", ".h"],
    mimeTypes: ["text/x-c++src"],
    category: "code",
    description: "C++ Source Code",
    icon: "code",
  },
  cs: {
    id: "cs",
    name: "C#",
    extensions: [".cs"],
    mimeTypes: ["text/x-csharp"],
    category: "code",
    description: "C# Source Code",
    icon: "code",
  },
  php: {
    id: "php",
    name: "PHP",
    extensions: [".php"],
    mimeTypes: ["text/x-php"],
    category: "code",
    description: "PHP Source Code",
    icon: "code",
  },
  css: {
    id: "css",
    name: "CSS",
    extensions: [".css"],
    mimeTypes: ["text/css"],
    category: "code",
    description: "Cascading Style Sheets",
    icon: "code",
  },
  sql: {
    id: "sql",
    name: "SQL",
    extensions: [".sql"],
    mimeTypes: ["text/x-sql"],
    category: "code",
    description: "SQL Script",
    icon: "database",
  },
};

export function getFormatsByCategory(
  category: FormatCategory
): FileFormatInfo[] {
  return Object.values(FORMATS).filter((f) => f.category === category);
}

export function getFormatById(id: string): FileFormatInfo | undefined {
  return FORMATS[id];
}

export function getFormatByExtension(ext: string): FileFormatInfo | undefined {
  const normalized = ext.toLowerCase().startsWith(".")
    ? ext.toLowerCase()
    : `.${ext.toLowerCase()}`;
  return Object.values(FORMATS).find((f) => f.extensions.includes(normalized));
}
