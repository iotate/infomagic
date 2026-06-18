<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface StyleInfo {
  name: string
  path: string
  content?: string
  colors?: string[]
}

const styles = ref<StyleInfo[]>([])
const loading = ref(false)

// 编辑弹窗
const showEditor = ref(false)
const editingName = ref('')
const editingContent = ref('')
const isNewStyle = ref(false)
const saving = ref(false)

onMounted(async () => {
  await loadStyles()
})

async function loadStyles() {
  loading.value = true
  try {
    const files = await invoke<StyleInfo[]>('list_styles')
    // 加载每个风格的内容和提取颜色
    const stylesWithColors = await Promise.all(
      files.map(async (style) => {
        try {
          const content = await invoke<string>('get_style_content', { name: style.name })
          const colors = extractColors(content)
          return { ...style, content, colors }
        } catch {
          return { ...style, colors: [] }
        }
      })
    )
    styles.value = stylesWithColors
  } catch (e) {
    console.error('加载风格失败', e)
    styles.value = []
  } finally {
    loading.value = false
  }
}

// 从风格内容中提取颜色
function extractColors(content: string): string[] {
  const colors: string[] = []
  // 匹配 HEX 颜色 (#RRGGBB 或 #RGB)
  const hexPattern = /#([0-9A-Fa-f]{6}|[0-9A-Fa-f]{3})\b/g
  let match
  while ((match = hexPattern.exec(content)) !== null) {
    const color = match[0].toUpperCase()
    if (!colors.includes(color)) {
      colors.push(color)
    }
    if (colors.length >= 6) break // 最多显示6个颜色
  }
  return colors
}

async function createNewStyle() {
  isNewStyle.value = true
  editingName.value = ''
  editingContent.value = '# 新风格\n\n描述这个风格的特点...\n\n## 配色方案\n- 主色：\n- 辅色：\n- 背景：\n\n## 设计要点\n- '
  showEditor.value = true
}

async function editStyle(style: StyleInfo) {
  isNewStyle.value = false
  editingName.value = style.name
  
  try {
    const content = await invoke<string>('get_style_content', { name: style.name })
    editingContent.value = content
    showEditor.value = true
  } catch (e) {
    alert('加载风格内容失败：' + e)
  }
}

async function saveStyle() {
  if (!editingName.value.trim()) {
    alert('请输入风格名称')
    return
  }
  
  saving.value = true
  try {
    await invoke('save_style', { 
      name: editingName.value.trim(),
      content: editingContent.value 
    })
    
    showEditor.value = false
    await loadStyles()
  } catch (e) {
    alert('保存失败：' + e)
  } finally {
    saving.value = false
  }
}

async function deleteStyle(style: StyleInfo) {
  if (!confirm(`确定要删除风格 "${style.name}" 吗？\n\n文件 ${style.name}.md 将被删除。`)) return
  
  try {
    await invoke('delete_style', { name: style.name })
    await loadStyles()
  } catch (e) {
    alert('删除失败：' + e)
  }
}

function cancelEdit() {
  showEditor.value = false
}
</script>

<template>
  <div class="style-management">
    <div class="page-header">
      <h1 class="page-title">风格管理</h1>
      <a-space>
        <a-button @click="loadStyles" :loading="loading">
          <template #icon>
            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
          </template>
          刷新
        </a-button>
        <a-button type="primary" @click="createNewStyle">
          <template #icon><span>+</span></template>
          新建风格
        </a-button>
      </a-space>
    </div>

    <p class="page-desc">
      风格是一段 Markdown 文本，生成图片时会作为提示词的一部分传递给 AI，影响图片的配色、风格等。
    </p>

    <a-spin :spinning="loading">
      <div v-if="!loading && styles.length === 0" class="empty-state">
        <a-empty description="暂无风格，点击上方按钮创建">
          <a-button type="primary" @click="createNewStyle">新建风格</a-button>
        </a-empty>
      </div>

      <div v-else class="styles-grid">
        <a-card
          v-for="style in styles"
          :key="style.name"
          class="style-card"
        >
          <template #title>
            <div class="style-header">
              <span class="style-name">{{ style.name }}</span>
              <span class="style-filename">{{ style.name }}.md</span>
            </div>
          </template>
          <template #extra>
            <a-space>
              <a-button type="link" size="small" @click="editStyle(style)">编辑</a-button>
              <a-popconfirm
                title="确定要删除此风格吗？"
                ok-text="删除"
                cancel-text="取消"
                @confirm="deleteStyle(style)"
              >
                <a-button type="link" size="small" danger>删除</a-button>
              </a-popconfirm>
            </a-space>
          </template>
          
          <!-- 颜色色块 -->
          <div class="color-palette" v-if="style.colors && style.colors.length > 0">
            <div 
              v-for="color in style.colors" 
              :key="color" 
              class="color-block" 
              :style="{ backgroundColor: color }"
              :title="color"
            ></div>
          </div>
          <div class="no-colors" v-else>
            暂无配色信息
          </div>
        </a-card>
      </div>
    </a-spin>

    <!-- 编辑弹窗 -->
    <a-modal
      v-model:open="showEditor"
      :title="isNewStyle ? '新建风格' : '编辑风格'"
      ok-text="保存"
      cancel-text="取消"
      :confirm-loading="saving"
      @ok="saveStyle"
      @cancel="cancelEdit"
      width="700px"
    >
      <a-form layout="vertical">
        <a-form-item label="风格名称" required>
          <a-input
            v-model:value="editingName"
            placeholder="例如：商务风格、创意风格"
            :disabled="!isNewStyle"
          />
        </a-form-item>
        <a-form-item label="风格内容（Markdown）">
          <a-textarea
            v-model:value="editingContent"
            :rows="12"
            placeholder="描述这个风格的特点，包括配色方案、设计要点等..."
          />
          <template #extra>
            <span class="field-hint">内容将作为图片生成提示词的一部分，建议包含配色方案、设计风格等描述</span>
          </template>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<style scoped>
.style-management {
  max-width: 1000px;
  margin: 0 auto;
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

.styles-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.style-card {
  cursor: default;
}

.style-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.style-name {
  font-weight: 500;
  font-size: 14px;
}

.style-filename {
  font-size: 11px;
  color: var(--text-disabled);
  font-family: monospace;
}

.color-palette {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  padding: 8px 0;
}

.color-block {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: transform 0.2s;
}

.color-block:hover {
  transform: scale(1.1);
}

.no-colors {
  color: var(--text-disabled);
  font-size: 12px;
  padding: 8px 0;
}

.field-hint {
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
