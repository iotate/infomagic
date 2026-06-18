<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'

const router = useRouter()
const projectStore = useProjectStore()

interface PageImageInfo {
  page_num: number
  title: string
  prompt: string
  image_path?: string
  status: 'pending' | 'generating' | 'done' | 'failed'
}

const pages = ref<PageImageInfo[]>([
  { page_num: 1, title: '封面', prompt: '', status: 'pending' },
  { page_num: 2, title: '目录', prompt: '', status: 'pending' },
  { page_num: 3, title: '内容1', prompt: '', status: 'pending' },
  { page_num: 4, title: '内容2', prompt: '', status: 'pending' },
  { page_num: 5, title: '封底', prompt: '', status: 'pending' }
])

const selectedSize = ref('1920x1080')
const selectedStyle = ref('business')
const generating = ref(false)
const currentIndex = ref(0)

const progress = computed(() => {
  const done = pages.value.filter(p => p.status === 'done').length
  return Math.round((done / pages.value.length) * 100)
})

const sizeOptions = [
  { label: '16:9 横屏 (1920×1080)', value: '1920x1080' },
  { label: '16:9 小横屏 (1280×720)', value: '1280x720' },
  { label: '9:16 竖屏 (1080×1920)', value: '1080x1920' },
  { label: '4:3 标准 (1440×1080)', value: '1440x1080' }
]

const styleOptions = [
  { label: '商务风格', value: 'business' },
  { label: '创意风格', value: 'creative' }
]

async function generateAll() {
  generating.value = true
  for (let i = 0; i < pages.value.length; i++) {
    currentIndex.value = i
    pages.value[i].status = 'generating'
    try {
      await new Promise(r => setTimeout(r, 1500))
      pages.value[i].status = 'done'
    } catch (e) {
      pages.value[i].status = 'failed'
    }
  }
  generating.value = false
  alert('所有图片生成完成！')
}

async function regenerateOne(page: PageImageInfo) {
  page.status = 'generating'
  try {
    await new Promise(r => setTimeout(r, 1500))
    page.status = 'done'
    alert('图片重新生成成功！')
  } catch (e) {
    page.status = 'failed'
    alert('生成失败')
  }
}

function getStatusColor(status: string): string {
  const colors: Record<string, string> = { pending: 'default', generating: 'processing', done: 'success', failed: 'error' }
  return colors[status] || 'default'
}

function getStatusLabel(status: string): string {
  const labels: Record<string, string> = { pending: '待生成', generating: '生成中', done: '已完成', failed: '失败' }
  return labels[status] || status
}

function goBack() { router.push({ name: 'pages', params: { id: projectStore.currentProject?.name } }) }
function goNext() { router.push({ name: 'export', params: { id: projectStore.currentProject?.name } }) }
</script>

<template>
  <div class="images-page">
    <div class="page-header"><h1 class="page-title">步骤 4: 图片生成</h1></div>
    <p class="page-desc">为每个页面生成配图，可单独调整提示词和重新生成。</p>

    <a-card class="settings-card">
      <a-space :size="24">
        <div class="setting-item">
          <span class="setting-label">图片尺寸：</span>
          <a-select v-model:value="selectedSize" style="width: 200px">
            <a-select-option v-for="opt in sizeOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</a-select-option>
          </a-select>
        </div>
        <div class="setting-item">
          <span class="setting-label">风格：</span>
          <a-select v-model:value="selectedStyle" style="width: 120px">
            <a-select-option v-for="opt in styleOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</a-select-option>
          </a-select>
        </div>
        <a-button type="primary" :loading="generating" @click="generateAll">生成全部图片</a-button>
      </a-space>
      <div v-if="generating" class="progress-section">
        <a-progress :percent="progress" status="active" />
        <span class="progress-text">正在生成第 {{ currentIndex + 1 }} 页...</span>
      </div>
    </a-card>

    <a-card class="images-card">
      <div class="images-grid">
        <div v-for="page in pages" :key="page.page_num" class="image-card" :class="{ active: page.status === 'generating' }">
          <div class="image-preview">
            <span class="page-number">第 {{ page.page_num }} 页</span>
            <div v-if="page.image_path" class="thumbnail"><img :src="page.image_path" :alt="page.title" /></div>
            <div v-else class="placeholder"><a-spin v-if="page.status === 'generating'" /><span v-else>待生成</span></div>
          </div>
          <div class="image-info"><h4>{{ page.title }}</h4><a-tag :color="getStatusColor(page.status)">{{ getStatusLabel(page.status) }}</a-tag></div>
          <div class="image-actions"><a-button size="small" @click="regenerateOne(page)" :loading="page.status === 'generating'">重新生成</a-button></div>
        </div>
      </div>
    </a-card>

    <div class="page-actions">
      <a-space>
        <a-button @click="goBack">← 上一步</a-button>
        <a-button type="primary" @click="goNext">下一步：导出 →</a-button>
      </a-space>
    </div>
  </div>
</template>

<style scoped>
.images-page { max-width: 1200px; margin: 0 auto; }
.page-header { margin-bottom: 8px; }
.page-title { font-size: 20px; font-weight: 600; margin: 0; color: var(--text-primary); }
.page-desc { color: var(--text-secondary); margin-bottom: 20px; }
.settings-card { margin-bottom: 20px; }
.setting-item { display: flex; align-items: center; gap: 8px; }
.setting-label { color: var(--text-secondary); }
.progress-section { margin-top: 16px; display: flex; align-items: center; gap: 16px; }
.progress-text { color: var(--text-secondary); font-size: 13px; }
.images-card { margin-bottom: 20px; }
.images-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 16px; }
.image-card { border: 1px solid var(--border-color); border-radius: var(--radius-md); overflow: hidden; transition: all 0.2s; }
.image-card.active { border-color: var(--primary-color); box-shadow: 0 0 0 2px rgba(7, 193, 96, 0.2); }
.image-preview { background-color: var(--bg-color); position: relative; }
.page-number { position: absolute; top: 8px; left: 8px; background-color: rgba(0, 0, 0, 0.6); color: #fff; padding: 2px 8px; border-radius: 4px; font-size: 12px; z-index: 1; }
.thumbnail { aspect-ratio: 16/9; overflow: hidden; }
.thumbnail img { width: 100%; height: 100%; object-fit: cover; }
.placeholder { aspect-ratio: 16/9; display: flex; align-items: center; justify-content: center; color: var(--text-disabled); }
.image-info { padding: 12px; display: flex; justify-content: space-between; align-items: center; }
.image-info h4 { margin: 0; font-size: 14px; color: var(--text-primary); }
.image-actions { padding: 0 12px 12px; }
.page-actions { display: flex; justify-content: space-between; }
</style>
