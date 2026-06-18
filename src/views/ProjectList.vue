<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useProjectStore, type ProjectInfo } from '@/stores/project'

const router = useRouter()
const projectStore = useProjectStore()

const showCreateModal = ref(false)
const newTopic = ref('')
const creating = ref(false)

onMounted(() => {
  loadProjects()
})

async function loadProjects() {
  await projectStore.loadProjects()
}

async function handleCreate() {
  if (!newTopic.value.trim()) return
  
  creating.value = true
  try {
    const project = await projectStore.createProject(newTopic.value.trim(), 'default')
    showCreateModal.value = false
    newTopic.value = ''
    router.push({ name: 'outline', params: { id: project.name } })
  } catch (e) {
    alert('创建失败：' + e)
  } finally {
    creating.value = false
  }
}

function openProject(project: ProjectInfo) {
  router.push({ name: 'outline', params: { id: project.name } })
}

function openProjectOutline(project: ProjectInfo) {
  router.push({ name: 'outline', params: { id: project.name } })
}

function openProjectPages(project: ProjectInfo) {
  router.push({ name: 'pages', params: { id: project.name } })
}

function openProjectImages(project: ProjectInfo) {
  router.push({ name: 'pages', params: { id: project.name } })
}

async function deleteProject(project: ProjectInfo) {
  if (!confirm(`确定要删除项目 "${project.topic}" 吗？\n\n这将删除项目文件夹及所有相关文件。`)) return
  
  try {
    await projectStore.deleteProject(project.name)
  } catch (e) {
    alert('删除失败：' + e)
  }
}

async function openProjectFolder(project: ProjectInfo) {
  try {
    await invoke('open_project_folder', { projectName: project.name })
  } catch (e) {
    alert('打开文件夹失败：' + e)
  }
}

function getPageCount(project: ProjectInfo): number {
  return project.page_count ?? 0
}

function getImageCount(project: ProjectInfo): number {
  return project.image_count ?? 0
}

function hasOutline(project: ProjectInfo): boolean {
  return project.has_outline ?? false
}

function hasPages(project: ProjectInfo): boolean {
  return (project.page_count ?? 0) > 0
}

function hasImages(project: ProjectInfo): boolean {
  return project.has_images ?? false
}
</script>

<template>
  <div class="project-list">
    <div class="page-header">
      <h1 class="page-title">项目列表</h1>
      <a-button type="primary" @click="showCreateModal = true">
        <template #icon><span>+</span></template>
        新建项目
      </a-button>
    </div>

    <a-spin :spinning="projectStore.loading">
      <div v-if="!projectStore.loading && projectStore.sortedProjects.length === 0" class="empty-state">
        <a-empty description="暂无项目，点击上方按钮创建">
          <a-button type="primary" @click="showCreateModal = true">新建项目</a-button>
        </a-empty>
      </div>

      <div v-else class="projects-grid">
        <a-card
          v-for="project in projectStore.sortedProjects"
          :key="project.name"
          class="project-card"
          hoverable
        >
          <template #title>
            <span class="project-title" @click="openProject(project)">{{ project.topic }}</span>
          </template>
          <template #extra>
            <span class="update-date">{{ project.updated_at ? new Date(project.updated_at).toLocaleDateString() : '-' }}</span>
          </template>
          
          <!-- 文件状态图标 -->
          <div class="file-status">
            <div class="status-item" :class="{ active: hasOutline(project) }" @click="hasOutline(project) && openProjectOutline(project)">
              <div class="status-icon">
                <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                  <path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
                </svg>
              </div>
              <span class="status-label">大纲</span>
              <span class="status-check" v-if="hasOutline(project)">✓</span>
            </div>
            
            <div class="status-item" :class="{ active: hasPages(project) }" @click="hasPages(project) && openProjectPages(project)">
              <div class="status-icon">
                <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                  <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 17h2v-4H7v4zm4 0h2V7h-2v10zm4 0h2v-6h-2v6z"/>
                </svg>
              </div>
              <span class="status-label">页面</span>
              <span class="status-count" v-if="hasPages(project)">{{ getPageCount(project) }}</span>
            </div>
            
            <div class="status-item" :class="{ active: hasImages(project) }" @click="hasImages(project) && openProjectImages(project)">
              <div class="status-icon">
                <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                  <path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"/>
                </svg>
              </div>
              <span class="status-label">图片</span>
              <span class="status-count" v-if="hasImages(project)">{{ getImageCount(project) }}</span>
            </div>
          </div>
          
          <template #actions>
            <a-button type="link" @click="openProject(project)">打开</a-button>
            <a-button type="link" @click="openProjectFolder(project)">文件夹</a-button>
            <a-popconfirm
              title="确定要删除此项目吗？"
              description="将删除项目文件夹及所有相关文件"
              ok-text="删除"
              cancel-text="取消"
              ok-type="danger"
              @confirm="deleteProject(project)"
            >
              <a-button type="link" danger>删除</a-button>
            </a-popconfirm>
          </template>
        </a-card>
      </div>
    </a-spin>

    <!-- 新建项目弹窗 -->
    <a-modal
      v-model:open="showCreateModal"
      title="新建项目"
      ok-text="创建"
      cancel-text="取消"
      :confirm-loading="creating"
      @ok="handleCreate"
    >
      <a-form layout="vertical">
        <a-form-item label="项目主题" required>
          <a-input
            v-model:value="newTopic"
            placeholder="输入信息图表主题，如：2025年度销售报告"
            @pressEnter="handleCreate"
          />
        </a-form-item>
        <a-alert
          message="将创建项目文件夹及基础目录结构"
          type="info"
          show-icon
        />
      </a-form>
    </a-modal>
  </div>
</template>

<style scoped>
.project-list {
  max-width: 1000px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
  background: var(--bg-white);
  border-radius: var(--radius-md);
}

.projects-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.project-card {
  cursor: default;
}

.project-title {
  font-weight: 500;
  color: var(--text-primary);
  cursor: pointer;
}

.project-title:hover {
  color: var(--primary-color);
}

.update-date {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 文件状态图标 */
.file-status {
  display: flex;
  justify-content: space-around;
  padding: 12px 0;
  margin-bottom: 12px;
  border-bottom: 1px solid var(--border-light);
}

.status-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  opacity: 0.4;
  transition: opacity 0.2s;
}

.status-item.active {
  opacity: 1;
  cursor: pointer;
}

.status-item.active:hover .status-icon {
  transform: scale(1.1);
}

.status-icon {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--bg-color);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.status-item.active .status-icon {
  background: rgba(7, 193, 96, 0.1);
  color: var(--primary-color);
}

.status-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.status-check {
  position: absolute;
  top: -2px;
  right: -2px;
  width: 14px;
  height: 14px;
  background: var(--primary-color);
  color: white;
  border-radius: 50%;
  font-size: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-count {
  font-size: 11px;
  font-weight: 600;
  color: var(--primary-color);
  background: rgba(7, 193, 96, 0.1);
  padding: 1px 6px;
  border-radius: 10px;
}
</style>
