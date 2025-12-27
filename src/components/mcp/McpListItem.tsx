import React from "react";
import { useTranslation } from "react-i18next";
import { Edit3, Trash2, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import type { McpServer } from "@/types";
import type { AppId } from "@/lib/api/types";
import { settingsApi } from "@/lib/api";
import { mcpPresets } from "@/config/mcpPresets";

/**
 * 统一 MCP 列表项组件
 * 展示服务器名称、描述，以及三个应用的复选框
 */
export interface UnifiedMcpListItemProps {
  id: string;
  server: McpServer;
  isBusy: boolean;
  isDeleting: boolean;
  onToggleApp: (serverId: string, app: AppId, enabled: boolean) => void;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
}

const UnifiedMcpListItem: React.FC<UnifiedMcpListItemProps> = ({
  id,
  server,
  isBusy,
  isDeleting,
  onToggleApp,
  onEdit,
  onDelete,
}) => {
  const { t } = useTranslation();
  const name = server.name || id;
  const description = server.description || "";

  // 匹配预设元信息
  const meta = mcpPresets.find((p) => p.id === id);
  const docsUrl = server.docs || meta?.docs;
  const homepageUrl = server.homepage || meta?.homepage;
  const tags = server.tags || meta?.tags;

  const openDocs = async () => {
    const url = docsUrl || homepageUrl;
    if (!url) return;
    try {
      await settingsApi.openExternal(url);
    } catch {
      // ignore
    }
  };

  return (
    <div className="min-h-16 rounded-lg border border-border-default bg-card p-4 transition-[border-color,box-shadow] duration-200 hover:border-border-hover hover:shadow-sm">
      <div className="flex items-center gap-4">
        {/* 左侧：服务器信息 */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-medium text-gray-900 dark:text-gray-100">
              {name}
            </h3>
            {docsUrl && (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={openDocs}
                title={t("mcp.presets.docs")}
              >
                {t("mcp.presets.docs")}
              </Button>
            )}
          </div>
          {description && (
            <p className="text-sm text-gray-500 dark:text-gray-400 line-clamp-2">
              {description}
            </p>
          )}
          {!description && tags && tags.length > 0 && (
            <p className="text-xs text-gray-400 dark:text-gray-500 truncate">
              {tags.join(", ")}
            </p>
          )}
        </div>

        {/* 中间：应用开关 */}
        <div className="flex flex-col gap-2 flex-shrink-0 min-w-[120px]">
          <div className="flex items-center justify-between gap-3">
            <label
              htmlFor={`${id}-claude`}
              className="text-sm text-gray-700 dark:text-gray-300 cursor-pointer"
            >
              {t("mcp.unifiedPanel.apps.claude")}
            </label>
            <Switch
              id={`${id}-claude`}
              checked={server.apps?.claude ?? false}
              onCheckedChange={(checked: boolean) =>
                !isBusy && onToggleApp(id, "claude", checked)
              }
              disabled={isBusy}
            />
          </div>

          <div className="flex items-center justify-between gap-3">
            <label
              htmlFor={`${id}-codex`}
              className="text-sm text-gray-700 dark:text-gray-300 cursor-pointer"
            >
              {t("mcp.unifiedPanel.apps.codex")}
            </label>
            <Switch
              id={`${id}-codex`}
              checked={server.apps?.codex ?? false}
              onCheckedChange={(checked: boolean) =>
                !isBusy && onToggleApp(id, "codex", checked)
              }
              disabled={isBusy}
            />
          </div>

          <div className="flex items-center justify-between gap-3">
            <label
              htmlFor={`${id}-gemini`}
              className="text-sm text-gray-700 dark:text-gray-300 cursor-pointer"
            >
              {t("mcp.unifiedPanel.apps.gemini")}
            </label>
            <Switch
              id={`${id}-gemini`}
              checked={server.apps?.gemini ?? false}
              onCheckedChange={(checked: boolean) =>
                !isBusy && onToggleApp(id, "gemini", checked)
              }
              disabled={isBusy}
            />
          </div>
        </div>

        {/* 右侧：操作按钮 */}
        <div className="flex items-center gap-2 flex-shrink-0">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={() => onEdit(id)}
            title={t("common.edit")}
          >
            <Edit3 size={16} />
          </Button>

          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={() => onDelete(id)}
            className="hover:text-red-500 hover:bg-red-100 dark:hover:text-red-400 dark:hover:bg-red-500/10"
            title={t("common.delete")}
            disabled={isBusy}
          >
            {isDeleting ? (
              <Loader2 size={16} className="animate-spin" />
            ) : (
              <Trash2 size={16} />
            )}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default UnifiedMcpListItem;
