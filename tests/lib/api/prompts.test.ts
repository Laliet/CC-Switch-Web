import { beforeEach, describe, expect, it, vi } from "vitest";
import { promptsApi } from "@/lib/api/prompts";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api/adapter", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("prompts API module", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("getPrompts returns prompt map", async () => {
    const prompts = {
      "1": {
        id: "1",
        name: "Prompt",
        content: "Hello",
        enabled: true,
      },
    };
    invokeMock.mockResolvedValueOnce(prompts);

    const result = await promptsApi.getPrompts("claude");

    expect(result).toEqual(prompts);
    expect(invokeMock).toHaveBeenCalledWith("get_prompts", { app: "claude" });
  });

  it("upsertPrompt sends payload", async () => {
    const prompt = {
      id: "2",
      name: "Test",
      content: "Body",
      enabled: false,
    };
    invokeMock.mockResolvedValueOnce(undefined);

    await promptsApi.upsertPrompt("codex", "2", prompt);

    expect(invokeMock).toHaveBeenCalledWith("upsert_prompt", {
      app: "codex",
      id: "2",
      prompt,
    });
  });

  it("deletePrompt sends id", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await promptsApi.deletePrompt("gemini", "3");

    expect(invokeMock).toHaveBeenCalledWith("delete_prompt", {
      app: "gemini",
      id: "3",
    });
  });

  it("enablePrompt sends id", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await promptsApi.enablePrompt("claude", "4");

    expect(invokeMock).toHaveBeenCalledWith("enable_prompt", {
      app: "claude",
      id: "4",
    });
  });

  it("importFromFile sends app", async () => {
    invokeMock.mockResolvedValueOnce("/tmp/prompt.json");

    const result = await promptsApi.importFromFile("codex");

    expect(result).toBe("/tmp/prompt.json");
    expect(invokeMock).toHaveBeenCalledWith("import_prompt_from_file", {
      app: "codex",
    });
  });

  it("getCurrentFileContent returns current content", async () => {
    invokeMock.mockResolvedValueOnce("prompt content");

    const result = await promptsApi.getCurrentFileContent("claude");

    expect(result).toBe("prompt content");
    expect(invokeMock).toHaveBeenCalledWith("get_current_prompt_file_content", {
      app: "claude",
    });
  });
});
