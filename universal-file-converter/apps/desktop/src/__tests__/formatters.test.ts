import { describe, it, expect } from "vitest";
import {
  truncateFilename,
  getFileExtension,
  getFileBaseName,
} from "@/utils/formatters";

describe("truncateFilename", () => {
  it("returns short names unchanged", () => {
    expect(truncateFilename("file.txt")).toBe("file.txt");
  });

  it("truncates long names preserving extension", () => {
    const name = "a-very-long-filename-that-exceeds-the-limit.pdf";
    const result = truncateFilename(name, 30);
    expect(result.length).toBeLessThanOrEqual(30);
    expect(result).toContain("...");
    expect(result).toMatch(/\.pdf$/);
  });

  it("handles files without extension", () => {
    const name = "a-very-long-filename-without-extension";
    const result = truncateFilename(name, 20);
    expect(result.length).toBeLessThanOrEqual(20);
    expect(result).toContain("...");
  });

  it("respects custom maxLength", () => {
    const name = "document.txt";
    expect(truncateFilename(name, 5)).toContain("...");
  });

  it("uses default maxLength of 40", () => {
    const name = "x".repeat(41) + ".txt";
    const result = truncateFilename(name);
    expect(result.length).toBeLessThanOrEqual(40);
  });
});

describe("getFileExtension", () => {
  it("returns lowercase extension with dot", () => {
    expect(getFileExtension("photo.JPG")).toBe(".jpg");
    expect(getFileExtension("doc.PDF")).toBe(".pdf");
  });

  it("returns empty string for no extension", () => {
    expect(getFileExtension("Makefile")).toBe("");
  });

  it("handles multiple dots", () => {
    expect(getFileExtension("archive.tar.gz")).toBe(".gz");
  });
});

describe("getFileBaseName", () => {
  it("strips extension", () => {
    expect(getFileBaseName("photo.jpg")).toBe("photo");
  });

  it("returns full name when no extension", () => {
    expect(getFileBaseName("Makefile")).toBe("Makefile");
  });

  it("strips only the last extension", () => {
    expect(getFileBaseName("archive.tar.gz")).toBe("archive.tar");
  });
});
