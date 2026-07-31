import path from "node:path";

export function resolveWorkspacePaths(
  candidates: readonly string[],
  workspaceRoot: string | undefined,
  platform: NodeJS.Platform = process.platform,
): string[] {
  const pathApi = platform === "win32" ? path.win32 : path.posix;
  return candidates.map((candidate) => {
    if (pathApi.isAbsolute(candidate) || workspaceRoot === undefined) {
      return candidate;
    }
    return pathApi.resolve(workspaceRoot, candidate);
  });
}
