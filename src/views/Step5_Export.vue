<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'

const router = useRouter()
const projectStore = useProjectStore()

const exportFormat = ref<'pdf' | 'images' | 'both'>('pdf')
const includeNotes = ref(true)
const exporting = ref(false)
const exportComplete = ref(false)

async function doExport() {
  exporting.value = true
  try {
    await new Promise(r => setTimeout(r, 2000))
    exportComplete.value = true
    alert('导出成功！')
  } catch (e) {
    alert('导出失败：' + e)
  } finally {
    exporting.value = false
  }
}

function goBack() { router.push({ name: 'images', params: { id: projectStore.currentProject?.name } }) }
function goToProjects() { router.push({ name: 'projects' }) }
</script>

<template>
  <div class="export-page">
    <div class="page-header"><h1 class="page-title">步骤 5: 导出</h1></div>
    <p class="page-desc">选择导出格式，生成最终的文件。</p>

    <a-card v-if="!exportComplete" class="settings-card">
      <div class="export-options">
        <div class="option-section">
          <h4>导出格式</h4>
          <a-radio-group v-model:value="exportFormat" button-style="solid">
            <a-radio-button value="pdf">PDF 文档</a-radio-button>
            <a-radio-button value="images">图片包</a-radio-button>
            <a-radio-button value="both">全部</a-radio-button>
          </a-radio-group>
        </div>
        <div class="option-section">
          <a-checkbox v-model:checked="includeNotes">包含讲稿备注</a-checkbox>
        </div>
      </div>
      <div class="export-actions">
        <a-button type="primary" size="large" :loading="exporting" @click="doExport">开始导出</a-button>
      </div>
    </a-card>

    <a-card v-else class="result-card">
      <a-result status="success" title="导出成功！" sub-title="文件已保存到项目目录">
        <template #extra>
          <a-space>
            <a-button @click="exportComplete = false">重新导出</a-button>
            <a-button type="primary" @click="goToProjects">返回项目列表</a-button>
          </a-space>
        </template>
      </a-result>
    </a-card>

    <div v-if="!exportComplete" class="page-actions">
      <a-space><a-button @click="goBack">← 上一步</a-button></a-space>
    </div>
  </div>
</template>

<style scoped>
.export-page { max-width: 800px; margin: 0 auto; }
.page-header { margin-bottom: 8px; }
.page-title { font-size: 20px; font-weight: 600; margin: 0; color: var(--text-primary); }
.page-desc { color: var(--text-secondary); margin-bottom: 20px; }
.settings-card { margin-bottom: 20px; }
.export-options { display: flex; flex-direction: column; gap: 24px; }
.option-section h4 { margin-bottom: 12px; color: var(--text-primary); }
.export-actions { margin-top: 24px; display: flex; justify-content: center; }
.result-card { margin-bottom: 20px; }
.page-actions { display: flex; justify-content: space-between; }
</style>
