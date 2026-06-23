<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'

async function openGithub() {
  try {
    await openUrl('https://github.com/iotate/infomagic')
  } catch (e) {
    // 如果在浏览器环境或 Tauri API 不可用，使用 window.open
    window.open('https://github.com/iotate/infomagic', '_blank')
  }
}
</script>

<template>
  <div class="about-page">
    <div class="about-header">
      <div class="header-row">
        <h1 class="about-title">关于 InfoMagic</h1>
        <a class="github-link" @click="openGithub">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
          </svg>
          <span>GitHub: iotate/infomagic</span>
        </a>
      </div>
      <p class="about-desc">InfoMagic 是一款基于 AI 的信息图表生成工具，可以帮助用户制作一系列的专业的信息图表（当然也可以做成PPT）。</p>
    </div>

    <div class="about-sections">
      <section class="about-section">
        <h2 class="section-title">创建内容</h2>
        <ul class="feature-list">
          <li><strong>智能大纲生成</strong>：描述你的想法，AI 自动生成包含标题、内容、布局、配图建议的完整大纲</li>
          <li><strong>自由编辑</strong>：在线编辑大纲和每个页面的 Markdown 内容，随时调整</li>
          <li><strong>自定义页数</strong>：支持 1-50 页，按需生成</li>
        </ul>
      </section>

      <section class="about-section">
        <h2 class="section-title">生成图片</h2>
        <ul class="feature-list">
          <li><strong>批量生成</strong>：一键为所有页面生成配图</li>
          <li><strong>单页重生成</strong>：对不满意的单页可重新生成</li>
          <li><strong>多种尺寸</strong>：支持横屏、竖屏、方形等多种图片比例</li>
          <li><strong>模板参考</strong>：使用模板图片作为参考，保持风格一致</li>
        </ul>
      </section>

      <section class="about-section">
        <h2 class="section-title">导出分享</h2>
        <ul class="feature-list">
          <li><strong>PNG 图像</strong>：默认生成高清PNG图片</li>
          <li><strong>PDF 导出</strong>：一键导出高质量 PDF 文件，方便分享和演示</li>
        </ul>
      </section>

      <section class="about-section">
        <h2 class="section-title">自定义配置</h2>
        <ul class="feature-list">
          <li><strong>大模型API</strong>：自定义接入文生文和文生图大模型API</li>
          <li><strong>模板</strong>：自定义多个模板按需选用，包含封面、内容页、封底三张图片，生成的图片会参考模板布局</li>
          <li><strong>风格</strong>：自定义多个模板按需选用，描述配色、风格等要求</li>
        </ul>
      </section>
    </div>
  </div>
</template>

<style scoped>
.about-page {
  max-width: 1920px;
  margin: 0 auto;
  padding: 24px 16px;
}

.about-header {
  margin-bottom: 32px;
}

.header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.about-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.github-link {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--bg-white);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-primary);
  font-size: 13px;
}

.github-link:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.about-desc {
  font-size: 15px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 0;
}

.about-sections {
  display: flex;
  flex-direction: column;
  gap: 24px;
  margin-bottom: 32px;
}

.about-section {
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  padding: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 12px 0;
  color: var(--text-primary);
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.feature-list {
  margin: 0;
  padding-left: 20px;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.8;
}

.feature-list li {
  margin-bottom: 4px;
}

.feature-list strong {
  color: var(--text-primary);
}
</style>
