#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';
import { createInterface } from 'readline';

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

// ── 交互输入 ─────────────────────────────────────────
function ask(question) {
  const rl = createInterface({ input: process.stdin, output: process.stdout });
  return new Promise(resolve => rl.question(question, ans => { rl.close(); resolve(ans.trim()); }));
}

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
  const desc = await ask(c.yellow(`\n📝 请输入 ${tag} 的版本描述: `));
  if (!desc) { console.error(c.red('描述不能为空')); rollbackVersion(); process.exit(1); }
  run(`git add ${versionFiles.join(' ')}`);
  run(`git commit -m "release: ${tag} - ${desc}"`);
  run(`git tag -a ${tag} -m "${desc}"`);
  run('git push', { proxy: true });
  run(`git push origin ${tag}`, { proxy: true });
} catch {
  console.error(c.red('\n✗ Git 操作失败，清理中 ...'));
  try { run(`git tag -d ${tag}`, { silent: true }); } catch { /* ignore */ }
  try { run('git reset --soft HEAD~1', { silent: true }); } catch { /* ignore */ }
  rollbackVersion();
  process.exit(1);
}

// ── 4. 创建 GitHub Release 并上传构建产物 ────────────
try {
  console.log(c.cyan('\n📤 创建 GitHub Release ...'));
  const bundleDir = resolve(root, 'src-tauri/target/release/bundle');
  const assets = [];

  // macOS: .dmg
  const dmgDir = resolve(bundleDir, 'dmg');
  try {
    const dmgFiles = execSync(`ls "${dmgDir}"/*.dmg 2>/dev/null`, { encoding: 'utf-8' }).trim().split('\n').filter(Boolean);
    assets.push(...dmgFiles);
  } catch { /* no dmg */ }

  // Windows: .msi / .exe (NSIS)
  for (const sub of ['msi', 'nsis']) {
    const dir = resolve(bundleDir, sub);
    try {
      const files = execSync(`ls "${dir}"/*.msi "${dir}"/*.exe 2>/dev/null`, { encoding: 'utf-8' }).trim().split('\n').filter(Boolean);
      assets.push(...files);
    } catch { /* no windows artifacts */ }
  }

  if (assets.length === 0) {
    console.log(c.yellow('  ⚠ 未找到构建产物，仅创建 Release（不含附件）'));
  } else {
    console.log(`  找到 ${assets.length} 个构建产物:`);
    assets.forEach(a => console.log(c.dim(`    ${a}`)));
  }

  const assetArgs = assets.map(a => `"${a}"`).join(' ');
  run(`gh release create ${tag} --title "${tag} - ${desc}" --notes "${desc}" ${assetArgs}`, { proxy: true });
} catch (e) {
  console.error(c.yellow(`\n⚠ GitHub Release 创建失败（tag 和代码已推送成功）`));
  console.error(c.dim(`  可手动执行: gh release create ${tag} --title "${tag}" src-tauri/target/release/bundle/dmg/*.dmg`));
}

console.log(c.green(`\n✓ 发布完成: ${tag}\n`));
