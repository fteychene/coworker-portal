const STORAGE_KEY = 'theme'
const DEFAULT_THEME = 'corporate'

export const THEMES = [
  { id: 'corporate', label: 'Clair' },
  { id: 'dim',       label: 'Sombre' },
] as const

export type ThemeId = (typeof THEMES)[number]['id']

export function getTheme(): ThemeId {
  return (localStorage.getItem(STORAGE_KEY) as ThemeId) ?? DEFAULT_THEME
}

export function setTheme(theme: ThemeId) {
  localStorage.setItem(STORAGE_KEY, theme)
  document.documentElement.setAttribute('data-theme', theme)
}

export function initTheme() {
  document.documentElement.setAttribute('data-theme', getTheme())
}
