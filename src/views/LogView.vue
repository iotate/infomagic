<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const logContent = ref('')
const searchQuery = ref('')
const loading = ref(false)
const clearing = ref(false)
const autoRefresh = ref(false)
let refreshInterval: number | null = null

const filteredLines = computed(() => {
  if (!logContent.value) return []
  const lines = logContent.value.split('\n')
  if (!searchQuery.value.trim()) return lines
  const query = searchQuery.value.toLowerCase()
  return lines.filter(line => line.toLowerCase().includes(query))
})

const lineCount = computed(() => {
  return filteredLines.value.length
})

onMounted(async () => {
  await loadLog()
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

async function loadLog() {
  loading.value = true
  try {
    logContent.value = await invoke<string>('load_error_log')
  } catch (e) {
    logContent.value = '无法加载日志文件：' + e
  } finally {
    loading.value = false
  }
}

async function clearLog() {
  if (!confirm('确定要清空日志文件吗？此操作不可恢复。')) {
    return
  }
  
  clearing.value = true
  try {
    await invoke('clear_error_log')
    logContent.value = ''
  } catch (e) {
    alert('清空日志失败：' + e)
  } finally {
    clearing.value = false
  }
}

function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value
  if (autoRefresh.value) {
    refreshInterval = window.setInterval(loadLog, 5000)
  } else if (refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  }
}

function clearSearch() {
  searchQuery.value = ''
}

function scrollToBottom() {
  const container = document.querySelector('.log-container')
  if (container) {
    container.scrollTop = container.scrollHeight
  }
}

function scrollToTop() {
  const container = document.querySelector('.log-container')
  if (container) {
    container.scrollTop = 0
  }
}
</script>

<template>
  <div class="log-page">
    <div class="page-header">
      <h1 class="page-title">错误日志</h1>
    </div>
    <p class="page-desc">查看 error.log 文件内容，用于诊断问题。</p>

    <div class="toolbar">
      <a-space>
        <a-input-search
          v-model:value="searchQuery"
          placeholder="搜索日志..."
          style="width: 300px"
          allow-clear
          @clear="clearSearch"
        />
        <a-button @click="loadLog" :loading="loading">刷新</a-button>
        <a-button :type="autoRefresh ? 'primary' : 'default'" @click="toggleAutoRefresh">
          {{ autoRefresh ? '停止自动刷新' : '自动刷新' }}
        </a-button>
        <a-popconfirm
          title="确定要清空日志文件吗？"
          ok-text="确定"
          cancel-text="取消"
          @confirm="clearLog"
        >
          <a-button type="primary" danger :loading="clearing">清空日志</a-button>
        </a-popconfirm>
      </a-space>
      <a-space>
        <a-button @click="scrollToTop">滚动到顶部</a-button>
        <a-button @click="scrollToBottom">滚动到底部</a-button>
        <span class="line-count">共 {{ lineCount }} 行</span>
      </a-space>
    </div>

    <a-spin :spinning="loading">
      <div class="log-container">
        <div v-if="!logContent" class="empty-state">
          <a-empty description="日志文件为空或不存在" />
        </div>
        <div v-else class="log-content">
          <div 
            v-for="(line, index) in filteredLines" 
            :key="index" 
            class="log-line"
            :class="{
              'log-error': line.includes('ERROR'),
              'log-warn': line.includes('WARN'),
              'log-info': line.includes('INFO')
            }"
          >
            <span class="line-number">{{ index + 1 }}</span>
            <span class="line-text">{{ line }}</span>
          </div>
        </div>
      </div>
    </a-spin>
  </div>
</template>

<style scoped>
.log-page {
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
  margin-bottom: 16px;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 12px 16px;
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.line-count {
  font-size: 13px;
  color: var(--text-secondary);
}

.log-container {
  background: var(--bg-white);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  max-height: calc(100vh - 280px);
  overflow-y: auto;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
}

.log-content {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.6;
}

.log-line {
  display: flex;
  padding: 2px 0;
  border-bottom: 1px solid var(--border-light);
}

.log-line:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.log-error {
  background-color: rgba(255, 77, 79, 0.08);
}

.log-warn {
  background-color: rgba(250, 173, 20, 0.08);
}

.log-info {
  background-color: rgba(24, 144, 255, 0.04);
}

.line-number {
  display: inline-block;
  width: 50px;
  padding: 0 12px;
  color: var(--text-disabled);
  text-align: right;
  flex-shrink: 0;
  user-select: none;
  background-color: var(--bg-color);
  border-right: 1px solid var(--border-light);
}

.line-text {
  padding: 0 12px;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-primary);
}
</style>
