export type PolicyCategoryFilter =
  'overview' | 'network' | 'pod-security' | 'resource' | 'admission'

export function getFilterOptionsForCategory(category: PolicyCategoryFilter): string[] {
  switch (category) {
    case 'overview':
      return [
        'All Types',
        'Network Policy',
        'Pod Security',
        'Resource Quota',
        'Limit Range',
        'Admission Policy'
      ]
    case 'resource':
      return ['All Resource Policies', 'Resource Quotas', 'Limit Ranges']
    case 'admission':
      return ['All Webhooks', 'Validating Webhooks', 'Mutating Webhooks']
    case 'network':
    case 'pod-security':
    default:
      return []
  }
}

export function filterPolicies<T extends { type: string; kind?: string }>(
  policies: T[],
  category: PolicyCategoryFilter,
  selectedType: string
): T[] {
  return policies.filter((p) => {
    switch (category) {
      case 'network':
        return p.type === 'Network Policy'
      case 'pod-security':
        return p.type === 'Pod Security'
      case 'resource':
        if (p.type !== 'Resource Quota' && p.type !== 'Limit Range') {
          return false
        }
        if (selectedType === 'Resource Quotas' || selectedType === 'Resource Quota') {
          return p.type === 'Resource Quota'
        }
        if (selectedType === 'Limit Ranges' || selectedType === 'Limit Range') {
          return p.type === 'Limit Range'
        }
        return true
      case 'admission':
        if (p.type !== 'Admission Policy') {
          return false
        }
        if (selectedType === 'Validating Webhooks') {
          return p.kind === 'ValidatingWebhookConfiguration'
        }
        if (selectedType === 'Mutating Webhooks') {
          return p.kind === 'MutatingWebhookConfiguration'
        }
        return true
      case 'overview':
      default:
        if (selectedType !== 'All Types' && p.type !== selectedType) {
          return false
        }
        return true
    }
  })
}
