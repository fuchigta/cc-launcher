import { execSync } from "child_process";

// 標準入力からhookデータを読み取り
let input = "";
process.stdin.setEncoding("utf8");
for await (const chunk of process.stdin) {
  input += chunk;
}

const data = JSON.parse(input);
const filePath = data.tool_input?.file_path || "";

if (/\.tsx?$/.test(filePath)) {
  console.log("Running TypeScript check...");
  execSync("pnpm lint:ts", { stdio: "inherit" });
} else if (/\.rs$/.test(filePath)) {
  console.log("Running Rust check...");
  execSync("pnpm lint:rs", { stdio: "inherit" });
}
