<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()
const route = useRoute()

// 左侧：提示词
const promptContent = ref('')
const savingPrompt = ref(false)

// 右侧：大纲
const outlineContent = ref('')
const saving = ref(false)
const splitting = ref(false)

// 生成大纲状态
const generating = ref(false)
const outlineMode = ref<'simple' | 'medium' | 'detailed' | 'custom'>('medium')
const expectedPages = ref(6)

// 图片尺寸选择
interface ImageSize {
  name: string
  width: number
  height: number
}
const imageSizes = ref<ImageSize[]>([])
const selectedSizeIndex = ref(3) // 默认选中 4:3 标准纵向

const projectId = route.params.id as string

// 模式对应的默认页数范围
const modePageRange = {
  simple: { min: 3, max: 5, default: 4 },
  medium: { min: 6, max: 10, default: 8 },
  detailed: { min: 10, max: 15, default: 12 },
  custom: { min: 1, max: 50, default: 10 }
}

// 监听模式变化，自动调整页数
watch(outlineMode, (newMode) => {
  expectedPages.value = modePageRange[newMode].default
})

onMounted(async () => {
  await Promise.all([
    loadPrompt(),
    loadOutline(),
    loadConfig()
  ])
  // 加载项目尺寸索引
  await loadProjectSizeIndex()
})

async function loadProjectSizeIndex() {
  try {
    const project = await invoke<any>('open_project', { name: projectId })
    if (project && project.size_index !== undefined) {
      selectedSizeIndex.value = project.size_index
    }
  } catch (e) {
    console.error('加载项目尺寸索引失败', e)
  }
}

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

async function loadPrompt() {
  try {
    const content = await invoke<string>('load_prompt', { projectName: projectId })
    promptContent.value = content || ''
  } catch (e) {
    console.error('加载提示词失败', e)
  }
}

async function loadOutline() {
  try {
    const content = await invoke<string>('load_outline', { projectName: projectId })
    outlineContent.value = content || ''
  } catch (e) {
    console.error('加载大纲失败', e)
  }
}

async function savePromptContent() {
  savingPrompt.value = true
  try {
    await invoke('save_prompt', { 
      projectName: projectId, 
      content: promptContent.value 
    })
    // 同时保存尺寸索引到 project.json
    await invoke('update_project_size_index', { 
      name: projectId, 
      sizeIndex: selectedSizeIndex.value 
    })
  } catch (e) {
    console.error('保存提示词失败', e)
  } finally {
    savingPrompt.value = false
  }
}

async function generateOutline() {
  if (!promptContent.value.trim()) {
    alert('请输入提示词')
    return
  }
  
  if (expectedPages.value < 1) {
    alert('期望页面数必须大于0')
    return
  }
  
  generating.value = true
  try {
    // 先保存提示词和尺寸索引
    await savePromptContent()
    
    // 获取配置
    const config = await invoke<any>('load_config')
    
    // 获取选中的尺寸信息
    const selectedSize = imageSizes.value[selectedSizeIndex.value] || { name: '3:4 标准纵向', width: 1080, height: 1440 }
    
    // 构建包含尺寸信息的主题
    const aspectRatio = selectedSize.name.includes('横屏') ? '横向' : selectedSize.name.includes('竖屏') || selectedSize.name.includes('纵向') ? '纵向' : '方形'
    const topicWithSize = `${promptContent.value}\n\n【输出要求】\n- 期望页数：${expectedPages.value} 页\n- 图片尺寸：\n- 尺寸：${selectedSize.name} (${selectedSize.width}×${selectedSize.height})\n- 方向：${aspectRatio}\n- 布局设计需适配此尺寸比例`
    
    // 调用 Tauri 命令生成大纲
    const content = await invoke<string>('generate_outline', {
      topic: topicWithSize,
      mode: outlineMode.value,
      expectedPages: expectedPages.value,
      config: config.llm
    })
    
    outlineContent.value = content
    
    // 自动保存大纲
    try {
      await invoke('save_outline', { 
        projectName: projectId, 
        content: content 
      })
    } catch (saveError) {
      alert('大纲生成成功，但自动保存失败：' + saveError)
    }
  } catch (e) {
    const errorMsg = String(e)
    console.error('大纲生成失败:', errorMsg)
    alert('生成失败：' + errorMsg + '\n\n详细错误已记录到 error.log 文件')
  } finally {
    generating.value = false
  }
}

async function saveOutline() {
  if (!outlineContent.value.trim()) {
    alert('大纲内容为空')
    return
  }
  
  saving.value = true
  try {
    await invoke('save_outline', { 
      projectName: projectId, 
      content: outlineContent.value 
    })
    alert('大纲已保存')
  } catch (e) {
    alert('保存失败：' + e)
  } finally {
    saving.value = false
  }
}

async function splitToPages() {
  if (!outlineContent.value.trim()) {
    alert('请先生成或输入大纲内容')
    return
  }
  
  if (!confirm('确定要将大纲切分为页面文件吗？')) {
    return
  }
  
  splitting.value = true
  try {
    await invoke('split_pages', { name: projectId })
    alert('页面切分成功！')
    router.push({ name: 'pages', params: { id: projectId } })
  } catch (e) {
    alert('切分失败：' + e)
  } finally {
    splitting.value = false
  }
}

function goToProjects() {
  router.push({ name: 'projects' })
}

function goToPages() {
  router.push({ name: 'pages', params: { id: projectId } })
}
</script>

<template>
  <div class="outline-page">
    <div class="page-header">
      <h1 class="page-title">大纲编辑</h1>
      <a-space>
        <a-button @click="goToProjects">← 返回项目</a-button>
        <a-button type="primary" @click="goToPages">前往页面编辑 →</a-button>
      </a-space>
    </div>

    <div class="two-column-layout">
      <!-- 左侧：提示词 -->
      <div class="left-column">
        <div class="column-header">
          <span class="column-title">我想要</span>
          <a-space>
            <a-select v-model:value="selectedSizeIndex" style="width: 100px" size="small">
              <a-select-option v-for="(size, index) in imageSizes" :key="index" :value="index">
                {{ size.name }}
              </a-select-option>
            </a-select>
            <a-input-number 
              v-model:value="expectedPages" 
              :min="1" 
              :max="30" 
              style="width: 50px" 
              size="small"
              placeholder="页数"
            />
            <a-button type="primary" @click="generateOutline" :loading="generating">生成大纲</a-button>
          </a-space>
        </div>
        <div class="column-content">
          <a-textarea
            v-model:value="promptContent"
            class="prompt-editor"
            placeholder="描述你想要的信息图表内容..."
            :auto-size="{ minRows: 18, maxRows: 30 }"
          />
        </div>
      </div>

      <!-- 右侧：大纲 -->
      <div class="right-column">
        <div class="column-header">
          <span class="column-title">大纲内容</span>
          <a-space>
            <a-button @click="saveOutline" :loading="saving">保存大纲</a-button>
            <a-tooltip :title="!outlineContent.trim() ? '请先生成大纲' : ''">
              <a-button @click="splitToPages" :loading="splitting" :disabled="!outlineContent.trim()">切分页面</a-button>
            </a-tooltip>
          </a-space>
        </div>
        <div class="column-content">
          <a-textarea
            v-model:value="outlineContent"
            class="outline-editor"
            placeholder="点击左侧【生成大纲】按钮 AI 生成，或直接输入 Markdown 内容..."
            :auto-size="{ minRows: 18, maxRows: 30 }"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.outline-page {
  max-width: 1400px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.two-column-layout {
  display: grid;
  grid-template-columns: 1fr 2fr;
  gap: 16px;
  margin-bottom: 16px;
}

.left-column,
.right-column {
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  min-height: 500px;
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.column-title {
  font-weight: 500;
  font-size: 14px;
  color: var(--text-primary);
}

.column-content {
  flex: 1;
  padding: 12px;
  display: flex;
  flex-direction: column;
}

.prompt-editor,
.outline-editor {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
  flex: 1;
}
</style>
