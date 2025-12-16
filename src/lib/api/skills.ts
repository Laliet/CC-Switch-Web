import { invoke } from "./adapter";

export interface Skill {
  key: string;
  name: string;
  description: string;
  directory: string;
  readmeUrl?: string;
  installed: boolean;
  repoOwner?: string;
  repoName?: string;
  repoBranch?: string;
  skillsPath?: string; // 技能所在的子目录路径，如 "skills"
}

export interface SkillRepo {
  owner: string;
  name: string;
  branch: string;
  enabled: boolean;
  skillsPath?: string; // 可选：技能所在的子目录路径，如 "skills"
}

export interface SkillsResponse {
  skills: Skill[];
  warnings?: string[];
}

export const skillsApi = {
  async getAll(): Promise<SkillsResponse> {
    const result = await invoke("get_skills");

    if (Array.isArray(result)) {
      return { skills: result as Skill[], warnings: [] };
    }

    const response = result as SkillsResponse;
    return {
      skills: Array.isArray(response?.skills) ? response.skills : [],
      warnings: Array.isArray(response?.warnings) ? response.warnings : [],
    };
  },

  async install(directory: string): Promise<boolean> {
    return await invoke("install_skill", { directory });
  },

  async uninstall(directory: string): Promise<boolean> {
    return await invoke("uninstall_skill", { directory });
  },

  async getRepos(): Promise<SkillRepo[]> {
    return await invoke("get_skill_repos");
  },

  async addRepo(repo: SkillRepo): Promise<boolean> {
    return await invoke("add_skill_repo", { repo });
  },

  async removeRepo(owner: string, name: string): Promise<boolean> {
    return await invoke("remove_skill_repo", { owner, name });
  },
};
