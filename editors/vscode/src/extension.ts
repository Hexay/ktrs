import * as vscode from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node";
import { resolveBinary } from "./binary";

const SECTION = "ktrs";

let client: LanguageClient | undefined;
let output: vscode.LogOutputChannel;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  output = vscode.window.createOutputChannel("ktrs", { log: true });
  context.subscriptions.push(
    output,
    vscode.commands.registerCommand("ktrs.restart", () => restart(context)),
    vscode.commands.registerCommand("ktrs.showOutput", () => output.show()),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration(`${SECTION}.path`)) {
        void restart(context);
      }
    }),
  );
  await start(context);
}

export async function deactivate(): Promise<void> {
  await stop();
}

/** The `ktrs` section as plain JSON; the server ignores `path` and `trace`. */
function settings(): unknown {
  return JSON.parse(JSON.stringify(vscode.workspace.getConfiguration(SECTION)));
}

async function start(context: vscode.ExtensionContext): Promise<void> {
  const configured = vscode.workspace.getConfiguration(SECTION).get<string>("path", "");
  const command = resolveBinary(configured, context.extensionPath);
  if (!command) {
    const open = "Open Settings";
    const choice = await vscode.window.showErrorMessage(
      "ktrs: no `ktrs` binary for this platform. Install ktrs (https://github.com/Hexay/ktrs#installation) or set `ktrs.path`.",
      open,
    );
    if (choice === open) {
      await vscode.commands.executeCommand("workbench.action.openSettings", `${SECTION}.path`);
    }
    return;
  }
  output.info(`Using ${command}`);
  const serverOptions: ServerOptions = { command, args: ["lsp"] };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ language: "kotlin" }],
    initializationOptions: settings(),
    synchronize: { configurationSection: SECTION },
    outputChannel: output,
    traceOutputChannel: output,
  };
  client = new LanguageClient(SECTION, "ktrs", serverOptions, clientOptions);
  try {
    await client.start();
  } catch (e) {
    output.error(`Failed to start ${command}: ${e}`);
    void vscode.window.showErrorMessage(`ktrs: failed to start \`${command} lsp\`. See the ktrs output.`);
  }
}

async function stop(): Promise<void> {
  const running = client;
  client = undefined;
  if (running?.isRunning()) {
    await running.stop();
  }
}

async function restart(context: vscode.ExtensionContext): Promise<void> {
  await stop();
  await start(context);
}
