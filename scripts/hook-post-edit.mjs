import { execSync } from "child_process";

// PostToolUse hook for Edit/Write tools
// ログは標準エラー出力（exit 0以外の時のみ表示される）
const log = (msg) => console.error(`[HOOK] ${msg}`);
const logError = (msg) => console.error(`[HOOK ERROR] ${msg}`);

// 標準入力からhookデータを読み取り
let input = "";
process.stdin.setEncoding("utf8");
for await (const chunk of process.stdin) {
  input += chunk;
}

try {
  const data = JSON.parse(input);
  const filePath = data.tool_input?.file_path || "";

  log(`Parsed: tool_name=${data.tool_name}, file_path=${filePath}`);

  if (/\.tsx?$/.test(filePath)) {
    log("Running TypeScript check...");
    execSync("pnpm ts:lint", { stdio: "inherit" });
    execSync("pnpm ts:fmt:check", { stdio: "inherit" });
    log("TypeScript check completed");
  } else if (/\.m?js$/.test(filePath)) {
    log("Running JavaScript check...");
    execSync("pnpm js:lint", { stdio: "inherit" });
    execSync("pnpm js:fmt:check", { stdio: "inherit" });
    log("JavaScript check completed");
  } else if (filePath.endsWith(".rs")) {
    log("Running Rust check...");
    execSync("pnpm rs:lint", { stdio: "inherit" });
    execSync("pnpm rs:fmt:check", { stdio: "inherit" });
    log("Rust check completed");
  } else {
    log(`No check needed for: ${filePath}`);
  }

  log("Hook finished successfully");
} catch (e) {
  logError(e.message);
  process.exit(2);
}
