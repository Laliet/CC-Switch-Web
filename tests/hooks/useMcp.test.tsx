import type { ReactNode } from "react";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  useAllMcpServers,
  useUpsertMcpServer,
  useToggleMcpApp,
  useDeleteMcpServer,
} from "@/hooks/useMcp";
import type { McpServer, McpServersMap } from "@/types";
import type { AppId } from "@/lib/api/types";

const getAllServersMock = vi.hoisted(() => vi.fn());
const upsertUnifiedServerMock = vi.hoisted(() => vi.fn());
const toggleAppMock = vi.hoisted(() => vi.fn());
const deleteUnifiedServerMock = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api/mcp", () => ({
  mcpApi: {
    getAllServers: (...args: unknown[]) => getAllServersMock(...args),
    upsertUnifiedServer: (...args: unknown[]) => upsertUnifiedServerMock(...args),
    toggleApp: (...args: unknown[]) => toggleAppMock(...args),
    deleteUnifiedServer: (...args: unknown[]) => deleteUnifiedServerMock(...args),
  },
}));

interface WrapperProps {
  children: ReactNode;
}

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  const wrapper = ({ children }: WrapperProps) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );

  return { wrapper, queryClient };
}

function createServer(overrides: Partial<McpServer> = {}): McpServer {
  return {
    id: "server-1",
    name: "Test Server",
    server: {
      type: "stdio",
      command: "node",
      args: ["index.js"],
    },
    apps: {
      claude: true,
      codex: false,
      gemini: false,
    },
    ...overrides,
  };
}

describe("useMcp hooks", () => {
  beforeEach(() => {
    getAllServersMock.mockReset();
    upsertUnifiedServerMock.mockReset();
    toggleAppMock.mockReset();
    deleteUnifiedServerMock.mockReset();
  });

  it("fetches all MCP servers", async () => {
    const expected: McpServersMap = {
      "server-1": createServer(),
    };
    getAllServersMock.mockResolvedValueOnce(expected);
    const { wrapper } = createWrapper();

    const { result } = renderHook(() => useAllMcpServers(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(getAllServersMock).toHaveBeenCalledTimes(1);
    expect(result.current.data).toEqual(expected);
  });

  it("upserts a server and invalidates the MCP query", async () => {
    upsertUnifiedServerMock.mockResolvedValueOnce(undefined);
    const { wrapper, queryClient } = createWrapper();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const server = createServer();

    const { result } = renderHook(() => useUpsertMcpServer(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(server);
    });

    expect(upsertUnifiedServerMock).toHaveBeenCalledWith(server);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["mcp", "all"] });
  });

  it("toggles MCP app state and invalidates the MCP query", async () => {
    toggleAppMock.mockResolvedValueOnce(undefined);
    const { wrapper, queryClient } = createWrapper();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const payload: { serverId: string; app: AppId; enabled: boolean } = {
      serverId: "server-2",
      app: "codex",
      enabled: true,
    };

    const { result } = renderHook(() => useToggleMcpApp(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(payload);
    });

    expect(toggleAppMock).toHaveBeenCalledWith(payload.serverId, payload.app, payload.enabled);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["mcp", "all"] });
  });

  it("deletes a server and invalidates the MCP query", async () => {
    deleteUnifiedServerMock.mockResolvedValueOnce(true);
    const { wrapper, queryClient } = createWrapper();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const { result } = renderHook(() => useDeleteMcpServer(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync("server-3");
    });

    expect(deleteUnifiedServerMock).toHaveBeenCalledWith("server-3");
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["mcp", "all"] });
  });
});
