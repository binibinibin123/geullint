import { resolveWorkspacePaths } from "./configuration-paths";

export interface ConfigurationReader {
  get<T>(section: string, defaultValue: T): T;
}

export interface LspConfiguration {
  profile: string;
  userDictionary: string[];
  dictionaryOverlay: string[];
  dictionaryOverlayPaths: string[];
  rulePacks: string[];
}

export function firstWorkspaceFolderUri<T>(
  folders: readonly { uri: T }[] | undefined,
): T | undefined {
  return folders?.[0]?.uri;
}

export function createLspConfiguration(
  configuration: ConfigurationReader,
  workspaceRoot: string | undefined,
  platform: NodeJS.Platform = process.platform,
): LspConfiguration {
  return {
    profile: configuration.get<string>("profile", "default"),
    userDictionary: configuration.get<string[]>("userDictionary", []),
    dictionaryOverlay: configuration.get<string[]>("dictionaryOverlay", []),
    dictionaryOverlayPaths: resolveWorkspacePaths(
      configuration.get<string[]>("dictionaryOverlayPaths", []),
      workspaceRoot,
      platform,
    ),
    rulePacks: resolveWorkspacePaths(
      configuration.get<string[]>("rulePacks", []),
      workspaceRoot,
      platform,
    ),
  };
}
