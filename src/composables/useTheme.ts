import { ref } from 'vue'

// Shared state so it remains consistent if used in multiple components
const isDark = ref(true)

export function useTheme() {
  const toggleTheme = () => {
    isDark.value = !isDark.value
    const html = document.documentElement
    html.classList.toggle('my-app-dark', isDark.value)
    html.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  }

  return {
    isDark,
    toggleTheme
  }
}
