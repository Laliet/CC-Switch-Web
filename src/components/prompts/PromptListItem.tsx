import React from "react";
import { useTranslation } from "react-i18next";
import { Edit3, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Prompt } from "@/lib/api";
import PromptToggle from "./PromptToggle";

interface PromptListItemProps {
  id: string;
  prompt: Prompt;
  onToggle: (id: string, enabled: boolean) => void;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
}

const PromptListItem: React.FC<PromptListItemProps> = ({
  id,
  prompt,
  onToggle,
  onEdit,
  onDelete,
}) => {
  const { t } = useTranslation();

  const enabled = prompt.enabled === true;
  const deleteDisabled = enabled;
  const deleteTitle = deleteDisabled
    ? t("prompts.deleteEnabledHint")
    : t("common.delete");
  const toggleLabel = enabled ? t("prompts.enabled") : t("prompts.enable");
  const editLabel = t("prompts.edit");
  const deleteLabel = deleteTitle;

  return (
    <div className="h-16 rounded-lg border border-border-default bg-card p-4 transition-[border-color,box-shadow] duration-200 hover:border-border-hover hover:shadow-sm">
      <div className="flex items-center gap-4 h-full">
        {/* Toggle 开关 */}
        <div className="flex-shrink-0">
          <PromptToggle
            enabled={enabled}
            onChange={(newEnabled) => onToggle(id, newEnabled)}
            ariaLabel={`${toggleLabel}: ${prompt.name}`}
          />
        </div>

        <div className="flex-1 min-w-0">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-1">
            {prompt.name}
          </h3>
          {prompt.description && (
            <p className="text-sm text-gray-500 dark:text-gray-400 truncate">
              {prompt.description}
            </p>
          )}
        </div>

        <div className="flex items-center gap-2 flex-shrink-0">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={() => onEdit(id)}
            title={t("common.edit")}
            aria-label={editLabel}
          >
            <Edit3 size={16} />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={() => onDelete(id)}
            className="hover:text-red-500 hover:bg-red-100 dark:hover:text-red-400 dark:hover:bg-red-500/10"
            title={deleteTitle}
            disabled={deleteDisabled}
            aria-label={deleteLabel}
          >
            <Trash2 size={16} />
          </Button>
        </div>
      </div>
    </div>
  );
};

export default PromptListItem;
