<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

interface TemplateInfo {
  name: string
  path: string
  front_cover_path?: string
  content_path?: string
  back_cover_path?: string
  has_front_cover: boolean
  has_content: boolean
  has_back_cover: boolean
}

const templates = ref<TemplateInfo[]>([])
const loading = ref(false)

onMounted(async () => {
  await loadTemplates()
})

async function loadTemplates() {
  loading.value = true
  try {
    templates.value = await invoke<TemplateInfo[]>('list_templates')
  } catch (e) {
    console.error('加载模板失败', e)
    templates.value = []
  } finally {
    loading.value = false
  }
}

async function deleteTemplate(template: TemplateInfo) {
  if (template.name === 'default') {
    alert('默认模板不能删除')
    return
  }
  
  if (!confirm(`确定要删除模板 "${template.name}" 吗？`)) return
  
  try {
    // TODO: 实现删除模板命令
    alert('删除模板功能待实现')
  } catch (e) {
    alert('删除失败：' + e)
  }
}

function getPreviewUrl(template: TemplateInfo, type: 'front' | 'content' | 'back'): string | undefined {
  let path: string | undefined
  if (type === 'front') path = template.front_cover_path
  else if (type === 'content') path = template.content_path
  else path = template.back_cover_path
  
  if (!path) return undefined
  
  // Use Tauri's convertFileSrc to create proper asset URL
  // Normalize path separators for Tauri asset protocol
  // Windows paths use backslashes, but asset protocol expects forward slashes
  const normalizedPath = path.replace(/\\/g, '/');
  return convertFileSrc(normalizedPath, 'asset')
}
</script>

<template>
  <div class="template-management">
    <div class="page-header">
      <h1 class="page-title">模板管理</h1>
      <a-button @click="loadTemplates" :loading="loading">
        <template #icon>
          <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
        </template>
        刷新
      </a-button>
    </div>

    <p class="page-desc">
      模板包含封面、内容页、封底三张图片，用于指导 AI 生成信息图表的风格和布局。
    </p>

    <a-spin :spinning="loading">
      <div v-if="!loading && templates.length === 0" class="empty-state">
        <a-empty description="暂无模板，请在 templates 文件夹中添加模板" />
      </div>

      <div v-else class="templates-grid">
        <a-card
          v-for="template in templates"
          :key="template.name"
          class="template-card"
        >
          <template #title>
            <span class="template-name">{{ template.name }}</span>
          </template>
          <template #extra>
            <a-popconfirm
              v-if="template.name !== 'default'"
              title="确定要删除此模板吗？"
              ok-text="删除"
              cancel-text="取消"
              @confirm="deleteTemplate(template)"
            >
              <a-button type="link" size="small" danger>删除</a-button>
            </a-popconfirm>
          </template>

          <div class="template-preview">
            <div class="preview-item" :class="{ missing: !template.has_front_cover }">
              <div class="preview-thumb">
                <span v-if="!template.has_front_cover">无</span>
                <img v-else :src="getPreviewUrl(template, 'front')" alt="封面" />
              </div>
              <span class="preview-label">封面</span>
              <span class="preview-filename">front-cover.png</span>
            </div>
            <div class="preview-item" :class="{ missing: !template.has_content }">
              <div class="preview-thumb">
                <span v-if="!template.has_content">无</span>
                <img v-else :src="getPreviewUrl(template, 'content')" alt="内容页" />
              </div>
              <span class="preview-label">内容</span>
              <span class="preview-filename">content.png</span>
            </div>
            <div class="preview-item" :class="{ missing: !template.has_back_cover }">
              <div class="preview-thumb">
                <span v-if="!template.has_back_cover">无</span>
                <img v-else :src="getPreviewUrl(template, 'back')" alt="封底" />
              </div>
              <span class="preview-label">封底</span>
              <span class="preview-filename">back-cover.png</span>
            </div>
          </div>
        </a-card>
      </div>
    </a-spin>

    <!-- 说明区域 -->
    <a-card class="help-card">
      <template #title>
        <span class="help-title">如何添加模板</span>
      </template>
      <div class="help-content">
        <p>在 <code>./templates/</code> 文件夹中创建新的文件夹即可添加模板，文件夹名称即为模板名称。</p>
        <p>每个模板文件夹应包含以下文件：</p>
        <ul>
          <li><code>front-cover.png</code> - 封面图片模板</li>
          <li><code>content.png</code> - 内容页图片模板</li>
          <li><code>back-cover.png</code> - 封底图片模板</li>
        </ul>
        <a-alert
          type="info"
          show-icon
          message="提示"
          description="添加模板文件夹后，点击上方【刷新】按钮即可看到新模板。"
        />
      </div>
    </a-card>
  </div>
</template>

<style scoped>
.template-management {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 16px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.page-desc {
  color: var(--text-secondary);
  margin-bottom: 24px;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
  background: var(--bg-white);
  border-radius: var(--radius-md);
}

.templates-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.template-card {
  cursor: default;
}

.template-name {
  font-weight: 500;
}

.template-preview {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.preview-thumb {
  width: 80px;
  height: 45px;
  background-color: var(--bg-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: var(--text-disabled);
  overflow: hidden;
}

.preview-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.preview-item.missing .preview-thumb {
  border-style: dashed;
  background-color: transparent;
}

.preview-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.preview-filename {
  font-size: 10px;
  color: var(--text-disabled);
  font-family: monospace;
}

.help-card {
  background: var(--bg-white);
}

.help-title {
  font-size: 14px;
  font-weight: 600;
}

.help-content {
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.8;
}

.help-content p {
  margin-bottom: 8px;
}

.help-content ul {
  margin: 0 0 12px 0;
  padding-left: 20px;
}

.help-content li {
  margin-bottom: 4px;
}

.help-content code {
  background: var(--bg-color);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
  color: var(--primary-color);
}
</style>
