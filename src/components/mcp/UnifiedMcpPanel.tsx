import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Check, Edit3, Loader2, Plus, Server, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Switch } from "@/components/ui/switch";
import { useAllMcpServers, useToggleMcpApp } from "@/hooks/useMcp";
import type { McpServer } from "@/types";
import type { AppId } from "@/lib/api/types";
import McpFormModal from "./McpFormModal";
import { ConfirmDialog } from "../ConfirmDialog";
import { useDeleteMcpServer } from "@/hooks/useMcp";
import { settingsApi } from "@/lib/api";
import { mcpPresets } from "@/config/mcpPresets";
import { toast } from "sonner";

interface UnifiedMcpPanelProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * 统一 MCP 管理面板
 * v3.7.0 新架构：所有 MCP 服务器统一管理，每个服务器通过复选框控制应用到哪些客户端
 */
const UnifiedMcpPanel: React.FC<UnifiedMcpPanelProps> = ({
  open,
  onOpenChange,
}) => {
  const { t } = useTranslation();
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    onConfirm: () => void;
  } | null>(null);

  // Queries and Mutations
  const {
    data: serversMap,
    isLoading,
    isError,
    error: queryError,
    refetch,
  } = useAllMcpServers();
  const toggleAppMutation = useToggleMcpApp();
  const deleteServerMutation = useDeleteMcpServer();
  const [togglingIds, setTogglingIds] = useState<Set<string>>(() => new Set());
  const [deletingIds, setDeletingIds] = useState<Set<string>>(() => new Set());

  // Convert serversMap to array for easier rendering
  const serverEntries = useMemo((): Array<[string, McpServer]> => {
    if (!serversMap) return [];
    return Object.entries(serversMap);
  }, [serversMap]);

  // Count enabled servers per app
  const enabledCounts = useMemo(() => {
    const counts = { claude: 0, codex: 0, gemini: 0 };
    serverEntries.forEach(([_, server]) => {
      if (server.apps.claude) counts.claude++;
      if (server.apps.codex) counts.codex++;
      if (server.apps.gemini) counts.gemini++;
    });
    return counts;
  }, [serverEntries]);

  const handleToggleApp = async (
    serverId: string,
    app: AppId,
    enabled: boolean,
  ) => {
    if (togglingIds.has(serverId) || deletingIds.has(serverId)) {
      return;
    }

    setTogglingIds((prev) => {
      const next = new Set(prev);
      next.add(serverId);
      return next;
    });

    try {
      await toggleAppMutation.mutateAsync({ serverId, app, enabled });
    } catch (error) {
      toast.error(t("common.error"), {
        description: String(error),
      });
    } finally {
      setTogglingIds((prev) => {
        const next = new Set(prev);
        next.delete(serverId);
        return next;
      });
    }
  };

  const handleEdit = (id: string) => {
    setEditingId(id);
    setIsFormOpen(true);
  };

  const handleAdd = () => {
    setEditingId(null);
    setIsFormOpen(true);
  };

  const handleDelete = (id: string) => {
    if (deletingIds.has(id)) {
      return;
    }

    setConfirmDialog({
      isOpen: true,
      title: t("mcp.unifiedPanel.deleteServer"),
      message: t("mcp.unifiedPanel.deleteConfirm", { id }),
      onConfirm: async () => {
        try {
          setDeletingIds((prev) => {
            const next = new Set(prev);
            next.add(id);
            return next;
          });

          await deleteServerMutation.mutateAsync(id);
          setConfirmDialog(null);
          toast.success(t("common.success"));
        } catch (error) {
          toast.error(t("common.error"), {
            description: String(error),
          });
        } finally {
          setDeletingIds((prev) => {
            const next = new Set(prev);
            next.delete(id);
            return next;
          });
        }
      },
    });
  };

  const handleCloseForm = () => {
    setIsFormOpen(false);
    setEditingId(null);
  };

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-3xl max-h-[85vh] min-h-[600px] flex flex-col">
          <DialogHeader>
            <div className="flex items-center justify-between pr-8">
              <DialogTitle>{t("mcp.unifiedPanel.title")}</DialogTitle>
              <Button type="button" variant="mcp" onClick={handleAdd}>
                <Plus size={16} />
                {t("mcp.unifiedPanel.addServer")}
              </Button>
            </div>
            <DialogDescription>
              {t("mcp.unifiedPanel.description", {
                defaultValue: "集中管理 MCP 服务器并控制其应用范围。",
              })}
            </DialogDescription>
          </DialogHeader>

          {/* Info Section */}
          <div className="flex-shrink-0 px-6 py-4">
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {t("mcp.serverCount", { count: serverEntries.length })} ·{" "}
              {t("mcp.unifiedPanel.apps.claude")}: {enabledCounts.claude} ·{" "}
              {t("mcp.unifiedPanel.apps.codex")}: {enabledCounts.codex} ·{" "}
              {t("mcp.unifiedPanel.apps.gemini")}: {enabledCounts.gemini}
            </div>
          </div>

          {/* Content - Scrollable */}
          <div className="flex-1 overflow-y-auto px-6 pb-4">
            {isLoading ? (
              <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                {t("mcp.loading")}
              </div>
            ) : isError ? (
              <div className="text-center py-12">
                <div className="text-red-500 dark:text-red-400 font-medium">
                  {t("mcp.loadFailed", {
                    defaultValue: "Failed to load MCP servers",
                  })}
                </div>
                {queryError && (
                  <p className="mt-2 text-sm text-muted-foreground break-words">
                    {queryError instanceof Error
                      ? queryError.message
                      : String(queryError)}
                  </p>
                )}
                <Button
                  type="button"
                  variant="outline"
                  className="mt-4"
                  onClick={() => refetch()}
                >
                  {t("common.retry", { defaultValue: "Retry" })}
                </Button>
              </div>
            ) : serverEntries.length === 0 ? (
              <div className="text-center py-12">
                <div className="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center">
                  <Server
                    size={24}
                    className="text-gray-400 dark:text-gray-500"
                  />
                </div>
                <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                  {t("mcp.unifiedPanel.noServers")}
                </h3>
                <p className="text-gray-500 dark:text-gray-400 text-sm">
                  {t("mcp.emptyDescription")}
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {serverEntries.map(([id, server]) => (
                  <UnifiedMcpListItem
                    key={id}
                    id={id}
                    server={server}
                    isBusy={togglingIds.has(id) || deletingIds.has(id)}
                    isDeleting={deletingIds.has(id)}
                    onToggleApp={handleToggleApp}
                    onEdit={handleEdit}
                    onDelete={handleDelete}
                  />
                ))}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="mcp"
              onClick={() => onOpenChange(false)}
            >
              <Check size={16} />
              {t("common.done")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Form Modal */}
      {isFormOpen && (
        <McpFormModal
          editingId={editingId || undefined}
          initialData={
            editingId && serversMap ? serversMap[editingId] : undefined
          }
          existingIds={serversMap ? Object.keys(serversMap) : []}
          defaultFormat="json"
          onSave={async () => {
            setIsFormOpen(false);
            setEditingId(null);
          }}
          onClose={handleCloseForm}
        />
      )}

      {/* Confirm Dialog */}
      {confirmDialog && (
        <ConfirmDialog
          isOpen={confirmDialog.isOpen}
          title={confirmDialog.title}
          message={confirmDialog.message}
          onConfirm={confirmDialog.onConfirm}
          onCancel={() => setConfirmDialog(null)}
        />
      )}
    </>
  );
};

/**
 * 统一 MCP 列表项组件
 * 展示服务器名称、描述，以及三个应用的复选框
 */
interface UnifiedMcpListItemProps {
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
              checked={server.apps.claude}
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
              checked={server.apps.codex}
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
              checked={server.apps.gemini}
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

export default UnifiedMcpPanel;
