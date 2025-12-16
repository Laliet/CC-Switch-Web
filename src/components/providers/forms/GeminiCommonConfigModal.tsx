import React from "react";
import { Save, Wand2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { formatJSON } from "@/utils/formatters";

interface GeminiCommonConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  value: string;
  onChange: (value: string) => void;
  error?: string;
}

/**
 * GeminiCommonConfigModal - Common Gemini configuration editor modal
 * Allows editing of common JSON configuration shared across Gemini providers
 */
export const GeminiCommonConfigModal: React.FC<
  GeminiCommonConfigModalProps
> = ({ isOpen, onClose, value, onChange, error }) => {
  const { t } = useTranslation();

  const handleFormat = () => {
    if (!value.trim()) return;

    try {
      const formatted = formatJSON(value);
      onChange(formatted);
      toast.success(t("common.formatSuccess", { defaultValue: "格式化成功" }));
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      toast.error(
        t("common.formatError", {
          defaultValue: "格式化失败：{{error}}",
          error: errorMessage,
        }),
      );
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        zIndex="nested"
        className="max-w-2xl max-h-[90vh] flex flex-col p-0"
      >
        <DialogHeader className="px-6 pt-6 pb-0">
          <DialogTitle>
            {t("geminiConfig.editCommonConfigTitle", {
              defaultValue: "编辑 Gemini 通用配置片段",
            })}
          </DialogTitle>
          <DialogDescription>
            {t("geminiConfig.commonConfigDescription", {
              defaultValue:
                "编辑共享配置片段，将其应用到启用的 Gemini 供应商。",
            })}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-auto px-6 py-4 space-y-4">
          <p className="text-sm text-gray-500 dark:text-gray-400">
            {t("geminiConfig.commonConfigHint", {
              defaultValue:
                "通用配置片段将合并到所有启用它的 Gemini 供应商配置中",
            })}
          </p>

          <textarea
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder={`{
  "timeout": 30000,
  "maxRetries": 3,
  "customField": "value"
}`}
            rows={12}
            className="w-full px-3 py-2 border border-border-default dark:bg-gray-800 dark:text-gray-100 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500/20 dark:focus:ring-blue-400/20 focus:border-border-active transition-colors resize-y"
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="none"
            spellCheck={false}
            lang="en"
            inputMode="text"
            data-gramm="false"
            data-gramm_editor="false"
            data-enable-grammarly="false"
          />

          <div className="flex items-center justify-between">
            <button
              type="button"
              onClick={handleFormat}
              className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
            >
              <Wand2 className="w-3.5 h-3.5" />
              {t("common.format", { defaultValue: "格式化" })}
            </button>

            {error && (
              <p className="text-sm text-red-500 dark:text-red-400">{error}</p>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button type="button" onClick={onClose} className="gap-2">
            <Save className="w-4 h-4" />
            {t("common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
