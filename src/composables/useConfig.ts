import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  readTextFile,
  writeTextFile,
  mkdir,
  exists,
  BaseDirectory,
} from '@tauri-apps/plugin-fs'

const DATA_DIR = 'com.desktodo.desk'
const CONFIG_FILE = 'config.json'

const DEFAULT_MODS = ['Control']
const DEFAULT_KEY = '2'

export interface AppConfig {
  shortcut: {
    modifiers: string[]
    key: string
  }
}

// Module-level singleton shared across all useConfig() calls
const config = ref<AppConfig>({
  shortcut: { modifiers: DEFAULT_MODS, key: DEFAULT_KEY },
})

export function useConfig() {
  async function ensureDataDir() {
    try {
      const dirExists = await exists(DATA_DIR, { baseDir: BaseDirectory.AppData })
      if (!dirExists) {
        await mkdir(DATA_DIR, { baseDir: BaseDirectory.AppData, recursive: true })
      }
    } catch (e) {
      // ignore
    }
  }

  async function loadConfig() {
    try {
      await ensureDataDir()
      const filePath = `${DATA_DIR}/${CONFIG_FILE}`
      const fileExists = await exists(filePath, { baseDir: BaseDirectory.AppData })
      if (fileExists) {
        const content = await readTextFile(filePath, { baseDir: BaseDirectory.AppData })
        const parsed = JSON.parse(content)
        config.value = {
          shortcut: {
            modifiers: parsed.shortcut?.modifiers ?? DEFAULT_MODS,
            key: parsed.shortcut?.key ?? DEFAULT_KEY,
          },
        }
      } else {
        config.value = { shortcut: { modifiers: DEFAULT_MODS, key: DEFAULT_KEY } }
      }
    } catch (e) {
      config.value = { shortcut: { modifiers: DEFAULT_MODS, key: DEFAULT_KEY } }
    }
  }

  async function saveConfig() {
    try {
      await ensureDataDir()
      const filePath = `${DATA_DIR}/${CONFIG_FILE}`
      await writeTextFile(filePath, JSON.stringify(config.value, null, 2), {
        baseDir: BaseDirectory.AppData,
      })
    } catch (e) {
      console.error('Failed to save config:', e)
    }
  }

  async function updateShortcut(modifiers: string[], key: string) {
    config.value.shortcut = { modifiers, key }
    await saveConfig()
  }

  async function refreshFromRust() {
    // Get the currently registered shortcut from Rust (in case it was updated)
    try {
      const label = await invoke<string>('get_current_shortcut')
      return label
    } catch {
      return getShortcutLabel()
    }
  }

  function getShortcutLabel(): string {
    const { modifiers, key } = config.value.shortcut
    const parts: string[] = []
    if (modifiers.includes('Control')) parts.push('Ctrl')
    if (modifiers.includes('Option')) parts.push('Option')
    if (modifiers.includes('Shift')) parts.push('Shift')
    if (modifiers.includes('Super')) parts.push('Cmd')
    parts.push(key)
    return parts.join(' + ')
  }

  return {
    config,
    loadConfig,
    saveConfig,
    updateShortcut,
    refreshFromRust,
    getShortcutLabel,
  }
}
