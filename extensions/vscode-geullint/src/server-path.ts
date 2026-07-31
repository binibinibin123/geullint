import path from "node:path";

export interface ServerCommandOptions {
  extensionPath: string;
  configuredPath: string;
  platform: NodeJS.Platform;
  arch: string;
  pathExists: (candidate: string) => boolean;
}

export function resolveServerCommand(options: ServerCommandOptions): string {
  const configuredPath = options.configuredPath.trim();
  if (configuredPath.length > 0) {
    return configuredPath;
  }

  const binaryName = options.platform === "win32" ? "geullint-lsp.exe" : "geullint-lsp";
  const bundledPath = path.join(
    options.extensionPath,
    "server",
    `${options.platform}-${options.arch}`,
    binaryName,
  );
  return options.pathExists(bundledPath) ? bundledPath : "geullint-lsp";
}
