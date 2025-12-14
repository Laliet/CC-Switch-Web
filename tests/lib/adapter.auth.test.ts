import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const importAdapter = async () => {
  vi.resetModules();
  return import("@/lib/api/adapter");
};

const mockFetchJson = (payload: unknown) =>
  vi.spyOn(globalThis, "fetch").mockResolvedValue({
    ok: true,
    status: 200,
    headers: new Headers({ "content-type": "application/json" }),
    json: async () => payload,
    text: async () => JSON.stringify(payload),
  } as any);

let originalTauri: unknown;
let originalTauriInternals: unknown;

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(console, "info").mockImplementation(() => {});
  delete (window as any).__CC_SWITCH_TOKENS__;
  originalTauri = (window as any).__TAURI__;
  originalTauriInternals = (window as any).__TAURI_INTERNALS__;
  delete (window as any).__TAURI__;
  delete (window as any).__TAURI_INTERNALS__;
});

afterEach(() => {
  (window as any).__TAURI__ = originalTauri;
  (window as any).__TAURI_INTERNALS__ = originalTauriInternals;
});

describe("adapter auth (web mode)", () => {
  it("getAutoTokens returns csrf token when available", async () => {
    (window as any).__CC_SWITCH_TOKENS__ = { csrfToken: "csrf-123" };
    const fetchMock = mockFetchJson("/tmp/config.json");
    const { invoke } = await importAdapter();

    await invoke("get_app_config_path");

    const [, init] = fetchMock.mock.calls[0]!;
    expect(init).toBeDefined();
    const headers = (init as RequestInit).headers as Record<string, string>;
    expect(headers["X-CSRF-Token"]).toBe("csrf-123");
  });

  it("getAutoTokens returns undefined when tokens not set", async () => {
    const fetchMock = mockFetchJson("/tmp/config.json");
    const { invoke } = await importAdapter();

    await invoke("get_app_config_path");

    const [, init] = fetchMock.mock.calls[0]!;
    expect(init).toBeDefined();
    const headers = (init as RequestInit).headers as Record<string, string>;
    expect(headers["X-CSRF-Token"]).toBeUndefined();
  });

  it("invoke includes X-CSRF-Token header when tokens available", async () => {
    (window as any).__CC_SWITCH_TOKENS__ = { csrfToken: "csrf-456" };
    const fetchMock = mockFetchJson(true);
    const { invoke } = await importAdapter();

    await invoke("add_provider", {
      app: "claude",
      provider: { name: "Test Provider", settingsConfig: {} },
    });

    const [, init] = fetchMock.mock.calls[0]!;
    expect(init).toBeDefined();
    const headers = (init as RequestInit).headers as Record<string, string>;
    expect(headers["X-CSRF-Token"]).toBe("csrf-456");
    expect(headers["Content-Type"]).toBe("application/json");
  });

  it("invoke uses credentials: include for Basic Auth", async () => {
    const fetchMock = mockFetchJson("/tmp/config.json");
    const { invoke } = await importAdapter();

    await invoke("get_app_config_path");

    const [, init] = fetchMock.mock.calls[0]!;
    expect(init).toBeDefined();
    expect((init as RequestInit).credentials).toBe("include");
  });

  it("invoke does not include Authorization Bearer header (removed)", async () => {
    const fetchMock = mockFetchJson("/tmp/config.json");
    const { invoke } = await importAdapter();

    await invoke("get_app_config_path");

    const [, init] = fetchMock.mock.calls[0]!;
    expect(init).toBeDefined();
    const headers = (init as RequestInit).headers as Record<string, string>;
    expect(headers.Authorization).toBeUndefined();
    expect(
      Object.values(headers).some((value) => value.startsWith("Bearer ")),
    ).toBe(false);
  });
});
