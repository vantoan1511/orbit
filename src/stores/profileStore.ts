import { kubernetesService } from '@/services/kubernetesService'
import type { UserProfileInfo } from '@/types/profile'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useProfileStore = defineStore('profile', () => {
  const isDrawerOpen = ref(false)
  const profile = ref<UserProfileInfo | null>(null)
  const isLoading = ref(false)

  function setProfile(data: UserProfileInfo) {
    profile.value = data
    isLoading.value = false
  }

  async function fetchProfile() {
    isLoading.value = true
    try {
      await kubernetesService.getUserProfile()
    } catch (e) {
      console.error('Failed to fetch user profile:', e)
      isLoading.value = false
    }
  }

  function toggleDrawer() {
    isDrawerOpen.value = !isDrawerOpen.value
    if (isDrawerOpen.value) {
      fetchProfile()
    }
  }

  function openDrawer() {
    isDrawerOpen.value = true
    fetchProfile()
  }

  function closeDrawer() {
    isDrawerOpen.value = false
  }

  return {
    isDrawerOpen,
    profile,
    isLoading,
    setProfile,
    fetchProfile,
    toggleDrawer,
    openDrawer,
    closeDrawer
  }
})
