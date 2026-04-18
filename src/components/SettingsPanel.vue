<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useConfig } from '../composables/useConfig'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const { loadConfig, updateShortcut, getShortcutLabel } = useConfig()

const recording = ref(false)
const recordingKey = ref('')
const recordingModifiers = ref<string[]>([])
const pressHint = ref('')
const currentLabel = ref('')

watch(() => props.open, async (isOpen) => {
  if (isOpen) {
    // Reload config fresh every time the panel opens
    await loadConfig()
    currentLabel.value = getShortcutLabel()
    recording.value = false
    recordingKey.value = ''
    recordingModifiers.value = []
    pressHint.value = ''
  }
})

async function startRecording() {
  recording.value = true
  recordingKey.value = ''
  recordingModifiers.value = []
  pressHint.value = 'Press keys...'
  try {
    await invoke('start_recording')
  } catch (e) {
    console.error('start_recording error:', e)
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (!recording.value) return

  e.preventDefault()
  e.stopPropagation()

  const mods: string[] = []
  if (e.ctrlKey) mods.push('Control')
  if (e.altKey) mods.push('Option')
  if (e.shiftKey) mods.push('Shift')
  if (e.metaKey) mods.push('Super')

  const key = e.key

  if (key === 'Control' || key === 'Alt' || key === 'Shift' || key === 'Meta' || key === 'OS') {
    recordingModifiers.value = mods
    pressHint.value = buildHint(mods, '')
    return
  }

  let keyName = key.toUpperCase()
  if (keyName === ' ') keyName = 'SPACE'
  if (keyName === 'ARROWUP') keyName = 'UP'
  if (keyName === 'ARROWDOWN') keyName = 'DOWN'
  if (keyName === 'ARROWLEFT') keyName = 'LEFT'
  if (keyName === 'ARROWRIGHT') keyName = 'RIGHT'
  if (keyName === 'ESCAPE') keyName = 'ESCAPE'
  if (keyName === 'BACKSPACE') keyName = 'BACKSPACE'
  if (keyName === 'DELETE') keyName = 'DELETE'
  if (keyName === 'ENTER') keyName = 'ENTER'
  if (keyName === 'TAB') keyName = 'TAB'

  if (mods.length === 0) {
    pressHint.value = '需要 Ctrl/Option/Cmd + 按键'
    return
  }

  recordingKey.value = keyName
  recordingModifiers.value = mods
  pressHint.value = buildHint(mods, keyName)
}

function buildHint(mods: string[], key: string): string {
  const parts = mods.map(m => {
    if (m === 'Control') return 'Ctrl'
    if (m === 'Option') return 'Option'
    if (m === 'Shift') return 'Shift'
    if (m === 'Super') return 'Cmd'
    return m
  })
  if (key) parts.push(key)
  return parts.length > 0 ? parts.join(' + ') : 'Press keys...'
}

async function confirmRecording() {
  if (!recordingKey.value || recordingModifiers.value.length === 0) return

  try {
    const newLabel = await invoke<string>('update_shortcut', {
      modifiers: recordingModifiers.value,
      key: recordingKey.value,
    })
    await updateShortcut(recordingModifiers.value, recordingKey.value)
    currentLabel.value = newLabel
    await invoke('stop_recording')
  } catch (e) {
    console.error('Failed to update shortcut:', e)
  } finally {
    recording.value = false
  }
}

async function cancelRecording() {
  recording.value = false
  try {
    await invoke('stop_recording')
  } catch (e) {
    console.error('stop_recording error:', e)
  }
}

async function handleClose() {
  if (recording.value) {
    await cancelRecording()
  }
  emit('close')
}

function getDisplay(): string {
  if (recording.value) {
    return pressHint.value || 'Press keys...'
  }
  return currentLabel.value || getShortcutLabel()
}
</script>

<template>
  <Transition name="settings">
    <div v-if="open" class="settings-overlay" @click.self="handleClose">
      <div class="settings-panel">
        <div class="settings-header">
          <h2>设置</h2>
          <button class="close-btn" @click="handleClose">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <div class="settings-section">
          <label class="section-label">全局快捷键</label>
          <p class="section-hint">
            {{ recording ? '按下想要的快捷键，然后点击保存' : '点击录制，然后按下快捷键' }}
          </p>

          <div
            class="shortcut-recorder"
            :class="{ recording }"
            tabindex="0"
            @keydown="handleKeyDown"
          >
            <span class="shortcut-display">{{ getDisplay() }}</span>
            <button v-if="!recording" class="record-btn" @click="startRecording">
              录制
            </button>
            <div v-else class="recording-actions">
              <button class="confirm-btn" :disabled="!recordingKey" @click="confirmRecording">保存</button>
              <button class="cancel-btn" @click="cancelRecording">取消</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>
