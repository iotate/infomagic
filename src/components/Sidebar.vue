<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()

// 第一部分：项目相关菜单
const projectMenuItems = [
  { key: 'projects', icon: 'folder', label: '项目' },
  { key: 'outline', icon: 'file-text', label: '大纲' },
  { key: 'pages', icon: 'copy', label: '页面' }
]

// 第二部分：设置相关菜单
const settingsMenuItems = [
  { key: 'config', icon: 'api', label: 'API' },
  { key: 'templates', icon: 'template', label: '模板' },
  { key: 'styles', icon: 'palette', label: '风格' },
  { key: 'logs', icon: 'logs', label: '日志' },
  { key: 'about', icon: 'info', label: '关于' }
]

const currentKey = computed(() => {
  return route.name as string
})

const isProjectOpen = computed(() => !!projectStore.currentProject)

function handleMenuClick(key: string) {
  if (key === 'outline' || key === 'pages') {
    // 大纲和页面需要打开项目
    if (isProjectOpen.value && projectStore.currentProject) {
      router.push({ name: key, params: { id: projectStore.currentProject.name } })
    }
  } else {
    router.push({ name: key })
  }
}
</script>

<template>
  <header class="topbar">
    <nav class="topbar-nav">
      <!-- 第一部分：项目相关 -->
      <div class="nav-section">
        <div
          v-for="item in projectMenuItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: currentKey === item.key, disabled: (item.key === 'outline' || item.key === 'pages') && !isProjectOpen }"
          @click="handleMenuClick(item.key)"
        >
          <span class="nav-icon">
            <!-- folder icon -->
            <svg v-if="item.icon === 'folder'" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
            </svg>
            <!-- file-text icon -->
            <svg v-else-if="item.icon === 'file-text'" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
            </svg>
            <!-- copy icon -->
            <svg v-else-if="item.icon === 'copy'" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </span>
          <span class="nav-label">{{ item.label }}</span>
        </div>
      </div>

      <div class="nav-divider"></div>

      <!-- 第二部分：设置相关 -->
      <div class="nav-section">
        <div
          v-for="item in settingsMenuItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: currentKey === item.key }"
          @click="handleMenuClick(item.key)"
        >
          <span class="nav-icon">
            <!-- api icon -->
            <svg v-if="item.icon === 'api'" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M14 12l-2 2-2-2 2-2 2 2zm-2-6l2.12 2.12 2.5-2.5L12 1 7.38 5.62l2.5 2.5L12 6zm-6 6l2.12-2.12-2.5-2.5L1 12l4.62 4.62 2.5-2.5L6 12zm12 0l-2.12 2.12 2.5 2.5L23 12l-4.62-4.62-2.5 2.5L18 12zm-6 6l-2.12-2.12-2.5 2.5L12 23l4.62-4.62-2.5-2.5L12 18z"/>
            </svg>
            <!-- template icon -->
            <svg v-else-if="item.icon === 'template'" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 17h2v-4H7v4zm4 0h2V7h-2v10zm4 0h2v-6h-2v6z"/>
            </svg>
            <!-- palette icon -->
            <svg v-else-if="item.icon === 'palette'" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M12 3c-4.97 0-9 4.03-9 9s4.03 9 9 9c.83 0 1.5-.67 1.5-1.5 0-.39-.15-.74-.39-1.01-.23-.26-.38-.61-.38-.99 0-.83.67-1.5 1.5-1.5H16c2.76 0 5-2.24 5-5 0-4.42-4.03-8-9-8zm-5.5 9c-.83 0-1.5-.67-1.5-1.5S5.67 9 6.5 9 8 9.67 8 10.5 7.33 12 6.5 12zm3-4C8.67 8 8 7.33 8 6.5S8.67 5 9.5 5s1.5.67 1.5 1.5S10.33 8 9.5 8zm5 0c-.83 0-1.5-.67-1.5-1.5S13.67 5 14.5 5s1.5.67 1.5 1.5S15.33 8 14.5 8zm3 4c-.83 0-1.5-.67-1.5-1.5S16.67 9 17.5 9s1.5.67 1.5 1.5-.67 1.5-1.5 1.5z"/>
            </svg>
            <!-- logs icon -->
            <svg v-else-if="item.icon === 'logs'" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 7h10v2H7V7zm0 4h10v2H7v-2zm0 4h7v2H7v-2z"/>
            </svg>
            <!-- info icon -->
            <svg v-else-if="item.icon === 'info'" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
            </svg>
          </span>
          <span class="nav-label">{{ item.label }}</span>
        </div>
      </div>
    </nav>

    <!-- 当前项目信息 -->
    <div v-if="isProjectOpen" class="topbar-project">
      <span class="project-label">当前项目:</span>
      <span class="project-name">{{ projectStore.currentProject?.topic }}</span>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 52px;
  background-color: var(--bg-white);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 24px;
  z-index: 100;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.topbar-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.logo-icon {
  width: 24px;
  height: 24px;
}

.logo-text {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.topbar-nav {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
}

.nav-section {
  display: flex;
  align-items: center;
  gap: 2px;
}

.nav-divider {
  width: 1px;
  height: 20px;
  background-color: var(--border-color);
  margin: 0 12px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  font-size: 13px;
}

.nav-item:hover {
  background-color: rgba(7, 193, 96, 0.08);
  color: var(--text-primary);
}

.nav-item.active {
  background-color: rgba(7, 193, 96, 0.12);
  color: var(--primary-color);
}

.nav-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.nav-item.disabled:hover {
  background-color: transparent;
  color: var(--text-secondary);
}

.nav-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.nav-label {
  font-size: 13px;
}

.topbar-project {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background-color: var(--bg-color);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.project-label {
  font-size: 12px;
  color: var(--text-disabled);
}

.project-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
