<script setup lang="ts">
import { onMounted, ref, reactive, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons-vue'

const router = useRouter()

interface ExtraHeader {
  key: string
  value: string
}

interface AppConfig {
  llm: {
    provider: string
    endpoint: string
    api_key: string
    model: string
    extra_headers: ExtraHeader[]
  }
  img: {
    provider: string
    endpoint: string
    api_key: string
    model: string
    extra_headers: ExtraHeader[]
  }
  image_sizes: Array<{ name: string; width: number; height: number }>
}

// LLM 提供商预设
const llmProviders = {
  openai: { endpoint: 'https://api.openai.com/v1', model: 'gpt-5' },
  azure: { endpoint: 'https://your-resource.openai.azure.com', model: 'gpt-5' },
  dashscope: { endpoint: 'https://dashscope.aliyuncs.com/compatible-mode/v1', model: 'qwen-plus-3.6' },
  deepseek: { endpoint: 'https://api.deepseek.com/v1', model: 'deepseek-v4-pro' },
  agnes: { endpoint: 'https://apihub.agnes-ai.com/v1', model: 'agnes-2.0-flash' },
  custom: { endpoint: '', model: '' }
}

// 图像生成提供商预设
const imgProviders = {
  openai: { endpoint: 'https://api.openai.com/v1/images/generations', model: 'gpt-image-2' },
  azure: { endpoint: 'https://your-resource.openai.azure.com', model: 'gpt-image-2' },
  dashscope: { endpoint: 'https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis', model: 'wanx-v1' },
  replicate: { endpoint: 'https://api.replicate.com/v1/predictions', model: 'stability-ai/sdxl' },
  agnes: { endpoint: 'https://apihub.agnes-ai.com/v1/images/generations', model: 'agnes-image-2.0-flash' },
  custom: { endpoint: '', model: '' }
}

const llmConfig = reactive({
  provider: 'openai',
  endpoint: 'https://api.openai.com/v1',
  api_key: '',
  model: 'gpt-5',
  extra_headers: [] as ExtraHeader[]
})

const imgConfig = reactive({
  provider: 'openai',
  endpoint: 'https://api.openai.com/v1/images/generations',
  api_key: '',
  model: 'gpt-image-2',
  extra_headers: [] as ExtraHeader[]
})

const testingLlm = ref(false)
const testingImg = ref(false)
const saving = ref(false)
const llmTestResult = ref<{ success: boolean; message: string } | null>(null)
const imgTestResult = ref<{ success: boolean; message: string } | null>(null)

// 判断是否为自定义提供商
const isCustomLlmProvider = ref(false)
const isCustomImgProvider = ref(false)

// 已知的图像提供商列表
const knownImgProviders = ['openai', 'azure', 'dashscope', 'replicate', 'agnes']

onMounted(async () => {
  await loadConfig()
})

// 监听 LLM 提供商变化，自动填充预设值
watch(() => llmConfig.provider, (newProvider) => {
  const preset = llmProviders[newProvider as keyof typeof llmProviders]
  if (preset && newProvider !== 'custom') {
    llmConfig.endpoint = preset.endpoint
    llmConfig.model = preset.model
    isCustomLlmProvider.value = false
  } else {
    isCustomLlmProvider.value = true
  }
})

// 监听图像提供商变化，自动填充预设值
watch(() => imgConfig.provider, (newProvider) => {
  const preset = imgProviders[newProvider as keyof typeof imgProviders]
  if (preset && newProvider !== 'custom') {
    imgConfig.endpoint = preset.endpoint
    imgConfig.model = preset.model
    isCustomImgProvider.value = false
  } else {
    isCustomImgProvider.value = true
  }
})

async function loadConfig() {
  try {
    const config = await invoke<AppConfig>('load_config')
    if (config) {
      Object.assign(llmConfig, config.llm)
      Object.assign(imgConfig, config.img)
      
      // 确保 extra_headers 存在
      if (!llmConfig.extra_headers) {
        llmConfig.extra_headers = []
      }
      if (!imgConfig.extra_headers) {
        imgConfig.extra_headers = []
      }
      
      // 检查是否为自定义提供商
      const knownLlmProvidersList = ['openai', 'azure', 'dashscope', 'deepseek', 'agnes']
      isCustomLlmProvider.value = !knownLlmProvidersList.includes(config.llm.provider)
      
      isCustomImgProvider.value = !knownImgProviders.includes(config.img.provider)
    }
  } catch (e) {
    console.error('加载配置失败', e)
  }
}

async function testLlm() {
  if (!llmConfig.api_key) {
    llmTestResult.value = { success: false, message: '请输入 API Key' }
    return
  }
  
  if (!llmConfig.endpoint) {
    llmTestResult.value = { success: false, message: '请输入 API 端点' }
    return
  }
  
  testingLlm.value = true
  llmTestResult.value = null
  
  try {
    const result = await invoke<boolean>('test_llm_connection', { config: llmConfig })
    llmTestResult.value = { 
      success: result, 
      message: result ? 'LLM API 连接成功！配置已自动保存' : 'LLM API 连接失败，请检查配置' 
    }
  } catch (e) {
    llmTestResult.value = { success: false, message: `连接错误: ${e}` }
  } finally {
    testingLlm.value = false
  }
}

async function testImg() {
  if (!imgConfig.api_key) {
    imgTestResult.value = { success: false, message: '请输入 API Key' }
    return
  }
  
  if (!imgConfig.endpoint) {
    imgTestResult.value = { success: false, message: '请输入 API 端点' }
    return
  }
  
  testingImg.value = true
  imgTestResult.value = null
  
  try {
    const result = await invoke<boolean>('test_img_connection', { config: imgConfig })
    imgTestResult.value = { 
      success: result, 
      message: result ? '图像 API 连接成功！配置已自动保存' : '图像 API 连接失败，请检查配置' 
    }
  } catch (e) {
    imgTestResult.value = { success: false, message: `连接错误: ${e}` }
  } finally {
    testingImg.value = false
  }
}

async function saveConfig() {
  saving.value = true
  
  try {
    await invoke('save_config', { 
      config: {
        llm: { ...llmConfig },
        img: { ...imgConfig },
        image_sizes: [
          { name: '16:9 横屏', width: 1920, height: 1072 },
          { name: '9:16 竖屏', width: 1072, height: 1920 },
          { name: '4:3 横向', width: 1440, height: 1072 },
          { name: '3:4 纵向', width: 1072, height: 1440 },
          { name: '1:1 方形', width: 1072, height: 1072 }
        ]
      }
    })
    alert('配置保存成功！')
  } catch (e) {
    alert(`保存失败: ${e}`)
  } finally {
    saving.value = false
  }
}

function addExtraHeader() {
  llmConfig.extra_headers.push({ key: '', value: '' })
}

function removeExtraHeader(index: number) {
  llmConfig.extra_headers.splice(index, 1)
}

function addImgExtraHeader() {
  imgConfig.extra_headers.push({ key: '', value: '' })
}

function removeImgExtraHeader(index: number) {
  imgConfig.extra_headers.splice(index, 1)
}

function goBack() {
  router.push({ name: 'projects' })
}
</script>

<template>
  <div class="config-page">
    <div class="page-header">
      <h1 class="page-title">API 配置</h1>
    </div>
    <p class="page-desc">配置 LLM 和图像生成 API，用于生成大纲和图片。选择"自定义"可填写任意兼容 API。</p>

    <a-row :gutter="24">
      <!-- LLM Config -->
      <a-col :span="12">
        <a-card title="LLM 配置（大纲生成）" class="config-card">
          <a-form layout="vertical">
            <a-form-item label="提供商">
              <a-select v-model:value="llmConfig.provider">
                <a-select-option value="openai">OpenAI</a-select-option>
                <a-select-option value="azure">Azure OpenAI</a-select-option>
                <a-select-option value="dashscope">DashScope (阿里云)</a-select-option>
                <a-select-option value="deepseek">DeepSeek</a-select-option>
                <a-select-option value="agnes">Agnes AI</a-select-option>
                <a-select-option value="custom">自定义</a-select-option>
              </a-select>
            </a-form-item>

            <a-form-item label="API 端点" :required="isCustomLlmProvider">
              <a-input
                v-model:value="llmConfig.endpoint"
                :placeholder="isCustomLlmProvider ? 'https://api.example.com/v1' : '自动填充，可修改'"
                :disabled="!isCustomLlmProvider && llmConfig.provider !== 'custom'"
              />
              <template #extra v-if="isCustomLlmProvider">
                <span class="field-hint">自定义 API 端点，需兼容 OpenAI API 格式</span>
              </template>
            </a-form-item>

            <a-form-item label="API Key" required>
              <a-input-password
                v-model:value="llmConfig.api_key"
                placeholder="sk-..."
              />
            </a-form-item>

            <a-form-item label="模型" :required="isCustomLlmProvider">
              <a-input
                v-model:value="llmConfig.model"
                :placeholder="isCustomLlmProvider ? 'gpt-4o / qwen-max / ...' : '自动填充，可修改'"
              />
            </a-form-item>

            <a-form-item label="额外请求头">
              <div class="extra-headers">
                <div v-for="(header, index) in llmConfig.extra_headers" :key="index" class="header-row">
                  <a-input
                    v-model:value="header.key"
                    placeholder="Header Name"
                    style="width: 200px"
                  />
                  <a-input
                    v-model:value="header.value"
                    placeholder="Header Value"
                    style="flex: 1"
                  />
                  <a-button type="text" danger @click="removeExtraHeader(index)">
                    <template #icon><DeleteOutlined /></template>
                  </a-button>
                </div>
                <a-button type="dashed" @click="addExtraHeader">
                  <template #icon><PlusOutlined /></template>
                  添加请求头
                </a-button>
              </div>
              <template #extra>
                <span class="field-hint">添加自定义 HTTP 请求头</span>
              </template>
            </a-form-item>

            <a-form-item>
              <a-space>
                <a-button @click="testLlm" :loading="testingLlm">测试连接</a-button>
                <a-tag v-if="llmTestResult" :color="llmTestResult.success ? 'success' : 'error'">
                  {{ llmTestResult.message }}
                </a-tag>
              </a-space>
            </a-form-item>
          </a-form>
        </a-card>
      </a-col>

      <!-- Image Config -->
      <a-col :span="12">
        <a-card title="图像生成配置" class="config-card">
          <a-form layout="vertical">
            <a-form-item label="提供商">
              <a-select v-model:value="imgConfig.provider">
                <a-select-option value="openai">OpenAI</a-select-option>
                <a-select-option value="azure">Azure</a-select-option>
                <a-select-option value="dashscope">DashScope (阿里云)</a-select-option>
                <a-select-option value="replicate">Replicate</a-select-option>
                <a-select-option value="agnes">Agnes AI</a-select-option>
                <a-select-option value="custom">自定义</a-select-option>
              </a-select>
            </a-form-item>

            <a-form-item label="API 端点" :required="isCustomImgProvider">
              <a-input
                v-model:value="imgConfig.endpoint"
                :placeholder="isCustomImgProvider ? 'https://api.example.com/v1/images/generations' : '自动填充，可修改'"
              />
              <template #extra v-if="isCustomImgProvider">
                <span class="field-hint">自定义图像生成 API 端点</span>
              </template>
            </a-form-item>

            <a-form-item label="API Key" required>
              <a-input-password
                v-model:value="imgConfig.api_key"
                placeholder="sk-..."
              />
            </a-form-item>

            <a-form-item label="模型" :required="isCustomImgProvider">
              <a-input
                v-model:value="imgConfig.model"
                :placeholder="isCustomImgProvider ? 'gpt-image-2 / sdxl / ...' : '自动填充，可修改'"
              />
            </a-form-item>

            <a-form-item label="额外请求头">
              <div class="extra-headers">
                <div v-for="(header, index) in imgConfig.extra_headers" :key="index" class="header-row">
                  <a-input
                    v-model:value="header.key"
                    placeholder="Header Name"
                    style="width: 200px"
                  />
                  <a-input
                    v-model:value="header.value"
                    placeholder="Header Value"
                    style="flex: 1"
                  />
                  <a-button type="text" danger @click="removeImgExtraHeader(index)">
                    <template #icon><DeleteOutlined /></template>
                  </a-button>
                </div>
                <a-button type="dashed" @click="addImgExtraHeader">
                  <template #icon><PlusOutlined /></template>
                  添加请求头
                </a-button>
              </div>
              <template #extra>
                <span class="field-hint">添加自定义 HTTP 请求头（如需要）</span>
              </template>
            </a-form-item>

            <a-form-item>
              <a-space>
                <a-button @click="testImg" :loading="testingImg">测试连接</a-button>
                <a-tag v-if="imgTestResult" :color="imgTestResult.success ? 'success' : 'error'">
                  {{ imgTestResult.message }}
                </a-tag>
              </a-space>
            </a-form-item>
          </a-form>
        </a-card>
      </a-col>
    </a-row>

    <div class="page-actions">
      <a-space>
        <a-button @click="goBack">← 返回</a-button>
        <a-button type="primary" :loading="saving" @click="saveConfig">保存配置</a-button>
      </a-space>
    </div>
  </div>
</template>

<style scoped>
.config-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 16px;
}

.page-header {
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

.extra-headers {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.header-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.field-hint {
  font-size: 12px;
  color: var(--text-secondary);
}

.page-actions {
  margin-top: 24px;
  display: flex;
  justify-content: flex-end;
}

.config-card {
  height: 100%;
}

.field-hint {
  font-size: 12px;
  color: var(--text-secondary);
}

.extra-headers {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.header-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.page-actions {
  margin-top: 24px;
  display: flex;
  justify-content: flex-end;
}
</style>
