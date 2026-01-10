import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  checkAllEnvConflicts,
  checkEnvConflicts,
  deleteEnvVars,
  restoreEnvBackup,
} from "@/lib/api/env";
import type { EnvConflict } from "@/types/env";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api/adapter", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("env API module", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("checkEnvConflicts requests conflicts for app", async () => {
    const conflicts: EnvConflict[] = [
      {
        varName: "API_KEY",
        varValue: "one",
        sourceType: "system",
        sourcePath: "/etc/profile",
      },
      {
        varName: "API_KEY",
        varValue: "two",
        sourceType: "file",
        sourcePath: "/path/.env:1",
      },
    ];
    invokeMock.mockResolvedValueOnce(conflicts);

    const result = await checkEnvConflicts("claude");

    expect(result).toEqual(conflicts);
    expect(invokeMock).toHaveBeenCalledWith("check_env_conflicts", {
      app: "claude",
    });
  });

  it("deleteEnvVars sends conflict list", async () => {
    const conflicts: EnvConflict[] = [
      {
        varName: "API_KEY",
        varValue: "one",
        sourceType: "system",
        sourcePath: "/etc/profile",
      },
    ];
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
      return Promise.resolve<EnvConflict[]>([
        {
          varName: `${args.app}-KEY`,
          varValue: "one",
          sourceType: "system",
          sourcePath: "/etc/profile",
        },
      ]);
    });

    const result = await checkAllEnvConflicts();

    expect(result.claude).toEqual([
      {
        varName: "claude-KEY",
        varValue: "one",
        sourceType: "system",
        sourcePath: "/etc/profile",
      },
    ]);
    expect(result.codex).toEqual([]);
    expect(result.gemini).toEqual([
      {
        varName: "gemini-KEY",
        varValue: "one",
        sourceType: "system",
        sourcePath: "/etc/profile",
      },
    ]);
    expect(consoleSpy).toHaveBeenCalled();

    consoleSpy.mockRestore();
  });
});
