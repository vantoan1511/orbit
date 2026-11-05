import type { ServiceInfo } from '@/types/kubernetes'

/**
 * Resolves an external HTTP/HTTPS URL for a Kubernetes service if an external IP/hostname exists.
 * Returns null if the service has no valid external IP or hostname.
 */
export function resolveServiceExternalUrl(service: ServiceInfo): string | null {
  const rawIp = service.externalIP?.trim()
  if (!rawIp || rawIp === '-') {
    return null
  }

  // If already an absolute URL scheme
  if (/^https?:\/\//i.test(rawIp)) {
    return rawIp
  }

  // Parse ports: prioritize port 443, then port 80, then the first port
  const detectedPorts: number[] = []

  if (service.portsList && service.portsList.length > 0) {
    for (const p of service.portsList) {
      if (typeof p.port === 'number') {
        detectedPorts.push(p.port)
      }
    }
  }

  if (detectedPorts.length === 0 && service.ports) {
    // e.g. "80/TCP", "443:30443/TCP, 80:30080/TCP"
    const matches = service.ports.matchAll(/(\d+)(?::\d+)?\/(?:TCP|UDP)/gi)
    for (const match of matches) {
      const rawPort = match[1]
      if (rawPort) {
        const portNum = Number.parseInt(rawPort, 10)
        if (!Number.isNaN(portNum)) {
          detectedPorts.push(portNum)
        }
      }
    }
  }

  const has443 = detectedPorts.includes(443)
  if (has443) {
    return `https://${rawIp}`
  }

  const primaryPort = detectedPorts.length > 0 ? detectedPorts[0] : 80
  if (primaryPort === 80) {
    return `http://${rawIp}`
  }

  return `http://${rawIp}:${primaryPort}`
}
