import { describe, it, expect } from "vitest";
import {
  getOutputFormats,
  isConversionSupported,
  getEngineName,
  getAllInputFormats,
} from "@/services/formatMatrix";

describe("getOutputFormats", () => {
  it("returns output formats for png", () => {
    const formats = getOutputFormats("png");
    expect(formats.length).toBeGreaterThan(0);
    const formatNames = formats.map((f) => f.format);
    expect(formatNames).toContain("jpg");
    expect(formatNames).toContain("webp");
    expect(formatNames).toContain("pdf");
  });

  it("returns output formats for csv", () => {
    const formats = getOutputFormats("csv");
    const formatNames = formats.map((f) => f.format);
    expect(formatNames).toContain("xlsx");
    expect(formatNames).toContain("pdf");
  });

  it("returns empty array for unknown format", () => {
    expect(getOutputFormats("xyz123")).toEqual([]);
  });

  it("includes support level info", () => {
    const formats = getOutputFormats("docx");
    const pdfEntry = formats.find((f) => f.format === "pdf");
    expect(pdfEntry).toBeDefined();
    expect(pdfEntry!.level).toBe("supported");
  });

  it("includes notes when present", () => {
    const formats = getOutputFormats("pdf");
    const csvEntry = formats.find((f) => f.format === "csv");
    expect(csvEntry?.notes).toBeDefined();
  });
});

describe("isConversionSupported", () => {
  it("returns support level for known conversion", () => {
    expect(isConversionSupported("png", "jpg")).toBe("supported");
    expect(isConversionSupported("docx", "pdf")).toBe("supported");
  });

  it("returns null for unsupported conversion", () => {
    expect(isConversionSupported("png", "docx")).toBeNull();
    expect(isConversionSupported("xyz", "abc")).toBeNull();
  });

  it("detects experimental conversions", () => {
    expect(isConversionSupported("html", "docx")).toBe("experimental");
  });
});

describe("getEngineName", () => {
  it("returns engine for known route", () => {
    expect(getEngineName("png", "jpg")).toBe("image_convert");
    expect(getEngineName("docx", "pdf")).toBe("libreoffice");
  });

  it("returns null for unknown route", () => {
    expect(getEngineName("png", "docx")).toBeNull();
  });
});

describe("getAllInputFormats", () => {
  it("returns all registered input formats", () => {
    const formats = getAllInputFormats();
    expect(formats).toContain("png");
    expect(formats).toContain("jpg");
    expect(formats).toContain("pdf");
    expect(formats).toContain("docx");
    expect(formats).toContain("csv");
    expect(formats).toContain("md");
    expect(formats).toContain("py");
    expect(formats.length).toBeGreaterThan(15);
  });
});
