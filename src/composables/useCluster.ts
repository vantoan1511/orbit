import { kubernetesService } from '@/services/kubernetesService'
import { os } from '@/services/nativeService'
import { useToast } from 'primevue'

export function useCluster() {
  const toast = useToast()

  const handleAddCluster = async () => {
    try {
      const selectedFiles = await os.showOpenDialog('Select Kubeconfig File')
      if (selectedFiles?.[0]) {
        await kubernetesService.addCluster(selectedFiles[0])
      }
    } catch (error) {
      console.error('Failed to add cluster:', error)
      toast.add({
        severity: 'error',
        summary: 'Error',
        detail: 'Failed to add cluster.',
        life: 3000
      })
    }
  }

  return {
    handleAddCluster
  }
}
