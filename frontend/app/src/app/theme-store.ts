import { ref } from 'vue'
import { defineStore } from 'pinia'

export const themes = [
  { id: 'forest', label: 'Bosque' },
  { id: 'lagoon', label: 'Laguna' },
  { id: 'nord', label: 'Nord · Noche' },
] as const
export type ThemeId = typeof themes[number]['id']
const storageKey = 'lukas.appearance.theme'
export const useAppearanceStore = defineStore('appearance', () => {
  const activeTheme = ref<ThemeId>('forest')
  const themeNotice = ref('')

  function isTheme(value: unknown): value is ThemeId {
    return themes.some((theme) => theme.id === value)
  }

  function initializeTheme(): void {
    try {
      const saved = localStorage.getItem(storageKey)
      if (isTheme(saved)) activeTheme.value = saved
    } catch {
      // Appearance remains usable when browser storage is restricted.
    }
    document.documentElement.dataset.theme = activeTheme.value
  }

  function setTheme(value: string): void {
    if (!isTheme(value)) return
    activeTheme.value = value
    document.documentElement.dataset.theme = value
    themeNotice.value = ''
    try {
      localStorage.setItem(storageKey, value)
    } catch {
      themeNotice.value = 'El tema se aplicó solo para esta sesión.'
    }
  }

  return { activeTheme, themeNotice, initializeTheme, setTheme }
})
