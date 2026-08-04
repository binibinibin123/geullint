import fs from "node:fs";

import * as vscode from "vscode";
import {
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

import { createLspConfiguration, firstWorkspaceFolderUri } from "./configuration";
import { createRuleQuickPickItems, type RuleCatalog } from "./rule-catalog";
import { resolveServerCommand } from "./server-path";
import { collectSafeFixEdits, type GeulLintDiagnostic } from "./safe-fixes";

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const workspaceUri = firstWorkspaceFolderUri(vscode.workspace.workspaceFolders);
  const configuration = vscode.workspace.getConfiguration("geullint", workspaceUri);
  const lspConfiguration = () => {
    const workspaceUri = firstWorkspaceFolderUri(vscode.workspace.workspaceFolders);
    return createLspConfiguration(
      vscode.workspace.getConfiguration("geullint", workspaceUri),
      workspaceUri?.fsPath,
    );
  };
  const configuredPath = configuration.get<string>("serverPath", "");
  const command = resolveServerCommand({
    extensionPath: context.extensionPath,
    configuredPath,
    platform: process.platform,
    arch: process.arch,
    pathExists: fs.existsSync,
  });
  const serverOptions: ServerOptions = {
    command,
    transport: TransportKind.stdio,
  };
  const clientOptions: LanguageClientOptions = {
    initializationOptions: lspConfiguration(),
    documentSelector: [
      { scheme: "file", language: "markdown" },
      { scheme: "file", language: "javascript" },
      { scheme: "file", language: "javascriptreact" },
      { scheme: "file", language: "typescript" },
      { scheme: "file", language: "typescriptreact" },
      { scheme: "file", language: "python" },
      { scheme: "file", language: "rust" },
      { scheme: "file", language: "plaintext" },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/.geullint.json"),
    },
    outputChannelName: "GeulLint",
  };

  client = new LanguageClient("geullint", "GeulLint", serverOptions, clientOptions);
  await client.start();
  context.subscriptions.push({ dispose: () => void client?.stop() });
  context.subscriptions.push(
    vscode.commands.registerCommand("geullint.fixAllSafe", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;
      const diagnostics = vscode.languages.getDiagnostics(editor.document.uri);
      const edits = collectSafeFixEdits(
        diagnostics.map<GeulLintDiagnostic>((diagnostic) => ({
          range: {
            start: {
              line: diagnostic.range.start.line,
              character: diagnostic.range.start.character,
            },
            end: {
              line: diagnostic.range.end.line,
              character: diagnostic.range.end.character,
            },
          },
          source: diagnostic.source,
          data: (diagnostic as vscode.Diagnostic & { data?: unknown }).data as GeulLintDiagnostic["data"],
        })),
      );
      if (edits.length === 0) {
        await vscode.window.showInformationMessage("적용할 GeulLint 안전 교정이 없습니다.");
        return;
      }
      const workspaceEdit = new vscode.WorkspaceEdit();
      for (const edit of edits) {
        workspaceEdit.replace(
          editor.document.uri,
          new vscode.Range(
            edit.range.start.line,
            edit.range.start.character,
            edit.range.end.line,
            edit.range.end.character,
          ),
          edit.newText,
        );
      }
      await vscode.workspace.applyEdit(workspaceEdit);
    }),
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("geullint.openRuleCatalog", async () => {
      try {
        const catalog = await client?.sendRequest<RuleCatalog>("geullint/rules");
        if (!catalog) return;
        const selected = await vscode.window.showQuickPick(
          createRuleQuickPickItems(catalog),
          {
            title: `GeulLint 규칙 ${catalog.ruleCount}개`,
            placeHolder: "규칙 ID, 분류, 잘못된 표현으로 검색하세요",
            matchOnDescription: true,
            matchOnDetail: true,
          },
        );
        if (selected) {
          await vscode.env.openExternal(vscode.Uri.parse(selected.rule.documentationUrl));
        }
      } catch (error) {
        await vscode.window.showErrorMessage(
          `GeulLint 규칙 목록을 열 수 없습니다: ${String(error)}`,
        );
      }
    }),
  );
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("geullint")) {
        void client?.sendNotification("workspace/didChangeConfiguration", {
          settings: { geullint: lspConfiguration() },
        });
      }
    }),
  );
}

export async function deactivate(): Promise<void> {
  await client?.stop();
}
