<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'


interface PageInfo {
  page_num: number
  title: string
  markdown: string
  image_path?: string
  image_status: 'pending' | 'generating' | 'done' | 'failed'
}

interface PageFileInfo {
  page_num: number
  title: string
  md_path: string
  png_path?: string
  status: string
}

interface TemplateInfo {
  name: string
  path: string
  has_front_cover: boolean
  has_content: boolean
  has_back_cover: boolean
}

interface StyleInfo {
  name: string
  path: string
}

interface ImageSize {
  name: string
  width: number
  height: number
}

// 图片预览弹窗
const previewModalVisible = ref(false)
const previewImageUrl = ref('')
const previewPageIndex = ref(0)

function openImagePreview(imagePath: string) {
  previewImageUrl.value = imagePath
  previewPageIndex.value = selectedPageIndex.value
  previewModalVisible.value = true
}

function closeImagePreview() {
  previewModalVisible.value = false
}

// 打开文件夹
async function openImageFolder(imagePath: string) {
  try {
    // 获取文件夹路径
    const lastSep = imagePath.lastIndexOf('\\')
    const folderPath = lastSep > 0 ? imagePath.substring(0, lastSep) : imagePath.substring(0, imagePath.lastIndexOf('/'))
    await invoke('open_folder', { path: folderPath })
  } catch (e) {
    console.error('打开文件夹失败', e)
    alert('打开文件夹失败：' + e)
  }
}



// Convert file path to asset URL for display (synchronous)
// 添加时间戳参数防止浏览器缓存
function getImageUrl(imagePath: string | undefined): string {
  if (!imagePath) return '';
  // Normalize Windows backslashes to forward slashes
  const normalizedPath = imagePath.replace(/\\/g, '/');
  // convertFileSrc with 'asset' protocol explicitly specified for Tauri v2
  const baseUrl = convertFileSrc(normalizedPath, 'asset');
  // 添加时间戳参数强制刷新缓存
  return `${baseUrl}?t=${Date.now()}`;
}

const router = useRouter()
const route = useRoute()

const pages = ref<PageInfo[]>([])
const selectedPageIndex = ref(0)
const loading = ref(false)
const saving = ref(false)

// 批量生成
const generatingAll = ref(false)
const generatingProgress = ref(0)

// 导出 PDF
const exporting = ref(false)

// 配置选项 - 从后端加载
const templates = ref<TemplateInfo[]>([])
const styles = ref<StyleInfo[]>([])
const imageSizes = ref<ImageSize[]>([])
const selectedTemplate = ref('none')
const selectedStyle = ref('')
const selectedSizeIndex = ref(3) // 默认 4:3，会从 prompt.md 读取

// 版式分配和风格遵循强度
const layoutAssignments = ref<Record<number, string>>({})
const adherenceLevel = ref<'loose' | 'balanced' | 'strict'>('strict')

const projectId = route.params.id as string

const selectedPage = computed(() => pages.value[selectedPageIndex.value] || null)

// 监听选择变化，自动保存项目设置
let saveSettingsTimer: ReturnType<typeof setTimeout> | null = null
watch([selectedTemplate, selectedStyle, selectedSizeIndex], () => {
  // 延迟保存，避免频繁调用
  if (saveSettingsTimer) {
    clearTimeout(saveSettingsTimer)
  }
  saveSettingsTimer = setTimeout(() => {
    saveProjectSettings()
  }, 500)
})

// 键盘事件处理
function handleKeydown(e: KeyboardEvent) {
  // 如果正在输入文本框，不处理方向键
  const target = e.target as HTMLElement
  if (target.tagName === 'TEXTAREA' || target.tagName === 'INPUT') {
    return
  }
  
  if (e.key === 'ArrowLeft') {
    e.preventDefault()
    if (previewModalVisible.value) {
      // 弹窗中切换上一页
      if (previewPageIndex.value > 0) {
        previewPageIndex.value--
        const page = pages.value[previewPageIndex.value]
        if (page?.image_path) {
          previewImageUrl.value = page.image_path
        }
      }
    } else {
      // 正常切换上一页
      if (selectedPageIndex.value > 0) {
        selectPage(selectedPageIndex.value - 1)
      }
    }
  } else if (e.key === 'ArrowRight') {
    e.preventDefault()
    if (previewModalVisible.value) {
      // 弹窗中切换下一页
      if (previewPageIndex.value < pages.value.length - 1) {
        previewPageIndex.value++
        const page = pages.value[previewPageIndex.value]
        if (page?.image_path) {
          previewImageUrl.value = page.image_path
        }
      }
    } else {
      // 正常切换下一页
      if (selectedPageIndex.value < pages.value.length - 1) {
        selectPage(selectedPageIndex.value + 1)
      }
    }
  } else if (e.key === 'Escape' && previewModalVisible.value) {
    closeImagePreview()
  }
}

onMounted(async () => {
  // 添加键盘事件监听
  window.addEventListener('keydown', handleKeydown)
  
  await Promise.all([
    loadPages(),
    loadTemplates(),
    loadStyles(),
    loadConfig()
  ])
  // 配置加载后再读取 prompt.md 中的尺寸
  await loadPromptSize()
})

onUnmounted(() => {
  // 移除键盘事件监听
  window.removeEventListener('keydown', handleKeydown)
})

async function loadConfig() {
  try {
    const config = await invoke<{ image_sizes: ImageSize[] }>('load_config')
    if (config.image_sizes && config.image_sizes.length > 0) {
      imageSizes.value = config.image_sizes
    }
  } catch (e) {
    console.error('加载配置失败', e)
  }
}

async function loadPromptSize() {
  try {
    const project = await invoke<any>('open_project', { name: projectId })
    if (project) {
      if (project.size_index !== undefined) {
        selectedSizeIndex.value = project.size_index
      }
      // 加载项目保存的模板和风格
      if (project.template) {
        selectedTemplate.value = project.template
      }
      if (project.style) {
        selectedStyle.value = project.style
      }
    }
  } catch (e) {
    console.error('加载项目设置失败', e)
  }
}

// 保存项目设置（模板、风格、尺寸）
async function saveProjectSettings() {
  try {
    await invoke('update_project_settings', {
      name: projectId,
      template: selectedTemplate.value === 'none' ? null : selectedTemplate.value || null,
      style: selectedStyle.value || null,
      sizeIndex: selectedSizeIndex.value
    })
  } catch (e) {
    console.error('保存项目设置失败', e)
  }
}

async function loadTemplates() {
  try {
    templates.value = await invoke<TemplateInfo[]>('list_templates')
    // 默认保持 'none'，如果项目有保存的模板则会在 loadPromptSize 中设置
  } catch (e) {
    console.error('加载模板失败', e)
    templates.value = []
  }
}

async function loadStyles() {
  try {
    styles.value = await invoke<StyleInfo[]>('list_styles')
    if (styles.value.length > 0 && !selectedStyle.value) {
      selectedStyle.value = styles.value[0].name
    }
  } catch (e) {
    console.error('加载风格失败', e)
    styles.value = []
  }
}

async function loadPages() {
  loading.value = true
  try {
    const pageFiles = await invoke<PageFileInfo[]>('list_pages', { projectName: projectId })
    
    // 一次性加载所有页面内容
    pages.value = await Promise.all(
      pageFiles.map(async (pf) => {
        let markdown = ''
        try {
          markdown = await invoke<string>('read_page', { projectName: projectId, pageNum: pf.page_num })
        } catch (e) {
          console.error(`加载页面 ${pf.page_num} 内容失败:`, e)
        }
        return {
          page_num: pf.page_num,
          title: pf.title,
          markdown: markdown,
          image_path: pf.png_path,
          image_status: pf.png_path ? 'done' : 'pending'
        }
      })
    )
  } catch (e) {
    console.error('加载页面失败', e)
    pages.value = []
  } finally {
    loading.value = false
  }
}

async function selectPage(index: number) {
  selectedPageIndex.value = index
  // 不再需要每次切换时加载，因为 loadPages 已经加载了所有内容
}

// 保存所有页面内容
async function saveAllPages() {
  if (pages.value.length === 0) {
    alert('没有页面可保存')
    return
  }
  
  saving.value = true
  let savedCount = 0
  let failedCount = 0
  
  // 遍历所有页面保存
  for (const page of pages.value) {
    try {
      // 只保存有内容的页面（markdown 不为空）
      const content = page.markdown ?? ''
      await invoke('save_page', { 
        projectName: projectId, 
        pageNum: page.page_num, 
        content: content 
      })
      savedCount++
    } catch (e) {
      failedCount++
      console.error(`保存页面 ${page.page_num} 失败:`, e)
    }
  }
  
  saving.value = false
  
  if (failedCount === 0) {
    alert(`所有 ${savedCount} 个页面已保存`)
  } else {
    alert(`保存完成：成功 ${savedCount} 个，失败 ${failedCount} 个`)
  }
}

// 是否有已生成的图片
const hasGeneratedImages = computed(() => {
  return pages.value.some(p => p.image_status === 'done')
})

// 生成当前图片
async function generateCurrentImage() {
  if (!selectedPage.value) return

  if (!selectedStyle.value) {
    alert('请先选择风格')
    return
  }
  
  const config = await invoke<any>('load_config')
  const size = imageSizes.value[selectedSizeIndex.value] || { width: 1920, height: 1080 }
  
  // 获取当前页面的版式分配
  const currentLayout = layoutAssignments.value[selectedPage.value.page_num] || null
  
  // 清理已有图片和状态
  selectedPage.value.image_path = undefined
  selectedPage.value.image_status = 'generating'
  
  try {
    // 返回值是图片路径字符串
    const imagePath = await invoke<string>('generate_image', { 
      projectName: projectId,
      pageNum: selectedPage.value.page_num,
      template: selectedTemplate.value === 'none' ? null : selectedTemplate.value || null,
      style: selectedStyle.value || null,
      width: size.width,
      height: size.height,
      config: config.img,
      layoutFamily: currentLayout,
      adherenceLevel: adherenceLevel.value
    })
    selectedPage.value.image_status = 'done'
    // 更新图片路径以立即刷新显示
    if (imagePath) {
      selectedPage.value.image_path = imagePath
    }
    alert('图片生成成功')
  } catch (e) {
    selectedPage.value.image_status = 'failed'
    const errorMsg = String(e)
    console.error('图片生成失败:', errorMsg)
    alert('图片生成失败：' + errorMsg + '\n\n详细信息请查看 error.log 文件')
  }
}

// 批量生成图片
async function generateAllImages() {
  if (!selectedStyle.value) {
    alert('请先选择风格')
    return
  }
  
  const config = await invoke<any>('load_config')
  const size = imageSizes.value[selectedSizeIndex.value] || { width: 1920, height: 1080 }
  
  // 获取版式分配
  try {
    const assignments = await invoke<Record<number, string>>('get_layout_assignments', {
      projectName: projectId,
      styleName: selectedStyle.value,
      pageCount: pages.value.length
    })
    layoutAssignments.value = assignments
  } catch (e) {
    console.error('获取版式分配失败:', e)
  }
  
  generatingAll.value = true
  generatingProgress.value = 0
  
  let successCount = 0
  let failCount = 0
  const errors: string[] = []
  
  // 先清理所有页面的图片和状态
  for (let i = 0; i < pages.value.length; i++) {
    pages.value[i].image_path = undefined
    pages.value[i].image_status = 'pending'
  }
  
  for (let i = 0; i < pages.value.length; i++) {
    pages.value[i].image_status = 'generating'
    try {
      // 获取当前页面的版式分配
      const currentLayout = layoutAssignments.value[pages.value[i].page_num] || null
      
      // 返回值是图片路径字符串
      const imagePath = await invoke<string>('generate_image', { 
        projectName: projectId,
        pageNum: pages.value[i].page_num,
        template: selectedTemplate.value === 'none' ? null : selectedTemplate.value || null,
        style: selectedStyle.value || null,
        width: size.width,
        height: size.height,
        config: config.img,
        layoutFamily: currentLayout,
        adherenceLevel: adherenceLevel.value
      })
      pages.value[i].image_status = 'done'
      // 更新图片路径以立即刷新显示
      if (imagePath) {
        pages.value[i].image_path = imagePath
      }
      successCount++
    } catch (e) {
      pages.value[i].image_status = 'failed'
      failCount++
      errors.push(`页面 ${pages.value[i].page_num}: ${e}`)
      console.error(`页面 ${pages.value[i].page_num} 生成失败:`, e)
    }
    generatingProgress.value = Math.round(((i + 1) / pages.value.length) * 100)
  }
  
  generatingAll.value = false
  
  if (failCount === 0) {
    alert(`所有图片生成成功！共 ${successCount} 张`)
  } else {
    alert(`图片生成完成\n成功: ${successCount} 张\n失败: ${failCount} 张\n\n失败详情:\n${errors.slice(0, 5).join('\n')}${errors.length > 5 ? '\n...' : ''}\n\n详细信息请查看 error.log 文件`)
  }
}

// 导出 PDF
async function exportPdf() {
  // 检查是否有图片
  const hasImages = pages.value.some(p => p.image_status === 'done')
  if (!hasImages) {
    alert('请先生成图片再导出PDF。点击"批量生成"或"生成当前"按钮生成图片。')
    return
  }
  
  // 检查是否有未生成的图片
  const pendingPages = pages.value.filter(p => p.image_status !== 'done')
  if (pendingPages.length > 0) {
    const confirm = window.confirm(`还有 ${pendingPages.length} 页图片未生成，确定要导出吗？`)
    if (!confirm) {
      return
    }
  }
  
  exporting.value = true
  try {
    const pdfPath = await invoke<string>('export_pdf', { 
        projectName: projectId
      })
      alert(`PDF 导出成功！\n文件位置: ${pdfPath}`)
      
      // 打开导出的 PDF 文件所在文件夹
      try {
        await invoke('open_folder', { path: pdfPath })
      } catch (e) {
        console.error('打开文件失败', e)
      }
    } catch (e) {
      console.error('PDF 导出失败', e)
      alert('导出失败：' + e)
    } finally {
      exporting.value = false
    }
  }

function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    pending: 'default',
    generating: 'processing',
    done: 'success',
    failed: 'error'
  }
  return colors[status] || 'default'
}

function goBack() {
  router.push({ name: 'outline', params: { id: projectId } })
}
</script>

<template>
  <div class="pages-page">
    <div class="page-header">
      <h1 class="page-title">页面编辑</h1>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar">
      <a-space>
        <span class="toolbar-label">模板</span>
        <a-select v-model:value="selectedTemplate" style="width: 120px" size="small" placeholder="选择模板">
          <a-select-option value="none">不使用</a-select-option>
          <a-select-option v-for="t in templates" :key="t.name" :value="t.name">{{ t.name }}</a-select-option>
        </a-select>
        <span class="toolbar-label">风格</span>
        <a-select v-model:value="selectedStyle" style="width: 140px" size="small" placeholder="选择风格">
          <a-select-option v-for="s in styles" :key="s.name" :value="s.name">{{ s.name }}</a-select-option>
        </a-select>
        <span class="toolbar-label">尺寸</span>
        <a-select v-model:value="selectedSizeIndex" style="width: 100px" size="small" placeholder="选择尺寸">
          <a-select-option v-for="(size, index) in imageSizes" :key="index" :value="index">{{ size.name }}</a-select-option>
        </a-select>
        <a-divider type="vertical" />
        <a-button @click="saveAllPages" :loading="saving">保存</a-button>
        <a-button type="primary" @click="generateAllImages" :loading="generatingAll">批量生成</a-button>
        <a-button @click="generateCurrentImage" :loading="selectedPage?.image_status === 'generating'">生成当前</a-button>
        <a-tooltip :title="!hasGeneratedImages ? '请先批量生成或生成当前图片' : ''">
          <a-button @click="exportPdf" :disabled="!hasGeneratedImages" :loading="exporting">导出</a-button>
        </a-tooltip>
      </a-space>
      <a-space>
        <a-button @click="goBack">← 返回</a-button>
      </a-space>
    </div>

    <!-- 进度条 -->
    <div v-if="generatingAll" class="progress-bar">
      <a-progress :percent="generatingProgress" status="active" />
    </div>

    <a-spin :spinning="loading">
      <div v-if="pages.length === 0" class="empty-state">
        <a-empty description="暂无页面，请先在大纲页面切分页面" />
      </div>

      <div v-else class="editor-layout">
        <!-- 左侧：页面列表 + Markdown 编辑器 -->
        <div class="left-panel">
          <div class="page-tabs">
            <div
              v-for="(page, index) in pages"
              :key="page.page_num"
              class="page-tab"
              :class="{ active: selectedPageIndex === index }"
              @click="selectPage(index)"
            >
              <span class="tab-num">{{ page.page_num }}</span>
              <a-tag v-if="page.image_status !== 'pending'" :color="getStatusColor(page.image_status)" size="small" style="margin-left: 4px;">
                {{ page.image_status === 'done' ? '✓' : page.image_status === 'generating' ? '...' : '!' }}
              </a-tag>
            </div>
          </div>

          <div class="markdown-editor">
            <div class="editor-header">
              <span>{{ selectedPage?.title || '选择页面' }}</span>
            </div>
            <a-textarea
              v-if="selectedPage"
              v-model:value="selectedPage.markdown"
              :auto-size="{ minRows: 12, maxRows: 20 }"
              placeholder="页面 Markdown 内容..."
            />
          </div>
        </div>

        <!-- 右侧：图片预览 -->
        <div class="right-panel">
          <div class="image-preview-header">
            <span>图片预览</span>
            <a-button 
              v-if="selectedPage?.image_path" 
              size="small" 
              @click="openImageFolder(selectedPage.image_path)"
            >
              打开文件夹
            </a-button>
          </div>
          
          <div class="image-preview" :class="{ clickable: selectedPage?.image_path }">
            <div v-if="!selectedPage?.image_path" class="no-image">
              <a-spin v-if="selectedPage?.image_status === 'generating'" />
              <span v-else>暂无图片</span>
            </div>
            <img v-else :src="getImageUrl(selectedPage.image_path)" :alt="selectedPage.title" @click="openImagePreview(selectedPage.image_path)" />
          </div>

          <div class="image-info">
            <span class="info-item">{{ selectedPage?.page_num }} - {{ selectedPage?.title }}</span>
            <a-tag v-if="selectedPage" :color="getStatusColor(selectedPage.image_status)" size="small">
              {{ selectedPage.image_status === 'done' ? '已生成' : selectedPage.image_status === 'generating' ? '生成中' : '待生成' }}
            </a-tag>
          </div>
        </div>
      </div>
    </a-spin>
  </div>


    <!-- 图片预览弹窗 -->
    <div v-if="previewModalVisible" class="image-modal-overlay" @click="closeImagePreview">
      <div class="image-modal-content" @click.stop>
        <div class="modal-nav-hint">
          <span>← → 切换页面 | ESC 关闭</span>
        </div>
        <img :src="getImageUrl(previewImageUrl)" alt="Preview" />
        <div class="modal-page-info">
          {{ previewPageIndex + 1 }} / {{ pages.length }}
        </div>
      </div>
    </div>
  </template>

<style scoped>
.pages-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 16px;
}

.page-header {
  margin-bottom: 16px;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 10px 16px;
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.toolbar-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.progress-bar {
  margin-bottom: 16px;
  padding: 10px 16px;
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
  background: var(--bg-white);
  border-radius: var(--radius-md);
}

.editor-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  min-height: 450px;
}

.left-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.right-panel {
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.page-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
  padding: 6px;
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.page-tab {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 12px;
}

.page-tab:hover {
  background-color: var(--bg-color);
}

.page-tab.active {
  background-color: var(--primary-color);
  color: white;
}

.tab-num {
  font-weight: 500;
}

.markdown-editor {
  flex: 1;
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  padding: 12px;
  display: flex;
  flex-direction: column;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-weight: 500;
  font-size: 13px;
}

.image-preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 500;
  font-size: 13px;
}

.image-preview {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-color);
  min-height: 200px;
}

.no-image {
  color: var(--text-disabled);
  font-size: 12px;
}

.image-preview img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.image-preview.clickable {
  cursor: pointer;
}

.image-preview.clickable:hover {
  opacity: 0.9;
}

.image-info {
  padding: 8px 12px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.info-item {
  color: var(--text-secondary);
}

.image-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  cursor: pointer;
}

.image-modal-content {
  max-width: 95vw;
  max-height: 95vh;
  display: flex;
  align-items: center;
  justify-content: center;
}

.image-modal-content img {
  max-width: 100%;
  max-height: 90vh;
  object-fit: contain;
}

.modal-nav-hint {
  position: absolute;
  top: 10px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.6);
  color: white;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
}

.modal-page-info {
  position: absolute;
  bottom: 10px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.6);
  color: white;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 14px;
}
</style>
