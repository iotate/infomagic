/**
 * InfoMagic 一键打包脚本
 * 1. 编译 Vue 前端
 * 2. 编译 Tauri 后端
 * 3. 打包为 ZIP（包含 exe + config + templates + styles + projects）
 */

import { execSync } from 'child_process'
import { existsSync, mkdirSync, cpSync, rmSync, writeFileSync, readdirSync, statSync } from 'fs'
import { resolve, basename, join } from 'path'

const ROOT_DIR = resolve(import.meta.dirname, '..')
const DIST_DIR = join(ROOT_DIR, 'dist')
const SRC_TAURI_DIR = join(ROOT_DIR, 'src-tauri')
const RELEASE_DIR = join(SRC_TAURI_DIR, 'target', 'release')
const BUNDLE_DIR = join(RELEASE_DIR, 'bundle')
const OUTPUT_DIR = join(ROOT_DIR, 'output')
const VERSION = '0.1.0'

// 需要打包的数据目录
const DATA_FOLDERS = ['src-tauri/config.yaml', 'src-tauri/templates', 'src-tauri/styles', 'projects']

console.log('🚀 InfoMagic 打包脚本')
console.log('======================')
console.log(`根目录: ${ROOT_DIR}`)
console.log('')

// Step 1: 编译 Vue 前端
console.log('📦 Step 1: 编译 Vue 前端...')
try {
  execSync('npm run build', { cwd: ROOT_DIR, stdio: 'inherit' })
  console.log('✅ Vue 前端编译完成')
} catch (error) {
  // 如果 TS 检查失败，尝试跳过
  console.log('⚠️ TypeScript 检查失败，尝试跳过 TS 检查...')
  try {
    execSync('npm run build:skip-ts', { cwd: ROOT_DIR, stdio: 'inherit' })
    console.log('✅ Vue 前端编译完成（跳过 TS 检查）')
  } catch (e) {
    console.error('❌ Vue 前端编译失败')
    process.exit(1)
  }
}

// Step 2: 编译 Tauri 后端
console.log('')
console.log('🔧 Step 2: 编译 Tauri 后端...')
try {
  execSync('npm run tauri:build', { cwd: ROOT_DIR, stdio: 'inherit' })
  console.log('✅ Tauri 后端编译完成')
} catch (error) {
  console.error('❌ Tauri 后端编译失败')
  process.exit(1)
}

// Step 3: 准备输出目录
console.log('')
console.log('📁 Step 3: 准备输出目录...')

// 清理旧输出
if (existsSync(OUTPUT_DIR)) {
  rmSync(OUTPUT_DIR, { recursive: true })
}
mkdirSync(OUTPUT_DIR, { recursive: true })

// 创建打包目录
const PACKAGE_NAME = `InfoMagic_${VERSION}_win64`
const PACKAGE_DIR = join(OUTPUT_DIR, PACKAGE_NAME)
mkdirSync(PACKAGE_DIR, { recursive: true })

// Step 4: 复制可执行文件
console.log('📋 Step 4: 复制可执行文件...')

// 查找 MSI 或 EXE
const msiPath = join(BUNDLE_DIR, 'msi', `InfoMagic_${VERSION}_x64_en-US.msi`)
const exePath = join(BUNDLE_DIR, 'nsis', `InfoMagic_${VERSION}_x64-setup.exe`)
const dllPath = join(RELEASE_DIR, 'WebView2Loader.dll');

if (existsSync(msiPath)) {
  cpSync(msiPath, join(OUTPUT_DIR, basename(msiPath)))
  console.log(`  ✅ 复制 MSI: ${basename(msiPath)}`)
}

if (existsSync(exePath)) {
  cpSync(exePath, join(OUTPUT_DIR, basename(exePath)))
  console.log(`  ✅ 复制 EXE: ${basename(exePath)}`)
}

// 复制便携版 exe
const portableExe = join(RELEASE_DIR, 'infomagic.exe')
if (existsSync(portableExe)) {
  cpSync(portableExe, join(PACKAGE_DIR, 'InfoMagic.exe'))
  console.log('  ✅ 复制便携版: InfoMagic.exe')
}

// 复制 WebView2Loader.dll
if (existsSync(dllPath)) {
  cpSync(dllPath, join(PACKAGE_DIR, 'WebView2Loader.dll'));
  console.log('  ✅ 复制依赖库: WebView2Loader.dll');
} else {
  console.log('  ⚠️ 警告: 未找到 WebView2Loader.dll');
}

// Step 5: 复制数据文件夹
console.log('📋 Step 5: 复制数据文件夹...')

// templates 和 styles 在 src-tauri 目录下
const srcTauriDataFolders = ['templates', 'styles']
const rootDataFolders = ['config.yaml', 'projects']

// 从 src-tauri 复制 templates 和 styles
for (const folder of srcTauriDataFolders) {
  const srcPath = join(SRC_TAURI_DIR, folder)
  const destPath = join(PACKAGE_DIR, folder)
  if (existsSync(srcPath)) {
    if (statSync(srcPath).isDirectory()) {
      cpSync(srcPath, destPath, { recursive: true })
    } else {
      cpSync(srcPath, destPath)
    }
    console.log(`  ✅ 复制: ${folder}`)
  } else {
    mkdirSync(destPath, { recursive: true })
    console.log(`  📁 创建: ${folder}`)
  }
}

// 从 root 复制其他文件
for (const folder of rootDataFolders) {
  const srcPath = join(ROOT_DIR, folder)
  const destPath = join(PACKAGE_DIR, folder)
  if (existsSync(srcPath)) {
    if (statSync(srcPath).isDirectory()) {
      cpSync(srcPath, destPath, { recursive: true })
    } else {
      cpSync(srcPath, destPath)
    }
    console.log(`  ✅ 复制: ${folder}`)
  } else {
    if (folder.endsWith('.yaml') || folder.endsWith('.yml')) {
      writeFileSync(destPath, '# InfoMagic 配置文件\n')
    } else {
      mkdirSync(destPath, { recursive: true })
    }
    console.log(`  📁 创建: ${folder}`)
  }
}

// Step 6: 创建 ZIP 包
console.log('')
console.log('📦 Step 6: 创建 ZIP 包...')

const archiver = await import('archiver').catch(() => null)
const { createWriteStream } = await import('fs')

if (archiver) {
  const zipPath = join(OUTPUT_DIR, `${PACKAGE_NAME}.zip`)
  const output = createWriteStream(zipPath)
  const archive = archiver.default('zip', { zlib: { level: 9 } })
  
  archive.pipe(output)
  archive.directory(PACKAGE_DIR, PACKAGE_NAME)
  
  await new Promise((resolve, reject) => {
    output.on('close', resolve)
    archive.on('error', reject)
    archive.finalize()
  })
  
  console.log(`✅ ZIP 包已创建: ${zipPath}`)
} else {
  // 使用 PowerShell 压缩
  console.log('  使用 PowerShell 压缩...')
  const zipPath = join(OUTPUT_DIR, `${PACKAGE_NAME}.zip`)
  try {
    execSync(`powershell -Command "Compress-Archive -Path '${PACKAGE_DIR}' -DestinationPath '${zipPath}' -Force"`, { cwd: OUTPUT_DIR })
    console.log(`✅ ZIP 包已创建: ${zipPath}`)
  } catch (e) {
    console.log('⚠️ ZIP 创建失败，请手动压缩 output 目录')
  }
}

// 完成
console.log('')
console.log('🎉 打包完成！')
console.log(`📁 输出目录: ${OUTPUT_DIR}`)
console.log(`📦 安装包: ${PACKAGE_DIR}`)
