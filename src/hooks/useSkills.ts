import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { skillsApi, type Skill, type SkillRepo } from "@/lib/api/skills";

/**
 * 查询所有技能
 */
export function useAllSkills() {
  return useQuery<Skill[]>({
    queryKey: ["skills", "all"],
    queryFn: async () => {
      const { skills } = await skillsApi.getAll();
      return skills;
    },
  });
}

/**
 * 查询技能仓库列表
 */
export function useSkillRepos() {
  return useQuery<SkillRepo[]>({
    queryKey: ["skills", "repos"],
    queryFn: () => skillsApi.getRepos(),
  });
}

/**
 * 安装技能
 */
export function useInstallSkill() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (directory: string) => skillsApi.install(directory),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "all"] });
    },
  });
}

/**
 * 卸载技能
 */
export function useUninstallSkill() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (directory: string) => skillsApi.uninstall(directory),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "all"] });
    },
  });
}

/**
 * 添加技能仓库
 */
export function useAddSkillRepo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (repo: SkillRepo) => skillsApi.addRepo(repo),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "repos"] });
      queryClient.invalidateQueries({ queryKey: ["skills", "all"] });
    },
  });
}

/**
 * 删除技能仓库
 */
export function useRemoveSkillRepo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ owner, name }: { owner: string; name: string }) =>
      skillsApi.removeRepo(owner, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "repos"] });
      queryClient.invalidateQueries({ queryKey: ["skills", "all"] });
    },
  });
}
