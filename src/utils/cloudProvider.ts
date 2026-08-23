export interface CloudProviderInfo {
  provider: string
  platform: string
}

interface ProviderRule {
  patterns: string[]
  provider: string
  platform: string
}

const PROVIDER_RULES: readonly ProviderRule[] = [
  { patterns: ['eks.amazonaws.com', 'aws'], provider: 'AWS', platform: 'EKS' },
  { patterns: ['google.com', 'gke'], provider: 'GCP', platform: 'GKE' },
  { patterns: ['azure', 'aks'], provider: 'Azure', platform: 'AKS' },
  { patterns: ['doks.digitalocean.com'], provider: 'DigitalOcean', platform: 'DOKS' },
  { patterns: ['k8s.scaleway.com'], provider: 'Scaleway', platform: 'Kapsule' },
  { patterns: ['lke.linode.com', 'linode'], provider: 'Linode', platform: 'LKE' },
  { patterns: ['vke.vultr.com', 'vultr'], provider: 'Vultr', platform: 'VKE' },
  { patterns: ['civo.com', 'civo'], provider: 'Civo', platform: 'Civo' },
  { patterns: ['oke.oraclecloud.com', 'oraclecloud'], provider: 'Oracle', platform: 'OKE' },
  { patterns: ['minikube'], provider: 'Minikube', platform: 'Local' },
  { patterns: ['k3d'], provider: 'K3d', platform: 'Local' },
  { patterns: ['k3s'], provider: 'K3s', platform: 'Local' },
  { patterns: ['k0s'], provider: 'k0s', platform: 'Local' },
  { patterns: ['microk8s'], provider: 'MicroK8s', platform: 'Local' },
  { patterns: ['docker-desktop'], provider: 'Docker', platform: 'Docker Desktop' },
  { patterns: ['kind.x-k8s.io', 'kind'], provider: 'Kind', platform: 'Local' },
  { patterns: ['talos.dev'], provider: 'Talos', platform: 'Custom' },
  { patterns: ['rancher-desktop'], provider: 'Rancher', platform: 'Rancher Desktop' },
  { patterns: ['colima'], provider: 'Colima', platform: 'Local' },
  { patterns: ['openshift'], provider: 'Red Hat', platform: 'OpenShift' }
]

export const DEFAULT_CLOUD_PROVIDER: CloudProviderInfo = {
  provider: 'Local',
  platform: 'Custom'
}

/**
 * Detects cloud provider and platform based on Kubernetes node labels.
 */
export function detectCloudProvider(labels: string[] | undefined | null): CloudProviderInfo {
  if (!labels || labels.length === 0) {
    return DEFAULT_CLOUD_PROVIDER
  }

  for (const label of labels) {
    const l = label.toLowerCase()
    const match = PROVIDER_RULES.find((rule) =>
      rule.patterns.some((pattern) => l.includes(pattern))
    )
    if (match) {
      return { provider: match.provider, platform: match.platform }
    }
  }

  return DEFAULT_CLOUD_PROVIDER
}
