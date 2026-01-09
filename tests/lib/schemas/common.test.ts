import { describe, expect, it } from "vitest";
import { jsonConfigSchema, tomlConfigSchema } from "@/lib/schemas/common";

describe("jsonConfigSchema", () => {
  it("rejects empty string", () => {
    expect(jsonConfigSchema.safeParse("").success).toBe(false);
  });

  it("rejects invalid JSON syntax", () => {
    expect(jsonConfigSchema.safeParse("{").success).toBe(false);
  });

  it("rejects array values", () => {
    expect(jsonConfigSchema.safeParse("[]").success).toBe(false);
  });

  it("rejects null values", () => {
    expect(jsonConfigSchema.safeParse("null").success).toBe(false);
  });

  it("accepts valid objects", () => {
    expect(jsonConfigSchema.safeParse('{"enabled":true}').success).toBe(true);
  });
});

describe("tomlConfigSchema", () => {
  it("accepts empty string", () => {
    expect(tomlConfigSchema.safeParse("").success).toBe(true);
  });

  it("rejects invalid TOML syntax", () => {
    expect(tomlConfigSchema.safeParse("foo =").success).toBe(false);
  });

  it("rejects stdio config missing command", () => {
    expect(tomlConfigSchema.safeParse('type = "stdio"').success).toBe(false);
  });

  it("rejects http config missing url", () => {
    expect(tomlConfigSchema.safeParse('type = "http"').success).toBe(false);
  });

  it("rejects sse config missing url", () => {
    expect(tomlConfigSchema.safeParse('type = "sse"').success).toBe(false);
  });

  it("accepts valid config", () => {
    const validToml = ['type = "stdio"', 'command = "node"'].join("\n");
    expect(tomlConfigSchema.safeParse(validToml).success).toBe(true);
  });
});
