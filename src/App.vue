<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useTasks } from './composables/useTasks'
import { useConfig } from './composables/useConfig'
import TaskInput from './components/TaskInput.vue'
import TaskList from './components/TaskList.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import type { Task } from './types/task'

const { filter, filteredTasks, pendingCount, tasks, loadTasks, addTask, toggleTask, deleteTask, setFilter } = useTasks()
const { loadConfig } = useConfig()

const showSettings = ref(false)
let unlisten: UnlistenFn | null = null

onMounted(async () => {
  await loadTasks()
  await loadConfig()

  // Keep frontend in sync with Rust-side state changes
  unlisten = await listen<Task[]>('tasks-changed', (event) => {
    tasks.value = event.payload
  })
})

onUnmounted(() => {
  unlisten?.()
})

async function startDrag() {
  await getCurrentWindow().startDragging()
}
</script>

<template>
  <div class="app">
    <header class="app-header" @mousedown="startDrag">
      <div class="header-left">
        <h1>ToDo</h1>
        <span v-if="pendingCount > 0" class="badge">{{ pendingCount }}</span>
      </div>
      <button class="settings-btn" @click.stop="showSettings = true" title="设置">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
      </button>
    </header>

    <TaskInput @add="(text, ddl) => addTask(text, ddl)" />

    <nav class="filters">
      <button
        v-for="f in (['all', 'pending', 'completed'] as const)"
        :key="f"
        :class="{ active: filter === f }"
        @click="setFilter(f)"
      >
        {{ f === 'all' ? '全部' : f === 'pending' ? '待办' : '已完成' }}
      </button>
    </nav>

    <TaskList
      :tasks="filteredTasks"
      @toggle="toggleTask"
      @delete="deleteTask"
    />

    <SettingsPanel :open="showSettings" @close="showSettings = false" />
  </div>
</template>
