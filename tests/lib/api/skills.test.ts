import { beforeEach, describe, expect, it, vi } from "vitest";
import { skillsApi } from "@/lib/api/skills";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api/adapter", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

const skill = {
  key: "skill-1",
  name: "Sample Skill",
  description: "Skill description",
  directory: "skills/sample",
  installed: false,
};

describe("skills API module", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("getAll normalizes array response", async () => {
    invokeMock.mockResolvedValueOnce([skill]);

    const result = await skillsApi.getAll();

    expect(result).toEqual({
      skills: [skill],
      warnings: [],
      cacheHit: false,
      refreshing: false,
    });
    expect(invokeMock).toHaveBeenCalledWith("get_skills");
  });

  it("getAll handles non-object response", async () => {
    invokeMock.mockResolvedValueOnce(null);

    const result = await skillsApi.getAll();

    expect(result).toEqual({
      skills: [],
      warnings: [],
      cacheHit: false,
      refreshing: false,
    });
  });

  it("getAll returns cache metadata", async () => {
    invokeMock.mockResolvedValueOnce({
      skills: [skill],
      warnings: ["warn"],
      cacheHit: true,
      refreshing: true,
    });

    const result = await skillsApi.getAll();

    expect(result).toEqual({
      skills: [skill],
      warnings: ["warn"],
      cacheHit: true,
      refreshing: true,
    });
  });

  it("getAll reads snake_case cache metadata", async () => {
    invokeMock.mockResolvedValueOnce({
      skills: [skill],
      warnings: ["warn"],
      cache_hit: true,
      refreshing: false,
    });

    const result = await skillsApi.getAll();

    expect(result).toEqual({
      skills: [skill],
      warnings: ["warn"],
      cacheHit: true,
      refreshing: false,
    });
  });

  it("getAll defaults missing metadata", async () => {
    invokeMock.mockResolvedValueOnce({
      skills: [skill],
      warnings: [],
    });

    const result = await skillsApi.getAll();

    expect(result).toEqual({
      skills: [skill],
      warnings: [],
      cacheHit: false,
      refreshing: false,
    });
  });
});
