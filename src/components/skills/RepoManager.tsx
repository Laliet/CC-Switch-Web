import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Trash2, ExternalLink, Plus } from "lucide-react";
import { settingsApi } from "@/lib/api";
import type { Skill, SkillRepo } from "@/lib/api/skills";

const normalizeBranch = (value?: string) => (value ?? "").trim();

interface RepoManagerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  repos: SkillRepo[];
  skills: Skill[];
  onAdd: (repo: SkillRepo) => Promise<void>;
  onRemove: (owner: string, name: string) => Promise<void>;
}

export function RepoManager({
  open: isOpen,
  onOpenChange,
  repos,
  skills,
  onAdd,
  onRemove,
}: RepoManagerProps) {
  const { t } = useTranslation();
  const [repoUrl, setRepoUrl] = useState("");
  const [branch, setBranch] = useState("");
  const [skillsPath, setSkillsPath] = useState("");
  const [error, setError] = useState("");

  const skillCountByRepo = useMemo(() => {
    const counts = new Map<string, number>();
    for (const skill of skills) {
      const key = `${skill.repoOwner}/${skill.repoName}:${normalizeBranch(
        skill.repoBranch,
      )}`;
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    return counts;
  }, [skills]);

  const parseRepoUrl = (
    url: string,
  ): { owner: string; name: string } | null => {
    // 支持格式:
    // - https://github.com/owner/name
    // - https://github.com/owner/name/
    // - https://github.com/owner/name.git
    // - git@github.com:owner/name.git
    // - owner/name

    let cleaned = url.trim();
    cleaned = cleaned.replace(/^git@github\.com:/i, "");
    cleaned = cleaned.replace(/^https?:\/\/github\.com\//i, "");
    cleaned = cleaned.replace(/\/+$/, "");
    cleaned = cleaned.replace(/\.git$/i, "");
    cleaned = cleaned.replace(/\/+$/, "");

    const parts = cleaned.split("/");
    if (parts.length === 2 && parts[0] && parts[1]) {
      return { owner: parts[0], name: parts[1] };
    }

    return null;
  };

  const handleAdd = async () => {
    setError("");

    const parsed = parseRepoUrl(repoUrl);
    if (!parsed) {
      setError(t("skills.repo.invalidUrl"));
      return;
    }

    try {
      const normalizedBranch = normalizeBranch(branch);

      await onAdd({
        owner: parsed.owner,
        name: parsed.name,
        branch: normalizedBranch,
        enabled: true,
        skillsPath: skillsPath.trim() || undefined, // 仅在有值时传递
      });

      setRepoUrl("");
      setBranch("");
      setSkillsPath("");
    } catch (e) {
      setError(e instanceof Error ? e.message : t("skills.repo.addFailed"));
    }
  };

  const handleOpenRepo = async (owner: string, name: string) => {
    try {
      await settingsApi.openExternal(`https://github.com/${owner}/${name}`);
    } catch (error) {
      console.error("Failed to open URL:", error);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] flex flex-col p-0">
        {/* 固定头部 */}
        <DialogHeader className="flex-shrink-0 border-b border-border-default px-6 py-4">
          <DialogTitle>{t("skills.repo.title")}</DialogTitle>
          <DialogDescription>{t("skills.repo.description")}</DialogDescription>
        </DialogHeader>

        {/* 可滚动内容区域 */}
        <div className="flex-1 min-h-0 overflow-y-auto px-6 py-4">
          {/* 添加仓库表单 */}
          <div className="space-y-5">
            <div className="space-y-2">
              <Label htmlFor="repo-url">{t("skills.repo.url")}</Label>
              <div className="flex flex-col gap-3">
                <Input
                  id="repo-url"
                  placeholder={t("skills.repo.urlPlaceholder")}
                  value={repoUrl}
                  onChange={(e) => setRepoUrl(e.target.value)}
                  className="flex-1"
                />
                <div className="flex flex-col gap-3 sm:flex-row">
                  <Input
                    id="branch"
                    placeholder={t("skills.repo.branchPlaceholder")}
                    value={branch}
                    onChange={(e) => setBranch(e.target.value)}
                    className="flex-1"
                  />
                  <Input
                    id="skills-path"
                    placeholder={t("skills.repo.pathPlaceholder")}
                    value={skillsPath}
                    onChange={(e) => setSkillsPath(e.target.value)}
                    className="flex-1"
                  />
                  <Button
                    onClick={handleAdd}
                    className="w-full sm:w-auto sm:px-4"
                    variant="mcp"
                    type="button"
                  >
                    <Plus className="h-4 w-4 mr-2" />
                    {t("skills.repo.add")}
                  </Button>
                </div>
              </div>
              {error && <p className="text-xs text-destructive">{error}</p>}
            </div>

            {/* 仓库列表 */}
            <div className="space-y-3">
              <h4 className="text-sm font-medium">{t("skills.repo.list")}</h4>
              {repos.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  {t("skills.repo.empty")}
                </p>
              ) : (
                <div className="space-y-3">
                  {repos.map((repo) => (
                    <div
                      key={`${repo.owner}/${repo.name}`}
                      className="flex items-center justify-between rounded-xl border border-border-default bg-card px-4 py-3"
                    >
                      <div>
                        <div className="text-sm font-medium text-foreground">
                          {repo.owner}/{repo.name}
                        </div>
                        <div className="mt-1 text-xs text-muted-foreground">
                          {t("skills.repo.branch")}:{" "}
                          {normalizeBranch(repo.branch) ||
                            t("skills.repo.defaultBranch", {
                              defaultValue: "default",
                            })}
                          {repo.skillsPath && (
                            <>
                              <span className="mx-2">•</span>
                              {t("skills.repo.path")}: {repo.skillsPath}
                            </>
                          )}
                          <span className="ml-3 inline-flex items-center rounded-full border border-border-default px-2 py-0.5 text-[11px]">
                            {t("skills.repo.skillCount", {
                              count:
                                skillCountByRepo.get(
                                  `${repo.owner}/${repo.name}:${normalizeBranch(
                                    repo.branch,
                                  )}`,
                                ) ?? 0,
                            })}
                          </span>
                        </div>
                      </div>
                      <div className="flex gap-2">
                        <Button
                          variant="ghost"
                          size="icon"
                          type="button"
                          onClick={() => handleOpenRepo(repo.owner, repo.name)}
                          title={t("common.view", { defaultValue: "查看" })}
                        >
                          <ExternalLink className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          type="button"
                          onClick={() => onRemove(repo.owner, repo.name)}
                          title={t("common.delete")}
                          className="hover:text-red-500 hover:bg-red-100 dark:hover:text-red-400 dark:hover:bg-red-500/10"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
