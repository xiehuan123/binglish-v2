#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

// ── 参数解析 ──────────────────────────────────────────
const args = process.argv.slice(2);
const dryRun = args.includes('--dry-run');
const noProxy = args.includes('--no-proxy');
const bumpType = ['major', 'minor', 'patch'].find(t => args.includes(t)) ?? 'patch';

// ── 颜色工具 ──────────────────────────────────────────
const c = {
  green: s => `\x1b[32m${s}\x1b[0m`,
  red: s => `\x1b[31m${s}\x1b[0m`,
  yellow: s => `\x1b[33m${s}\x1b[0m`,
  cyan: s => `\x1b[36m${s}\x1b[0m`,
  dim: s => `\x1b[2m${s}\x1b[0m`,
};

// ── 代理配置（从环境变量读取，默认 127.0.0.1:7890）────
function buildEnv() {
  if (noProxy) return process.env;
  const proxy = process.env.HTTPS_PROXY || process.env.HTTP_PROXY || 'http://127.0.0.1:7890';
  return {
    ...process.env,
    HTTP_PROXY: proxy,
    HTTPS_PROXY: proxy,
    NO_PROXY: process.env.NO_PROXY || 'gitlab.deepcool.com',
  };
}

const run = (cmd, { proxy = false, silent = false } = {}) => {
  if (dryRun) { console.log(c.dim(`  [dry-run] ${cmd}`)); return; }
  execSync(cmd, { cwd: root, stdio: silent ? 'pipe' : 'inherit', env: proxy ? buildEnv() : process.env });
};

// ── 版本计算 ──────────────────────────────────────────
const versionFiles = ['package.json', 'src-tauri/tauri.conf.json'];
const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf-8'));
const [major, minor, patch] = pkg.version.split('.').map(Number);
const oldVersion = pkg.version;

const newVersion = {
  major: `${major + 1}.0.0`,
  minor: `${major}.${minor + 1}.0`,
  patch: `${major}.${minor}.${patch + 1}`,
}[bumpType];
const tag = `v${newVersion}`;

console.log(c.cyan(`\n📦 bump ${bumpType}: ${oldVersion} → ${newVersion}\n`));
if (dryRun) console.log(c.yellow('  ⚠ dry-run 模式，不会执行实际操作\n'));

// ── 1. 更新版本号 ────────────────────────────────────
for (const file of versionFiles) {
  const full = resolve(root, file);
  const obj = JSON.parse(readFileSync(full, 'utf-8'));
  console.log(`  ${file}: ${c.dim(obj.version)} → ${c.green(newVersion)}`);
  if (!dryRun) {
    obj.version = newVersion;
    writeFileSync(full, JSON.stringify(obj, null, 2) + '\n', 'utf-8');
  }
}

function rollbackVersion() {
  console.error(c.yellow(`\n  回退版本号 ${newVersion} → ${oldVersion} ...`));
  for (const file of versionFiles) {
    const full = resolve(root, file);
    const obj = JSON.parse(readFileSync(full, 'utf-8'));
    obj.version = oldVersion;
    writeFileSync(full, JSON.stringify(obj, null, 2) + '\n', 'utf-8');
  }
  console.error(c.yellow('  版本号已回退'));
}

// ── 2. 打包 ──────────────────────────────────────────
try {
  console.log(c.cyan('\n🔨 开始打包 ...'));
  run('pnpm tauri build', { proxy: true });
} catch {
  console.error(c.red('\n✗ 打包失败!'));
  rollbackVersion();
  process.exit(1);
}

// ── 3. Git 提交 + tag + 推送 ─────────────────────────
try {
  console.log(c.cyan('\n🚀 提交并推送 ...'));
  run(`git add ${versionFiles.join(' ')}`);
  run(`git commit -m "release: ${tag}"`);
  run(`git tag ${tag}`);
  run('git push');
  run(`git push origin ${tag}`);
} catch {
  console.error(c.red('\n✗ Git 操作失败，清理中 ...'));
  try { run(`git tag -d ${tag}`, { silent: true }); } catch { /* ignore */ }
  try { run('git reset --soft HEAD~1', { silent: true }); } catch { /* ignore */ }
  rollbackVersion();
  process.exit(1);
}

console.log(c.green(`\n✓ 发布完成: ${tag}\n`));
