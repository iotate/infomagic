# InfoMagic - AI 信息图表生成器

InfoMagic 是一款基于 AI 的信息图表生成工具，可以帮助用户制作一系列的专业的信息图表（当然也可以做成PPT）。

## 功能特性

### 创建内容

- **智能大纲生成**：描述你的想法，AI 自动生成包含标题、内容、布局、配图建议的完整大纲
- **自由编辑**：在线编辑大纲和每个页面的 Markdown 内容，随时调整
- **自定义页数**：支持 1-50 页，按需生成

### 生成图片

- **批量生成**：一键为所有页面生成配图
- **单页重生成**：对不满意的单页可重新生成
- **多种尺寸**：支持横屏、竖屏、方形等多种图片比例
- **模板参考**：使用模板图片作为参考，保持风格一致

### 导出分享

- **PNG 图像**：默认生成高清PNG图片
- **PDF 导出**：一键导出高质量 PDF 文件，方便分享和演示

## 自定义配置

- **大模型API**：自定义接入文生文和文生图大模型API
- **模板**：自定义多个模板按需选用，包含封面、内容页、封底三张图片，生成的图片会参考模板布局
- **风格**：自定义多个模板按需选用，描述配色、风格等要求

## 联系作者

GitHub: https://github.com/iotate/infomagic

扫码关注获取更多内容（微信/小红书/抖音 - 舟游AI国）：

| 微信 | 小红书 | 抖音 |
|------|--------|------|
| ![微信](connect/wechat.jpg) | ![小红书](connect/redbook.jpg) | ![抖音](connect/douyin.png) |



## 安装

### Windows

下载 `InfoMagic_x.x.x_x64-setup.exe` 或 `InfoMagic_x.x.x_win64.zip` 安装包，双击安装。

### 从源码构建

```bash
# 安装依赖
npm install

# 构建
npm run tauri:build

# 一键打包（包含 exe + 数据文件夹）
npm run package
```

## 使用指南

### 1. 配置 API

首次使用需要配置 LLM 和图像生成 API：

1. 点击左侧菜单「API」
2. 填写 LLM 配置（用于生成大纲）
   - 提供商：OpenAI / Azure / DashScope / DeepSeek / 自定义
   - API 端点
   - API Key
   - 模型名称
3. 填写图像生成配置（用于生成图片）
4. 点击「测试连接」验证配置
5. 点击「保存配置」

### 2. 创建项目

1. 点击左侧菜单「项目」
2. 点击「新建项目」
3. 输入项目主题
4. 系统将自动创建项目文件夹

### 3. 生成大纲

1. 打开项目后进入「大纲」页面
2. 点击「生成大纲」
3. 输入提示词，描述你想要的信息图表内容
4. 选择生成模式（简单/中等/详细/自定义）
5. 设置期望页面数
6. AI 将生成大纲内容
7. 可以直接编辑 Markdown 内容
8. 点击「保存大纲」

### 4. 切分页面

1. 在大纲页面点击「切分页面」
2. 系统将根据 `---` 分隔符将大纲切分为多个页面文件

### 5. 编辑页面与生成图片

1. 进入「页面」页面
2. 左侧显示页面列表和 Markdown 编辑器
3. 右侧显示图片预览
4. 选择模板和风格
5. 点击「批量生成图片」或「生成当前图片」
6. 点击「导出 PDF」生成最终文件

## 支持的 AI 提供商

### LLM（大纲生成）

| 提供商 | 端点 | 推荐模型 |
|--------|------|----------|
| OpenAI | https://api.openai.com/v1 | gpt-4o |
| Azure OpenAI | https://your-resource.openai.azure.com | gpt-4 |
| DashScope | https://dashscope.aliyuncs.com/compatible-mode/v1 | qwen-max |
| DeepSeek | https://api.deepseek.com/v1 | deepseek-chat |
| 自定义 | 用户自行填写 | 用户自行填写 |

### 图像生成

| 提供商 | 端点 | 推荐模型 |
|--------|------|----------|
| OpenAI | https://api.openai.com/v1/images/generations | gpt-image-2 |
| Azure | https://your-resource.openai.azure.com | gpt-image-2 |
| DashScope | https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis | wanx-v1 |
| Replicate | https://api.replicate.com/v1/predictions | stability-ai/sdxl |
| 自定义 | 用户自行填写 | 用户自行填写 |

> **自定义供应商**：选择"自定义"选项后，可以填写任意兼容 OpenAI API 格式的端点和模型名称，支持更多第三方服务。

## 技术栈

- **前端**：Vue 3 + TypeScript + Ant Design Vue + Pinia + Vue Router
- **后端**：Tauri 2.x + Rust
- **构建**：Vite

## 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 类型检查
npm run build

# 构建 Tauri 应用
npm run tauri:build
```

## 目录结构

```
infomagic/
├── config.yaml          # 应用配置（API 密钥等）
├── templates/           # 模板目录
│   └── default/
│       ├── front-cover.png
│       ├── content.png
│       └── back-cover.png
├── styles/              # 风格目录
│   ├── business.md
│   └── creative.md
└── projects/            # 项目目录
    └── my-project/
        ├── outline.md
        ├── page-1.md
        ├── page-1.png
        └── ...
```

## 许可证

MIT License
