import { invoke as tauriInvoke } from "@tauri-apps/api/core";

type HttpMethod = "GET" | "POST" | "PUT" | "DELETE";
type CommandArgs = Record<string, any>;

interface Endpoint {
  url: string;
  method: HttpMethod;
  body?: unknown;
}

const API_BASE = "/api";

const encode = (value: unknown) => encodeURIComponent(String(value));

const requireArg = (args: CommandArgs, key: string, cmd: string) => {
  const value = args?.[key];
  if (value === undefined || value === null) {
    throw new Error(`Missing argument "${key}" for command "${cmd}" in web mode`);
  }
  return value;
};

export function isWeb(): boolean {
  if (import.meta.env?.VITE_MODE === "web") {
    return true;
  }
  return typeof window === "undefined" || !(window as any).__TAURI__;
}

declare global {
  interface Window {
    __CC_SWITCH_TOKENS__?: {
      apiToken: string;
      csrfToken: string;
      __noticeShown?: boolean;
    };
  }
}

function getAutoTokens() {
  if (typeof window === "undefined") return undefined;
  const tokens = window.__CC_SWITCH_TOKENS__;
  if (tokens?.apiToken && tokens?.csrfToken) {
    if (!tokens.__noticeShown) {
      console.info("cc-switch: 已自动应用内置 API Token 与 CSRF Token");
      tokens.__noticeShown = true;
    }
    return { apiToken: tokens.apiToken, csrfToken: tokens.csrfToken };
  }
  return undefined;
}

export function commandToEndpoint(
  cmd: string,
  args: CommandArgs = {},
): Endpoint {
  switch (cmd) {
    // Provider commands
    case "get_providers": {
      const app = requireArg(args, "app", cmd);
      return { method: "GET", url: `${API_BASE}/providers/${encode(app)}` };
    }
    case "get_current_provider": {
      const app = requireArg(args, "app", cmd);
      return {
        method: "GET",
        url: `${API_BASE}/providers/${encode(app)}/current`,
      };
    }
    case "get_backup_provider": {
      const app = requireArg(args, "app", cmd);
      return {
        method: "GET",
        url: `${API_BASE}/providers/${encode(app)}/backup`,
      };
    }
    case "set_backup_provider": {
      const app = requireArg(args, "app", cmd);
      return {
        method: "PUT",
        url: `${API_BASE}/providers/${encode(app)}/backup`,
        body: { id: args.id ?? null },
      };
    }
    case "add_provider": {
      const app = requireArg(args, "app", cmd);
      const provider = requireArg(args, "provider", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/providers/${encode(app)}`,
        body: provider,
      };
    }
    case "update_provider": {
      const app = requireArg(args, "app", cmd);
      const provider = requireArg(args, "provider", cmd);
      const providerId =
        (provider && (provider.id ?? provider.providerId)) ?? args.id;
      if (!providerId) {
        throw new Error(`Missing provider id for command "${cmd}" in web mode`);
      }
      return {
        method: "PUT",
        url: `${API_BASE}/providers/${encode(app)}/${encode(providerId)}`,
        body: provider,
      };
    }
    case "delete_provider": {
      const app = requireArg(args, "app", cmd);
      const id = requireArg(args, "id", cmd);
      return {
        method: "DELETE",
        url: `${API_BASE}/providers/${encode(app)}/${encode(id)}`,
      };
    }
    case "switch_provider": {
      const app = requireArg(args, "app", cmd);
      const id = requireArg(args, "id", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/providers/${encode(app)}/${encode(id)}/switch`,
      };
    }
    case "import_default_config": {
      const app = requireArg(args, "app", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/providers/${encode(app)}/import-default`,
      };
    }
    case "update_tray_menu": {
      return { method: "POST", url: `${API_BASE}/tray/update` };
    }
    case "update_providers_sort_order": {
      const app = requireArg(args, "app", cmd);
      return {
        method: "PUT",
        url: `${API_BASE}/providers/${encode(app)}/sort-order`,
        body: { updates: args.updates },
      };
    }
    case "queryProviderUsage": {
      const app = requireArg(args, "app", cmd);
      const providerId = requireArg(args, "providerId", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/providers/${encode(app)}/${encode(providerId)}/usage`,
      };
    }
    case "testUsageScript": {
      const app = requireArg(args, "app", cmd);
      const providerId = requireArg(args, "providerId", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/providers/${encode(app)}/${encode(providerId)}/usage/test`,
        body: {
          scriptCode: requireArg(args, "scriptCode", cmd),
          timeout: args.timeout,
          apiKey: args.apiKey,
          baseUrl: args.baseUrl,
          accessToken: args.accessToken,
          userId: args.userId,
        },
      };
    }

    // MCP commands
    case "get_claude_mcp_status":
      return { method: "GET", url: `${API_BASE}/mcp/status` };
    case "read_claude_mcp_config":
      return { method: "GET", url: `${API_BASE}/mcp/config/claude` };
    case "upsert_claude_mcp_server": {
      const id = requireArg(args, "id", cmd);
      const spec = requireArg(args, "spec", cmd);
      return {
        method: "PUT",
        url: `${API_BASE}/mcp/config/claude/servers/${encode(id)}`,
        body: { spec },
      };
    }
    case "delete_claude_mcp_server": {
      const id = requireArg(args, "id", cmd);
      return {
        method: "DELETE",
        url: `${API_BASE}/mcp/config/claude/servers/${encode(id)}`,
      };
    }
    case "validate_mcp_command":
      return {
        method: "POST",
        url: `${API_BASE}/mcp/validate`,
        body: { cmd: requireArg(args, "cmd", cmd) },
      };
    case "get_mcp_config": {
      const app = requireArg(args, "app", cmd);
      return { method: "GET", url: `${API_BASE}/mcp/config/${encode(app)}` };
    }
    case "upsert_mcp_server_in_config": {
      const app = requireArg(args, "app", cmd);
      const id = requireArg(args, "id", cmd);
      const spec = requireArg(args, "spec", cmd);
      return {
        method: "PUT",
        url: `${API_BASE}/mcp/config/${encode(app)}/servers/${encode(id)}`,
        body: {
          spec,
          ...(args.syncOtherSide !== undefined
            ? { syncOtherSide: args.syncOtherSide }
            : {}),
        },
      };
    }
    case "delete_mcp_server_in_config": {
      const app = requireArg(args, "app", cmd);
      const id = requireArg(args, "id", cmd);
      return {
        method: "DELETE",
        url: `${API_BASE}/mcp/config/${encode(app)}/servers/${encode(id)}`,
        body:
          args.syncOtherSide !== undefined
            ? { syncOtherSide: args.syncOtherSide }
            : undefined,
      };
    }
    case "set_mcp_enabled": {
      const app = requireArg(args, "app", cmd);
      const id = requireArg(args, "id", cmd);
      const enabled = requireArg(args, "enabled", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/mcp/config/${encode(app)}/servers/${encode(id)}/enabled`,
        body: { enabled },
      };
    }
    case "get_mcp_servers":
      return { method: "GET", url: `${API_BASE}/mcp/servers` };
    case "upsert_mcp_server": {
      const server = requireArg(args, "server", cmd);
      const id = requireArg(server, "id", cmd);
      return {
        method: "PUT",
        url: `${API_BASE}/mcp/servers/${encode(id)}`,
        body: server,
      };
    }
    case "delete_mcp_server": {
      const id = requireArg(args, "id", cmd);
      return {
        method: "DELETE",
        url: `${API_BASE}/mcp/servers/${encode(id)}`,
      };
    }
    case "toggle_mcp_app": {
      const serverId = requireArg(args, "serverId", cmd);
      const app = requireArg(args, "app", cmd);
      const enabled = requireArg(args, "enabled", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/mcp/servers/${encode(serverId)}/apps/${encode(app)}`,
        body: { enabled },
      };
    }

    // Prompt commands
    case "get_prompts": {
      const app = requireArg(args, "app", cmd);
      return { method: "GET", url: `${API_BASE}/prompts/${encode(app)}` };
    }
    case "upsert_prompt": {
      const app = requireArg(args, "app", cmd);
      const id = requireArg(args, "id", cmd);
      const prompt = requireArg(args, "prompt", cmd);
      return {
        method: "PUT",
        url: `${API_BASE}/prompts/${encode(app)}/${encode(id)}`,
        body: prompt,
      };
    }
    case "delete_prompt": {
      const app = requireArg(args, "app", cmd);
      const id = requireArg(args, "id", cmd);
      return {
        method: "DELETE",
        url: `${API_BASE}/prompts/${encode(app)}/${encode(id)}`,
      };
    }
    case "enable_prompt": {
      const app = requireArg(args, "app", cmd);
      const id = requireArg(args, "id", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/prompts/${encode(app)}/${encode(id)}/enable`,
      };
    }
    case "import_prompt_from_file": {
      const app = requireArg(args, "app", cmd);
      return {
        method: "POST",
        url: `${API_BASE}/prompts/${encode(app)}/import-from-file`,
      };
    }
    case "get_current_prompt_file_content": {
      const app = requireArg(args, "app", cmd);
      return {
        method: "GET",
        url: `${API_BASE}/prompts/${encode(app)}/current-file`,
      };
    }

    // Skill commands
    case "get_skills":
      return { method: "GET", url: `${API_BASE}/skills` };
    case "install_skill":
      return {
        method: "POST",
        url: `${API_BASE}/skills/install`,
        body: { directory: requireArg(args, "directory", cmd) },
      };
    case "uninstall_skill":
      return {
        method: "POST",
        url: `${API_BASE}/skills/uninstall`,
        body: { directory: requireArg(args, "directory", cmd) },
      };
    case "get_skill_repos":
      return { method: "GET", url: `${API_BASE}/skills/repos` };
    case "add_skill_repo":
      return {
        method: "POST",
        url: `${API_BASE}/skills/repos`,
        body: requireArg(args, "repo", cmd),
      };
    case "remove_skill_repo": {
      const owner = requireArg(args, "owner", cmd);
      const name = requireArg(args, "name", cmd);
      return {
        method: "DELETE",
        url: `${API_BASE}/skills/repos/${encode(owner)}/${encode(name)}`,
      };
    }

    // Settings / system commands
    case "get_settings":
      return { method: "GET", url: `${API_BASE}/settings` };
    case "save_settings":
      return {
        method: "PUT",
        url: `${API_BASE}/settings`,
        body: requireArg(args, "settings", cmd),
      };
    case "restart_app":
      return { method: "POST", url: `${API_BASE}/system/restart` };
    case "check_for_updates":
      return { method: "POST", url: `${API_BASE}/system/check-updates` };
    case "is_portable_mode":
      return { method: "GET", url: `${API_BASE}/system/is-portable` };
    case "get_config_dir": {
      const app = requireArg(args, "app", cmd);
      return { method: "GET", url: `${API_BASE}/config/${encode(app)}/dir` };
    }
    case "open_config_folder": {
      const app = requireArg(args, "app", cmd);
      return { method: "POST", url: `${API_BASE}/config/${encode(app)}/open` };
    }
    case "pick_directory":
      return {
        method: "POST",
        url: `${API_BASE}/fs/pick-directory`,
        body:
          args.defaultPath !== undefined
            ? { defaultPath: args.defaultPath }
            : undefined,
      };
    case "get_claude_code_config_path":
      return { method: "GET", url: `${API_BASE}/config/claude-code/path` };
    case "get_app_config_path":
      return { method: "GET", url: `${API_BASE}/config/app/path` };
    case "open_app_config_folder":
      return { method: "POST", url: `${API_BASE}/config/app/open` };
    case "get_app_config_dir_override":
      return { method: "GET", url: `${API_BASE}/config/app/override` };
    case "set_app_config_dir_override":
      return {
        method: "PUT",
        url: `${API_BASE}/config/app/override`,
        body: { path: args.path },
      };
    case "apply_claude_plugin_config":
      return {
        method: "POST",
        url: `${API_BASE}/config/claude/plugin`,
        body: { official: requireArg(args, "official", cmd) },
      };
    case "save_file_dialog":
      return {
        method: "POST",
        url: `${API_BASE}/fs/save-file`,
        body: { defaultName: requireArg(args, "defaultName", cmd) },
      };
    case "open_file_dialog":
      return { method: "POST", url: `${API_BASE}/fs/open-file` };
    case "export_config_to_file":
      return {
        method: "POST",
        url: `${API_BASE}/config/export`,
        body: { filePath: requireArg(args, "filePath", cmd) },
      };
    case "import_config_from_file":
      return {
        method: "POST",
        url: `${API_BASE}/config/import`,
        body: { filePath: requireArg(args, "filePath", cmd) },
      };
    case "sync_current_providers_live":
      return { method: "POST", url: `${API_BASE}/providers/sync-current` };
    case "open_external":
      return {
        method: "POST",
        url: `${API_BASE}/system/open-external`,
        body: { url: requireArg(args, "url", cmd) },
      };

    // Config snippet commands
    case "get_claude_common_config_snippet":
      return {
        method: "GET",
        url: `${API_BASE}/config/claude/common-snippet`,
      };
    case "set_claude_common_config_snippet":
      return {
        method: "PUT",
        url: `${API_BASE}/config/claude/common-snippet`,
        body: { snippet: requireArg(args, "snippet", cmd) },
      };
    case "get_common_config_snippet": {
      const appType = requireArg(args, "appType", cmd);
      return {
        method: "GET",
        url: `${API_BASE}/config/${encode(appType)}/common-snippet`,
      };
    }
    case "set_common_config_snippet": {
      const appType = requireArg(args, "appType", cmd);
      return {
        method: "PUT",
        url: `${API_BASE}/config/${encode(appType)}/common-snippet`,
        body: { snippet: requireArg(args, "snippet", cmd) },
      };
    }

    default:
      return {
        method: "POST",
        url: `${API_BASE}/tauri/${cmd}`,
        body: args,
      };
  }
}

export async function invoke<T>(
  cmd: string,
  args: CommandArgs = {},
): Promise<T> {
  if (!isWeb()) {
    return tauriInvoke<T>(cmd, args);
  }

  switch (cmd) {
    case "update_tray_menu":
      return true as T;
    case "check_for_updates":
      return null as T;
    case "restart_app":
      return undefined as T;
    case "is_portable_mode":
      return false as T;
    case "check_env_conflicts":
      return [] as T;
    case "delete_env_vars":
      return {
        backupPath: "",
        timestamp: "",
        conflicts: [],
      } as T;
    case "restore_env_backup":
      return undefined as T;
    case "get_env_var":
    case "set_env_var":
      return null as T;
    case "open_external": {
      const url = args.url as string | undefined;
      if (typeof window !== "undefined" && url) {
        window.open(url, "_blank", "noopener,noreferrer");
      }
      return true as T;
    }
    default:
      break;
  }

  const endpoint = commandToEndpoint(cmd, args);
  const headers: Record<string, string> = { Accept: "application/json" };
  const tokens = getAutoTokens();
  if (tokens) {
    headers["Authorization"] = `Bearer ${tokens.apiToken}`;
    headers["X-CSRF-Token"] = tokens.csrfToken;
  }
  const init: RequestInit = {
    method: endpoint.method,
    credentials: "include",
    headers,
  };

  if (endpoint.method !== "GET" && endpoint.body !== undefined) {
    headers["Content-Type"] = "application/json";
    init.body = JSON.stringify(endpoint.body);
  }

  const response = await fetch(endpoint.url, init);
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      errorText || `Request failed with status ${response.status}`,
    );
  }

  if (response.status === 204) {
    return undefined as T;
  }

  const contentType = response.headers.get("content-type") || "";
  if (contentType.includes("application/json")) {
    return (await response.json()) as T;
  }

  const text = await response.text();
  return text as unknown as T;
}
