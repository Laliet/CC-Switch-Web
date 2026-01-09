import { renderHook, act, waitFor } from "@testing-library/react";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { useDirectorySettings } from "@/hooks/useDirectorySettings";
import type { SettingsFormState } from "@/hooks/useSettingsForm";

const getAppConfigDirOverrideMock = vi.hoisted(() => vi.fn());
const getConfigDirMock = vi.hoisted(() => vi.fn());
const selectConfigDirectoryMock = vi.hoisted(() => vi.fn());
const toastErrorMock = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api", () => ({
  settingsApi: {
    getAppConfigDirOverride: getAppConfigDirOverrideMock,
    getConfigDir: getConfigDirMock,
    selectConfigDirectory: selectConfigDirectoryMock,
  },
}));

vi.mock("@/lib/api/adapter", () => ({
  isWeb: () => true,
}));

vi.mock("sonner", () => ({
  toast: {
    error: (...args: unknown[]) => toastErrorMock(...args),
  },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: Record<string, unknown>) =>
      (options?.defaultValue as string) ?? key,
  }),
}));

const createSettings = (
  overrides: Partial<SettingsFormState> = {},
): SettingsFormState => ({
  showInTray: true,
  minimizeToTrayOnClose: true,
  enableClaudePluginIntegration: false,
  claudeConfigDir: "/claude/custom",
  codexConfigDir: "/codex/custom",
  language: "zh",
  ...overrides,
});

describe("useDirectorySettings (web mode)", () => {
  const onUpdateSettings = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    getAppConfigDirOverrideMock.mockResolvedValue(null);
    getConfigDirMock.mockImplementation(async (app: string) =>
      app === "claude"
        ? "/remote/claude"
        : app === "codex"
          ? "/remote/codex"
          : "/remote/gemini",
    );
  });

  it("uses prompt to update app directories", async () => {
    const promptSpy = vi
      .spyOn(window, "prompt")
      .mockReturnValue(" /web/claude ");

    const { result } = renderHook(() =>
      useDirectorySettings({ settings: createSettings(), onUpdateSettings }),
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.browseDirectory("claude");
    });

    expect(promptSpy).toHaveBeenCalled();
    expect(onUpdateSettings).toHaveBeenCalledWith({
      claudeConfigDir: "/web/claude",
    });
    expect(result.current.resolvedDirs.claude).toBe("/web/claude");
  });

  it("ignores empty prompt results", async () => {
    vi.spyOn(window, "prompt").mockReturnValue("  ");

    const { result } = renderHook(() =>
      useDirectorySettings({ settings: createSettings(), onUpdateSettings }),
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.browseDirectory("codex");
    });

    expect(onUpdateSettings).not.toHaveBeenCalledWith({
      codexConfigDir: expect.any(String),
    });
  });

  it("uses prompt to update app config dir without settings update", async () => {
    vi.spyOn(window, "prompt").mockReturnValue("/web/app-config");

    const { result } = renderHook(() =>
      useDirectorySettings({ settings: createSettings(), onUpdateSettings }),
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.browseAppConfigDir();
    });

    expect(result.current.appConfigDir).toBe("/web/app-config");
    expect(onUpdateSettings).not.toHaveBeenCalledWith({
      appConfigDir: expect.any(String),
    });
  });
});
