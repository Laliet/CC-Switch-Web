import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  checkAllEnvConflicts,
  checkEnvConflicts,
  deleteEnvVars,
  restoreEnvBackup,
} from "@/lib/api/env";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api/adapter", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("env API module", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("checkEnvConflicts requests conflicts for app", async () => {
    const conflicts = [{ key: "API_KEY", values: ["one", "two"] }];
    invokeMock.mockResolvedValueOnce(conflicts);

    const result = await checkEnvConflicts("claude");

    expect(result).toEqual(conflicts);
    expect(invokeMock).toHaveBeenCalledWith("check_env_conflicts", {
      app: "claude",
    });
  });

  it("deleteEnvVars sends conflict list", async () => {
    const conflicts = [{ key: "API_KEY", values: ["one"] }];
    const backup = { backupPath: "/tmp/env.bak", timestamp: "now", conflicts };
    invokeMock.mockResolvedValueOnce(backup);

    const result = await deleteEnvVars(conflicts);

    expect(result).toEqual(backup);
    expect(invokeMock).toHaveBeenCalledWith("delete_env_vars", { conflicts });
  });

  it("restoreEnvBackup sends backup path", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await restoreEnvBackup("/tmp/env.bak");

    expect(invokeMock).toHaveBeenCalledWith("restore_env_backup", {
      backupPath: "/tmp/env.bak",
    });
  });

  it("checkAllEnvConflicts continues when one app fails", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    invokeMock.mockImplementation((cmd: string, args: { app: string }) => {
      if (cmd !== "check_env_conflicts") {
        throw new Error("unexpected cmd");
      }
      if (args.app === "codex") {
        return Promise.reject(new Error("boom"));
      }
      return Promise.resolve([{ key: `${args.app}-KEY`, values: ["one"] }]);
    });

    const result = await checkAllEnvConflicts();

    expect(result.claude).toEqual([{ key: "claude-KEY", values: ["one"] }]);
    expect(result.codex).toEqual([]);
    expect(result.gemini).toEqual([{ key: "gemini-KEY", values: ["one"] }]);
    expect(consoleSpy).toHaveBeenCalled();

    consoleSpy.mockRestore();
  });
});
