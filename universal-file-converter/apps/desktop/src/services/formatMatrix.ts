import type { SupportLevel } from "@/types/formats";

interface MatrixEntry {
  level: SupportLevel;
  engine: string;
  notes?: string;
}

const MATRIX: Record<string, Record<string, MatrixEntry>> = {
  pdf: {
    docx: { level: "supported", engine: "pdf_to_docx", notes: "Text extraction - layout is approximate" },
    txt: { level: "supported", engine: "pdf_text" },
    html: { level: "experimental", engine: "pdf_text" },
    png: { level: "supported", engine: "pdf_image" },
    jpg: { level: "supported", engine: "pdf_image" },
    csv: { level: "supported", engine: "pdf_table", notes: "Table extraction quality varies" },
    xlsx: { level: "supported", engine: "pdf_to_xlsx", notes: "Text extracted as rows" },
  },
  docx: {
    pdf: { level: "supported", engine: "libreoffice" },
    html: { level: "supported", engine: "libreoffice" },
    txt: { level: "supported", engine: "libreoffice" },
    odt: { level: "supported", engine: "libreoffice" },
  },
  doc: {
    pdf: { level: "supported", engine: "libreoffice" },
    docx: { level: "supported", engine: "libreoffice" },
    odt: { level: "supported", engine: "libreoffice" },
  },
  odt: {
    pdf: { level: "supported", engine: "libreoffice" },
    docx: { level: "supported", engine: "libreoffice" },
  },
  rtf: {
    pdf: { level: "supported", engine: "libreoffice" },
    docx: { level: "supported", engine: "libreoffice" },
  },
  txt: {
    pdf: { level: "supported", engine: "text_to_pdf" },
    docx: { level: "supported", engine: "text_to_docx" },
  },
  md: {
    pdf: { level: "supported", engine: "text_to_pdf" },
    docx: { level: "supported", engine: "text_to_docx" },
    html: { level: "supported", engine: "markdown_to_html" },
  },
  html: {
    pdf: { level: "supported", engine: "html_to_pdf" },
    docx: { level: "experimental", engine: "pandoc" },
    txt: { level: "supported", engine: "html_to_txt" },
    png: { level: "experimental", engine: "html_to_pdf" },
  },
  xlsx: {
    pdf: { level: "supported", engine: "libreoffice" },
    csv: { level: "supported", engine: "calamine" },
    tsv: { level: "supported", engine: "calamine" },
  },
  xls: {
    pdf: { level: "supported", engine: "libreoffice" },
    xlsx: { level: "supported", engine: "libreoffice" },
  },
  ods: {
    pdf: { level: "supported", engine: "libreoffice" },
    xlsx: { level: "supported", engine: "libreoffice" },
  },
  csv: {
    xlsx: { level: "supported", engine: "xlsx_writer" },
    pdf: { level: "supported", engine: "libreoffice" },
  },
  pptx: {
    pdf: { level: "supported", engine: "libreoffice" },
    png: { level: "supported", engine: "libreoffice+pdf_image" },
    jpg: { level: "supported", engine: "libreoffice+pdf_image" },
    txt: { level: "experimental", engine: "libreoffice" },
  },
  ppt: {
    pdf: { level: "supported", engine: "libreoffice" },
    pptx: { level: "supported", engine: "libreoffice" },
  },
  odp: {
    pdf: { level: "supported", engine: "libreoffice" },
  },
  png: {
    jpg: { level: "supported", engine: "image_convert" },
    webp: { level: "supported", engine: "image_convert" },
    pdf: { level: "supported", engine: "image_to_pdf" },
    bmp: { level: "supported", engine: "image_convert" },
    tiff: { level: "supported", engine: "image_convert" },
  },
  jpg: {
    png: { level: "supported", engine: "image_convert" },
    webp: { level: "supported", engine: "image_convert" },
    pdf: { level: "supported", engine: "image_to_pdf" },
    bmp: { level: "supported", engine: "image_convert" },
    tiff: { level: "supported", engine: "image_convert" },
  },
  webp: {
    png: { level: "supported", engine: "image_convert" },
    jpg: { level: "supported", engine: "image_convert" },
    pdf: { level: "supported", engine: "image_to_pdf" },
  },
  tiff: {
    png: { level: "supported", engine: "image_convert" },
    jpg: { level: "supported", engine: "image_convert" },
    pdf: { level: "supported", engine: "image_to_pdf" },
  },
  bmp: {
    png: { level: "supported", engine: "image_convert" },
    jpg: { level: "supported", engine: "image_convert" },
    webp: { level: "supported", engine: "image_convert" },
    pdf: { level: "supported", engine: "image_to_pdf" },
  },
  py: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  js: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  ts: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  java: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  cpp: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  cs: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  php: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  css: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  sql: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  json: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
  xml: { pdf: { level: "supported", engine: "code_to_pdf" }, html: { level: "supported", engine: "code_to_pdf" }, txt: { level: "supported", engine: "passthrough" } },
};

export function getOutputFormats(inputFormat: string): Array<{ format: string; level: SupportLevel; notes?: string }> {
  const entry = MATRIX[inputFormat];
  if (!entry) return [];
  return Object.entries(entry).map(([format, info]) => ({
    format,
    level: info.level,
    notes: info.notes,
  }));
}

export function isConversionSupported(from: string, to: string): SupportLevel | null {
  return MATRIX[from]?.[to]?.level ?? null;
}

export function getEngineName(from: string, to: string): string | null {
  return MATRIX[from]?.[to]?.engine ?? null;
}

export function getAllInputFormats(): string[] {
  return Object.keys(MATRIX);
}
