export interface UserProfileInfo {
  activeContext: string | null
  userName: string | null
  authType: string
  clusterName: string | null
  serverUrl: string | null
  kubeconfigPaths: string[]
  k8sVersion: string | null
}
