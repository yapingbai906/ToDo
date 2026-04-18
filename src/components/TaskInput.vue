<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ add: [text: string, ddl?: number] }>()
const input = ref('')
const showPicker = ref(false)
const pickerRef = ref<HTMLElement | null>(null)
const pickerDate = ref('')
const pickerTime = ref('')

function onDocClick(e: MouseEvent) {
  if (!showPicker.value) return
  const area = document.querySelector('.task-input-area')
  if (area && !area.contains(e.target as Node)) {
    showPicker.value = false
    pickerDate.value = ''
    pickerTime.value = ''
  }
}

onMounted(() => document.addEventListener('click', onDocClick))
onUnmounted(() => document.removeEventListener('click', onDocClick))

function togglePicker() {
  if (showPicker.value) {
    showPicker.value = false
    pickerDate.value = ''
    pickerTime.value = ''
    return
  }
  const tomorrow = new Date()
  tomorrow.setDate(tomorrow.getDate() + 1)
  pickerDate.value = tomorrow.toISOString().split('T')[0]
  pickerTime.value = '09:00'
  showPicker.value = true
  nextTick(() => {
    pickerRef.value?.querySelector<HTMLInputElement>('.ddl-date')?.focus()
  })
}

function confirmDdl() {
  if (!pickerDate.value || !pickerTime.value) return
  if (!input.value.trim()) {
    showPicker.value = false
    pickerDate.value = ''
    pickerTime.value = ''
    return
  }
  const ddl = new Date(`${pickerDate.value}T${pickerTime.value}`).getTime()
  emit('add', input.value, ddl)
  input.value = ''
  pickerDate.value = ''
  pickerTime.value = ''
  showPicker.value = false
}

function cancelPicker() {
  showPicker.value = false
  pickerDate.value = ''
  pickerTime.value = ''
}

function submit() {
  if (!input.value.trim()) return
  if (pickerDate.value && pickerTime.value) {
    const ddl = new Date(`${pickerDate.value}T${pickerTime.value}`).getTime()
    emit('add', input.value, ddl)
  } else {
    emit('add', input.value)
  }
  input.value = ''
  pickerDate.value = ''
  pickerTime.value = ''
  showPicker.value = false
}
</script>

<template>
  <div class="task-input-area">
    <!-- Floating DDL picker panel -->
    <div v-if="showPicker" ref="pickerRef" class="ddl-picker">
      <div class="ddl-picker-header">
        <span class="ddl-picker-title">截止时间</span>
        <button class="ddl-picker-close" @click="cancelPicker">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
      <div class="ddl-picker-body">
        <input
          v-model="pickerDate"
          type="date"
          class="ddl-input ddl-date"
        />
        <span class="ddl-sep">—</span>
        <input
          v-model="pickerTime"
          type="time"
          class="ddl-input ddl-time"
        />
      </div>
      <div class="ddl-picker-actions">
        <button class="ddl-confirm-btn" @click="confirmDdl">确认</button>
      </div>
    </div>

    <!-- Input row — always stable -->
    <form class="task-input" @submit.prevent="submit">
      <input
        v-model="input"
        type="text"
        placeholder="添加任务，按回车确认..."
        autofocus
      />
      <button
        type="button"
        class="ddl-trigger-btn"
        :class="{ active: showPicker }"
        title="设置截止时间"
        @click="togglePicker"
      >
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
          <line x1="16" y1="2" x2="16" y2="6"/>
          <line x1="8" y1="2" x2="8" y2="6"/>
          <line x1="3" y1="10" x2="21" y2="10"/>
          <path d="M8 14h.01M12 14h.01M16 14h.01M8 18h.01M12 18h.01"/>
        </svg>
        <span>DDL</span>
      </button>
      <button type="submit" class="add-btn">+</button>
    </form>
  </div>
</template>
