import { useCallback, useEffect, useMemo, useState } from "react";
import { Loader2, Save } from "lucide-react";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogDescription,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import { settingsApi } from "@/lib/api";
import { LanguageSettings } from "@/components/settings/LanguageSettings";
import { ThemeSettings } from "@/components/settings/ThemeSettings";
import { WindowSettings } from "@/components/settings/WindowSettings";
import { DirectorySettings } from "@/components/settings/DirectorySettings";
import { ImportExportSection } from "@/components/settings/ImportExportSection";
import { AboutSection } from "@/components/settings/AboutSection";
import { useSettings } from "@/hooks/useSettings";
import { useImportExport } from "@/hooks/useImportExport";
import { useTranslation } from "react-i18next";

interface SettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImportSuccess?: () => void | Promise<void>;
}

export function SettingsDialog({
  open,
  onOpenChange,
  onImportSuccess,
}: SettingsDialogProps) {
  const { t } = useTranslation();
  const {
    settings,
    isLoading,
    isSaving,
    isPortable,
    appConfigDir,
    resolvedDirs,
    updateSettings,
    updateDirectory,
    updateAppConfigDir,
    browseDirectory,
    browseAppConfigDir,
    resetDirectory,
    resetAppConfigDir,
    applyWslTemplate,
    saveSettings,
    resetSettings,
    requiresRestart,
    acknowledgeRestart,
  } = useSettings();

  const {
    selectedFile,
    status: importStatus,
    errorMessage,
    backupId,
    isImporting,
    selectImportFile,
    importConfig,
    exportConfig,
    clearSelection,
    resetStatus,
  } = useImportExport({ onImportSuccess });

  const [activeTab, setActiveTab] = useState<string>("general");
  const [showRestartPrompt, setShowRestartPrompt] = useState(false);

  useEffect(() => {
    if (open) {
      setActiveTab("general");
      resetStatus();
    }
  }, [open, resetStatus]);

  useEffect(() => {
    if (requiresRestart) {
      setShowRestartPrompt(true);
    }
  }, [requiresRestart]);

  const closeDialog = useCallback(() => {
    // 取消/直接关闭：恢复到初始设置（包括语言回滚）
    resetSettings();
    acknowledgeRestart();
    clearSelection();
    resetStatus();
    onOpenChange(false);
  }, [
    acknowledgeRestart,
    clearSelection,
    onOpenChange,
    resetSettings,
    resetStatus,
  ]);

  const closeAfterSave = useCallback(() => {
    // 保存成功后关闭：不再重置语言，避免需要“保存两次”才生效
    acknowledgeRestart();
    clearSelection();
    resetStatus();
    onOpenChange(false);
  }, [acknowledgeRestart, clearSelection, onOpenChange, resetStatus]);

  const handleDialogChange = useCallback(
    (nextOpen: boolean) => {
      if (!nextOpen) {
        closeDialog();
      } else {
        onOpenChange(true);
      }
    },
    [closeDialog, onOpenChange],
  );

  const handleCancel = useCallback(() => {
    closeDialog();
  }, [closeDialog]);

  const handleSave = useCallback(async () => {
    try {
      const result = await saveSettings();
      if (!result) return;
      if (result.requiresRestart) {
        setShowRestartPrompt(true);
        return;
      }
      closeAfterSave();
    } catch (error) {
      console.error("[SettingsDialog] Failed to save settings", error);
    }
  }, [closeAfterSave, saveSettings]);

  const handleRestartLater = useCallback(() => {
    setShowRestartPrompt(false);
    closeAfterSave();
  }, [closeAfterSave]);

  const handleRestartNow = useCallback(async () => {
    setShowRestartPrompt(false);
    if (import.meta.env.DEV) {
      toast.success(t("settings.devModeRestartHint"));
      closeAfterSave();
      return;
    }

    try {
      await settingsApi.restart();
    } catch (error) {
      console.error("[SettingsDialog] Failed to restart app", error);
      toast.error(t("settings.restartFailed"));
    } finally {
      closeAfterSave();
    }
  }, [closeAfterSave, t]);

  const isBusy = useMemo(() => isLoading && !settings, [isLoading, settings]);

  return (
    <Dialog open={open} onOpenChange={handleDialogChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>{t("settings.title")}</DialogTitle>
          <DialogDescription>
            {t("settings.description", {
              defaultValue: "管理语言、主题、目录等应用偏好。",
            })}
          </DialogDescription>
        </DialogHeader>

        {isBusy ? (
          <div className="flex min-h-[320px] items-center justify-center">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="flex-1 overflow-y-auto px-6 py-4">
            <Tabs
              value={activeTab}
              onValueChange={setActiveTab}
              className="flex flex-col h-full"
            >
              <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="general">
                  {t("settings.tabGeneral")}
                </TabsTrigger>
                <TabsTrigger value="advanced">
                  {t("settings.tabAdvanced")}
                </TabsTrigger>
                <TabsTrigger value="about">{t("common.about")}</TabsTrigger>
              </TabsList>

              <TabsContent
                value="general"
                className="space-y-6 mt-6 min-h-[400px]"
              >
                {settings ? (
                  <>
                    <LanguageSettings
                      value={settings.language}
                      onChange={(lang) => updateSettings({ language: lang })}
                    />
                    <ThemeSettings />
                    <WindowSettings
                      settings={settings}
                      onChange={updateSettings}
                    />
                  </>
                ) : null}
              </TabsContent>

              <TabsContent
                value="advanced"
                className="space-y-6 mt-6 min-h-[400px]"
              >
                {settings ? (
                  <>
                    <DirectorySettings
                      appConfigDir={appConfigDir}
                      resolvedDirs={resolvedDirs}
                      onAppConfigChange={updateAppConfigDir}
                      onBrowseAppConfig={browseAppConfigDir}
                      onResetAppConfig={resetAppConfigDir}
                      claudeDir={settings.claudeConfigDir}
                      codexDir={settings.codexConfigDir}
                      geminiDir={settings.geminiConfigDir}
                      onDirectoryChange={updateDirectory}
                      onBrowseDirectory={browseDirectory}
                      onResetDirectory={resetDirectory}
                      onApplyWslTemplate={applyWslTemplate}
                    />
                    <ImportExportSection
                      status={importStatus}
                      selectedFile={selectedFile}
                      errorMessage={errorMessage}
                      backupId={backupId}
                      isImporting={isImporting}
                      onSelectFile={selectImportFile}
                      onImport={importConfig}
                      onExport={exportConfig}
                      onClear={clearSelection}
                    />
                  </>
                ) : null}
              </TabsContent>

              <TabsContent value="about" className="mt-6 min-h-[400px]">
                <AboutSection isPortable={isPortable} />
              </TabsContent>
            </Tabs>
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={handleCancel}>
            {t("common.cancel")}
          </Button>
          <Button onClick={handleSave} disabled={isSaving || isBusy}>
            {isSaving ? (
              <span className="inline-flex items-center gap-2">
                <Loader2 className="h-4 w-4 animate-spin" />
                {t("settings.saving")}
              </span>
            ) : (
              <>
                <Save className="mr-2 h-4 w-4" />
                {t("common.save")}
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>

      <Dialog
        open={showRestartPrompt}
        onOpenChange={(open) => !open && handleRestartLater()}
      >
        <DialogContent zIndex="alert" className="max-w-md">
          <DialogHeader>
            <DialogTitle>{t("settings.restartRequired")}</DialogTitle>
            <DialogDescription>
              {t("settings.restartRequiredDescription", {
                defaultValue: "某些更改需要重启应用后才会生效。",
              })}
            </DialogDescription>
          </DialogHeader>
          <div className="px-6">
            <p className="text-sm text-muted-foreground">
              {t("settings.restartRequiredMessage")}
            </p>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={handleRestartLater}>
              {t("settings.restartLater")}
            </Button>
            <Button onClick={handleRestartNow}>
              {t("settings.restartNow")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Dialog>
  );
}
