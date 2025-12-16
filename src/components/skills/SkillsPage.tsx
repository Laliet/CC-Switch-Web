import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { RefreshCw, Settings } from "lucide-react";
import { toast } from "sonner";
import { SkillCard } from "./SkillCard";
import { RepoManager } from "./RepoManager";
import { skillsApi, type Skill, type SkillRepo } from "@/lib/api/skills";
import { formatSkillError } from "@/lib/errors/skillErrorParser";

interface SkillsPageProps {
  onClose?: () => void;
}

export function SkillsPage({ onClose: _onClose }: SkillsPageProps = {}) {
  const { t } = useTranslation();
  const [skills, setSkills] = useState<Skill[]>([]);
  const [repos, setRepos] = useState<SkillRepo[]>([]);
  const [loading, setLoading] = useState(true);
  const [repoManagerOpen, setRepoManagerOpen] = useState(false);

  const loadSkills = async (afterLoad?: (data: Skill[]) => void) => {
    try {
      setLoading(true);
      const { skills: data, warnings } = await skillsApi.getAll();
      setSkills(data);
      if (afterLoad) {
        afterLoad(data);
      }
      if (warnings && warnings.length > 0) {
        toast.warning(
          t("skills.repo.fetchWarning", {
            defaultValue: "部分技能仓库获取失败，已显示本地技能",
          }),
          {
            description: warnings.join("\n"),
            duration: 8000,
          },
        );
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      // 传入 "skills.loadFailed" 作为标题
      const { title, description } = formatSkillError(
        errorMessage,
        t,
        "skills.loadFailed",
      );

      toast.error(title, {
        description,
        duration: 8000,
      });

      console.error("Load skills failed:", error);
    } finally {
      setLoading(false);
    }
  };

  const loadRepos = async () => {
    try {
      const data = await skillsApi.getRepos();
      setRepos(data);
    } catch (error) {
      console.error("Failed to load repos:", error);
    }
  };

  useEffect(() => {
    Promise.all([loadSkills(), loadRepos()]);
  }, []);

  const handleInstall = async (directory: string) => {
    try {
      await skillsApi.install(directory);
      toast.success(t("skills.installSuccess", { name: directory }));
      await loadSkills();
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      // 使用错误解析器格式化错误，传入 "skills.installFailed"
      const { title, description } = formatSkillError(
        errorMessage,
        t,
        "skills.installFailed",
      );

      toast.error(title, {
        description,
        duration: 10000, // 延长显示时间让用户看清
      });

      // 打印到控制台方便调试
      console.error("Install skill failed:", {
        directory,
        error,
        message: errorMessage,
      });
    }
  };

  const handleUninstall = async (directory: string) => {
    try {
      await skillsApi.uninstall(directory);
      toast.success(t("skills.uninstallSuccess", { name: directory }));
      await loadSkills();
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      // 使用错误解析器格式化错误，传入 "skills.uninstallFailed"
      const { title, description } = formatSkillError(
        errorMessage,
        t,
        "skills.uninstallFailed",
      );

      toast.error(title, {
        description,
        duration: 10000,
      });

      console.error("Uninstall skill failed:", {
        directory,
        error,
        message: errorMessage,
      });
    }
  };

  const handleAddRepo = async (repo: SkillRepo) => {
    await skillsApi.addRepo(repo);

    let repoSkillCount = 0;
    await Promise.all([
      loadRepos(),
      loadSkills((data) => {
        repoSkillCount = data.filter(
          (skill) =>
            skill.repoOwner === repo.owner &&
            skill.repoName === repo.name &&
            (skill.repoBranch || "main") === (repo.branch || "main"),
        ).length;
      }),
    ]);

    toast.success(
      t("skills.repo.addSuccess", {
        owner: repo.owner,
        name: repo.name,
        count: repoSkillCount,
      }),
    );
  };

  const handleRemoveRepo = async (owner: string, name: string) => {
    await skillsApi.removeRepo(owner, name);
    toast.success(t("skills.repo.removeSuccess", { owner, name }));
    await Promise.all([loadRepos(), loadSkills()]);
  };

  return (
    <div className="flex flex-col h-full min-h-0 bg-background">
      {/* 顶部操作栏（固定区域） */}
      <div className="flex-shrink-0 border-b border-border-default bg-muted/20 px-6 py-4">
        <div className="flex items-center justify-between pr-8">
          <h1 className="text-lg font-semibold leading-tight tracking-tight text-gray-900 dark:text-gray-100">
            {t("skills.title")}
          </h1>
          <div className="flex gap-2">
            <Button
              variant="mcp"
              size="sm"
              onClick={() => loadSkills()}
              disabled={loading}
            >
              <RefreshCw
                className={`h-4 w-4 mr-2 ${loading ? "animate-spin" : ""}`}
              />
              {loading ? t("skills.refreshing") : t("skills.refresh")}
            </Button>
            <Button
              variant="mcp"
              size="sm"
              onClick={() => setRepoManagerOpen(true)}
            >
              <Settings className="h-4 w-4 mr-2" />
              {t("skills.repoManager")}
            </Button>
          </div>
        </div>

        {/* 描述 */}
        <p className="mt-1.5 text-sm text-gray-500 dark:text-gray-400">
          {t("skills.description")}
        </p>
      </div>

      {/* 技能网格（可滚动详情区域） */}
      <div className="flex-1 min-h-0 overflow-y-auto px-6 py-6 bg-muted/10">
        {loading ? (
          <div className="flex items-center justify-center h-64">
            <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : skills.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-center">
            <p className="text-lg font-medium text-gray-900 dark:text-gray-100">
              {t("skills.empty")}
            </p>
            <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">
              {t("skills.emptyDescription")}
            </p>
            <Button
              variant="link"
              onClick={() => setRepoManagerOpen(true)}
              className="mt-3 text-sm font-normal"
            >
              {t("skills.addRepo")}
            </Button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {skills.map((skill) => (
              <SkillCard
                key={skill.key}
                skill={skill}
                onInstall={handleInstall}
                onUninstall={handleUninstall}
              />
            ))}
          </div>
        )}
      </div>

      {/* 仓库管理对话框 */}
      <RepoManager
        open={repoManagerOpen}
        onOpenChange={setRepoManagerOpen}
        repos={repos}
        skills={skills}
        onAdd={handleAddRepo}
        onRemove={handleRemoveRepo}
      />
    </div>
  );
}
