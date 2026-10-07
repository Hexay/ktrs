import * as assert from "node:assert";
import * as path from "node:path";
import * as vscode from "vscode";

const repoRoot = path.resolve(__dirname, "../../../..");
const binary = process.env.KTRS_BIN ?? path.join(repoRoot, "target", "debug", process.platform === "win32" ? "ktrs.exe" : "ktrs");
const fixed = "fun main() {\n    val x = 1\n    println(x)\n}\n";

function ktlintDiagnostics(uri: vscode.Uri): vscode.Diagnostic[] {
  return vscode.languages.getDiagnostics(uri).filter((d) => d.source === "ktlint");
}

function waitForKtlintDiagnostics(uri: vscode.Uri): Promise<vscode.Diagnostic[]> {
  return new Promise((resolve, reject) => {
    const check = () => {
      const found = ktlintDiagnostics(uri);
      if (found.length > 0) {
        listener.dispose();
        clearTimeout(timer);
        resolve(found);
      }
    };
    const listener = vscode.languages.onDidChangeDiagnostics(check);
    const timer = setTimeout(() => {
      listener.dispose();
      reject(new Error("no ktlint diagnostics within 30 s"));
    }, 30000);
    check();
  });
}

async function applyEdits(document: vscode.TextDocument, edits: vscode.TextEdit[]): Promise<void> {
  const edit = new vscode.WorkspaceEdit();
  edit.set(document.uri, edits);
  assert.ok(await vscode.workspace.applyEdit(edit));
}

suite("ktrs lsp", () => {
  let document: vscode.TextDocument;

  suiteSetup(async () => {
    const config = vscode.workspace.getConfiguration("ktrs");
    await config.update("path", binary, vscode.ConfigurationTarget.Global);
    await config.update("format.tool", "ktlint", vscode.ConfigurationTarget.Global);
    const folder = vscode.workspace.workspaceFolders![0].uri;
    document = await vscode.workspace.openTextDocument(vscode.Uri.joinPath(folder, "Main.kt"));
    await vscode.window.showTextDocument(document);
  });

  teardown(async () => {
    await vscode.commands.executeCommand("workbench.action.files.revert");
  });

  test("publishes ktlint diagnostics", async () => {
    const diagnostics = await waitForKtlintDiagnostics(document.uri);
    assert.strictEqual(document.languageId, "kotlin");
    assert.deepStrictEqual(
      diagnostics.map((d) => [d.code, d.range.start.line]),
      [["standard:no-semi", 1]],
    );
  });

  test("fixes all with source.fixAll.ktlint", async () => {
    await waitForKtlintDiagnostics(document.uri);
    const full = new vscode.Range(0, 0, document.lineCount, 0);
    const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>(
      "vscode.executeCodeActionProvider",
      document.uri,
      full,
      "source.fixAll.ktlint",
    );
    const fixAll = actions.find((a) => a.kind?.value === "source.fixAll.ktlint");
    assert.ok(fixAll?.edit, `no fix-all action in ${JSON.stringify(actions.map((a) => a.title))}`);
    assert.ok(await vscode.workspace.applyEdit(fixAll.edit));
    assert.strictEqual(document.getText(), fixed);
  });

  test("formats with ktlint", async () => {
    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      document.uri,
      { tabSize: 4, insertSpaces: true },
    );
    assert.ok(edits && edits.length > 0, "no formatting edits");
    await applyEdits(document, edits);
    assert.strictEqual(document.getText(), fixed);
  });
});
