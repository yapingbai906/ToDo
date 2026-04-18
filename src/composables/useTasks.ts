import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Task } from '../types/task'

const tasks = ref<Task[]>([])
const filter = ref<'all' | 'pending' | 'completed'>('all')

export function useTasks() {
  const filteredTasks = computed(() => {
    if (filter.value === 'pending') return tasks.value.filter(t => !t.completed)
    if (filter.value === 'completed') return tasks.value.filter(t => t.completed)
    return tasks.value
  })

  const pendingCount = computed(() => tasks.value.filter(t => !t.completed).length)

  async function loadTasks() {
    try {
      const loaded = await invoke<Task[]>('load_tasks')
      tasks.value = loaded
    } catch (e) {
      console.error('Failed to load tasks:', e)
    }
  }

  async function addTask(text: string, ddl?: number) {
    if (!text.trim()) return
    try {
      await invoke<Task>('add_task', { text: text.trim(), ddl })
    } catch (e) {
      console.error('Failed to add task:', e)
    }
  }

  async function toggleTask(id: string) {
    try {
      await invoke('toggle_task', { id })
    } catch (e) {
      console.error('Failed to toggle task:', e)
    }
  }

  async function deleteTask(id: string) {
    try {
      await invoke('delete_task', { id })
    } catch (e) {
      console.error('Failed to delete task:', e)
    }
  }

  function setFilter(f: 'all' | 'pending' | 'completed') {
    filter.value = f
  }

  return {
    tasks,
    filteredTasks,
    filter,
    pendingCount,
    loadTasks,
    addTask,
    toggleTask,
    deleteTask,
    setFilter,
  }
}
