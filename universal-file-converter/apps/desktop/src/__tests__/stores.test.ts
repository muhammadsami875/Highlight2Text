import { describe, it, expect, beforeEach } from "vitest";
import { useConversionStore } from "@/stores/conversionStore";
import { useHistoryStore } from "@/stores/historyStore";
import { useBatchStore } from "@/stores/batchStore";
import { useToastStore } from "@/stores/toastStore";
import { useSettingsStore } from "@/stores/settingsStore";
import type { DetectedFile, ConversionResult, ConversionJob } from "@/types/conversion";

function makeDetectedFile(overrides: Partial<DetectedFile> = {}): DetectedFile {
  return {
    path: "/test.png",
    name: "test.png",
    extension: "png",
    detectedFormat: "png",
    mimeType: "image/png",
    sizeBytes: 1024,
    formatMismatch: false,
    metadata: {},
    ...overrides,
  };
}

function makeResult(overrides: Partial<ConversionResult> = {}): ConversionResult {
  return {
    jobId: "1",
    success: true,
    outputPath: "/out.pdf",
    outputSize: 2048,
    duration: 100,
    warnings: [],
    error: null,
    ...overrides,
  };
}

function makeJob(overrides: Partial<ConversionJob> = {}): ConversionJob {
  return {
    id: "j1",
    inputFile: makeDetectedFile(),
    outputFormat: "pdf",
    outputPath: "/out.pdf",
    options: {},
    plan: null,
    status: "queued",
    progress: 0,
    progressMessage: "",
    startedAt: null,
    completedAt: null,
    error: null,
    warnings: [],
    ...overrides,
  };
}

describe("conversionStore", () => {
  beforeEach(() => {
    useConversionStore.getState().reset();
  });

  it("starts with null input file", () => {
    expect(useConversionStore.getState().inputFile).toBeNull();
  });

  it("sets input file and clears output format", () => {
    useConversionStore.getState().setOutputFormat("pdf");
    const file = makeDetectedFile();
    useConversionStore.getState().setInputFile(file);
    const state = useConversionStore.getState();
    expect(state.inputFile).toEqual(file);
    expect(state.outputFormat).toBeNull();
    expect(state.outputFormats).toEqual([]);
  });

  it("sets and updates conversion options", () => {
    useConversionStore.getState().setOptions({ dpi: 300, imageQuality: 95 });
    const state = useConversionStore.getState();
    expect(state.options.dpi).toBe(300);
    expect(state.options.imageQuality).toBe(95);
    expect(state.options.pageSize).toBe("A4");
  });

  it("resets options to defaults", () => {
    useConversionStore.getState().setOptions({ dpi: 300 });
    useConversionStore.getState().resetOptions();
    expect(useConversionStore.getState().options.dpi).toBe(150);
  });

  it("resets entire store", () => {
    useConversionStore.getState().setOutputFormat("pdf");
    useConversionStore.getState().setIsConverting(true);
    useConversionStore.getState().reset();
    const state = useConversionStore.getState();
    expect(state.outputFormat).toBeNull();
    expect(state.isConverting).toBe(false);
  });
});

describe("historyStore", () => {
  beforeEach(() => {
    useHistoryStore.setState({ history: [], favorites: [], recentPairs: [] });
  });

  it("adds to history (prepends)", () => {
    useHistoryStore.getState().addToHistory(makeResult({ jobId: "1" }));
    expect(useHistoryStore.getState().history).toHaveLength(1);
    expect(useHistoryStore.getState().history[0].jobId).toBe("1");
  });

  it("caps history at 500 entries", () => {
    const items = Array.from({ length: 501 }, (_, i) =>
      makeResult({ jobId: String(i) })
    );
    useHistoryStore.setState({ history: items.slice(0, 500) });
    useHistoryStore.getState().addToHistory(makeResult({ jobId: "500" }));
    expect(useHistoryStore.getState().history).toHaveLength(500);
    expect(useHistoryStore.getState().history[0].jobId).toBe("500");
  });

  it("clears history", () => {
    useHistoryStore.getState().addToHistory(makeResult());
    useHistoryStore.getState().clearHistory();
    expect(useHistoryStore.getState().history).toEqual([]);
  });

  it("manages favorites", () => {
    useHistoryStore.getState().addFavorite({ id: "fav1", fromFormat: "png", toFormat: "pdf", label: "PNG to PDF" });
    expect(useHistoryStore.getState().favorites).toHaveLength(1);
    useHistoryStore.getState().removeFavorite("fav1");
    expect(useHistoryStore.getState().favorites).toHaveLength(0);
  });

  it("manages recent pairs with deduplication", () => {
    useHistoryStore.getState().addRecentPair("png", "pdf");
    useHistoryStore.getState().addRecentPair("jpg", "webp");
    useHistoryStore.getState().addRecentPair("png", "pdf");
    const pairs = useHistoryStore.getState().recentPairs;
    expect(pairs).toHaveLength(2);
    expect(pairs[0]).toEqual({ from: "png", to: "pdf" });
  });

  it("caps recent pairs at 10", () => {
    for (let i = 0; i < 15; i++) {
      useHistoryStore.getState().addRecentPair(`fmt${i}`, `out${i}`);
    }
    expect(useHistoryStore.getState().recentPairs).toHaveLength(10);
  });
});

describe("batchStore", () => {
  beforeEach(() => {
    useBatchStore.getState().clearAll();
  });

  it("adds and removes jobs", () => {
    const jobs = [
      makeJob({ id: "j1" }),
      makeJob({ id: "j2", outputFormat: "webp" }),
    ];
    useBatchStore.getState().addJobs(jobs);
    expect(useBatchStore.getState().jobs).toHaveLength(2);

    useBatchStore.getState().removeJob("j1");
    expect(useBatchStore.getState().jobs).toHaveLength(1);
    expect(useBatchStore.getState().jobs[0].id).toBe("j2");
  });

  it("updates job fields", () => {
    useBatchStore.getState().addJobs([makeJob({ id: "j1" })]);
    useBatchStore.getState().updateJob("j1", { status: "converting", progress: 50 });
    const job = useBatchStore.getState().jobs[0];
    expect(job.status).toBe("converting");
    expect(job.progress).toBe(50);
  });

  it("clears completed jobs", () => {
    useBatchStore.getState().addJobs([
      makeJob({ id: "j1", status: "completed", progress: 100 }),
      makeJob({ id: "j2", status: "queued" }),
    ]);
    useBatchStore.getState().clearCompleted();
    expect(useBatchStore.getState().jobs).toHaveLength(1);
    expect(useBatchStore.getState().jobs[0].id).toBe("j2");
  });
});

describe("toastStore", () => {
  beforeEach(() => {
    useToastStore.setState({ toasts: [] });
  });

  it("adds toasts with auto-generated id", () => {
    useToastStore.getState().addToast("success", "Done!");
    const toasts = useToastStore.getState().toasts;
    expect(toasts).toHaveLength(1);
    expect(toasts[0].type).toBe("success");
    expect(toasts[0].message).toBe("Done!");
    expect(toasts[0].id).toBeDefined();
  });

  it("removes toast by id", () => {
    useToastStore.getState().addToast("error", "Failed", 0);
    const id = useToastStore.getState().toasts[0].id;
    useToastStore.getState().removeToast(id);
    expect(useToastStore.getState().toasts).toHaveLength(0);
  });
});

describe("settingsStore", () => {
  beforeEach(() => {
    useSettingsStore.getState().resetToDefaults();
    useSettingsStore.getState().markClean();
  });

  it("starts clean", () => {
    expect(useSettingsStore.getState().isDirty).toBe(false);
  });

  it("marks dirty on nested update", () => {
    useSettingsStore.getState().updateSettings("general.theme", "dark");
    expect(useSettingsStore.getState().isDirty).toBe(true);
    expect(useSettingsStore.getState().settings.general.theme).toBe("dark");
  });

  it("resets to defaults", () => {
    useSettingsStore.getState().updateSettings("general.theme", "dark");
    useSettingsStore.getState().resetToDefaults();
    expect(useSettingsStore.getState().settings.general.theme).toBe("system");
  });

  it("marks clean", () => {
    useSettingsStore.getState().updateSettings("general.theme", "dark");
    useSettingsStore.getState().markClean();
    expect(useSettingsStore.getState().isDirty).toBe(false);
  });
});
