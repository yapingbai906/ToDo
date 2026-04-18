<script setup lang="ts">
import type { Task } from '../types/task'

defineProps<{ task: Task }>()
defineEmits<{ toggle: []; delete: [] }>()

function formatTime(ts: number): string {
  const d = new Date(ts)
  const dateStr = d.toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' })
  const timeStr = d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  return `${dateStr} ${timeStr}`
}

function formatDdl(ts: number): string {
  const d = new Date(ts)
  const monthDay = d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' }).replace('月', '').replace('日', '')
  const time = d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  return `${monthDay} ${time}`
}

/** Dim a hex color for completed state */
function dimColor(hex: string, opacity: number): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `rgba(${r}, ${g}, ${b}, ${opacity})`
}
</script>

<template>
  <div class="task-item" :class="{ completed: task.completed }">
    <!-- Colored accent strip -->
    <div
      class="accent-strip"
      :style="{
        background: task.completed ? dimColor(task.color, 0.3) : task.color,
        boxShadow: task.completed ? 'none' : `0 0 6px ${dimColor(task.color, 0.5)}`,
      }"
    />

    <label class="checkbox-label">
      <input
        type="checkbox"
        :checked="task.completed"
        :style="{ '--task-color': task.color }"
        @change="$emit('toggle')"
      />
      <div class="task-content">
        <div class="task-main">
          <span class="task-text">{{ task.text }}</span>
          <span v-if="task.ddl" class="ddl-badge" :class="{ overdue: task.ddl < Date.now() && !task.completed }">
            <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
              <circle cx="12" cy="12" r="10"/>
              <polyline points="12 6 12 12 16 14"/>
            </svg>
            DDL：{{ formatDdl(task.ddl) }}
          </span>
        </div>
        <span class="task-time">{{ formatTime(task.createdAt) }}</span>
      </div>
    </label>

    <button class="delete-btn" @click.stop="$emit('delete')" aria-label="删除任务">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </button>
  </div>
</template>
