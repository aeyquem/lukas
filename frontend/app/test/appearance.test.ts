import { createPinia } from 'pinia'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { themes, useAppearanceStore } from '../src/app/theme-store'

afterEach(() => vi.unstubAllGlobals())

function browser(saved: string | null = null) {
  const dataset: Record<string, string> = {}
  const storage = { getItem: vi.fn(() => saved), setItem: vi.fn() }
  vi.stubGlobal('document', { documentElement: { dataset } })
  vi.stubGlobal('localStorage', storage)
  return { dataset, storage }
}

describe('appearance preferences', () => {
  it.each(themes)('restores and persists $id using the existing storage key', ({ id }) => {
    const { dataset, storage } = browser(id)
    const store = useAppearanceStore(createPinia())
    store.initializeTheme()
    expect(store.activeTheme).toBe(id)
    expect(dataset.theme).toBe(id)
    store.setTheme(id)
    expect(storage.setItem).toHaveBeenCalledWith('lukas.appearance.theme', id)
  })

  it('ignores unknown themes and remains usable with restricted storage', () => {
    const { dataset, storage } = browser('unknown')
    const store = useAppearanceStore(createPinia())
    store.initializeTheme()
    expect(store.activeTheme).toBe('forest')
    storage.setItem.mockImplementation(() => { throw new Error('restricted') })
    store.setTheme('nord')
    expect(dataset.theme).toBe('nord')
    expect(store.themeNotice).toBe('El tema se aplicó solo para esta sesión.')
    store.setTheme('unknown')
    expect(store.activeTheme).toBe('nord')
    storage.setItem.mockReset()
    store.setTheme('lagoon')
    expect(store.themeNotice).toBe('')
  })

  it('applies the default before mount when storage reads fail', () => {
    const { dataset, storage } = browser()
    storage.getItem.mockImplementation(() => { throw new Error('restricted') })
    const store = useAppearanceStore(createPinia())
    store.initializeTheme()
    expect(dataset.theme).toBe('forest')
  })
})
