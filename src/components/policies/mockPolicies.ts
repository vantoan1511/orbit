export interface PolicyInfo {
  uid: string
  name: string
  type: string
  scope: string
  namespace: string
  status: 'Enforced' | 'Audit' | 'Disabled'
  mode: string
  violations: number
  lastUpdated: string
  description: string
  rules: string
}

const basePolicies: PolicyInfo[] = [
  {
    uid: 'p1',
    name: 'default-deny-ingress',
    type: 'Network Policy',
    scope: 'Namespaced',
    namespace: 'default',
    status: 'Enforced',
    mode: 'enforce',
    violations: 0,
    lastUpdated: '10m',
    description: 'Denies all ingress traffic by default for the default namespace.',
    rules:
      'kind: NetworkPolicy\napiVersion: networking.k8s.io/v1\nmetadata:\n  name: default-deny-ingress\n  namespace: default\nspec:\n  podSelector: {}\n  policyTypes:\n  - Ingress'
  },
  {
    uid: 'p2',
    name: 'require-resource-requests',
    type: 'Admission Policy',
    scope: 'Cluster',
    namespace: '-',
    status: 'Audit',
    mode: 'audit',
    violations: 12,
    lastUpdated: '1h',
    description: 'Ensures all pods specify CPU and memory resource requests.',
    rules:
      'kind: ValidatingAdmissionPolicy\napiVersion: admissionregistration.k8s.io/v1\nmetadata:\n  name: require-resource-requests\nspec:\n  matchConstraints:\n    resourceRules:\n    - apiGroups: [""]\n      apiVersions: ["v1"]\n      operations: ["CREATE", "UPDATE"]\n      resources: ["pods"]\n  validations:\n  - expression: "has(object.spec.containers.all(c, has(c.resources.requests)))"'
  },
  {
    uid: 'p3',
    name: 'restrict-host-network',
    type: 'Pod Security',
    scope: 'Cluster',
    namespace: '-',
    status: 'Enforced',
    mode: 'enforce',
    violations: 3,
    lastUpdated: '2d',
    description: 'Prevents pods from using the host network namespace.',
    rules:
      'kind: ClusterPolicy\nmetadata:\n  name: restrict-host-network\nspec:\n  rules:\n  - name: validate-host-network\n    validate:\n      message: "Host network is not allowed."\n      pattern:\n        spec:\n          hostNetwork: false'
  },
  {
    uid: 'p4',
    name: 'frontend-allow-api',
    type: 'Network Policy',
    scope: 'Namespaced',
    namespace: 'frontend',
    status: 'Disabled',
    mode: 'warn',
    violations: 0,
    lastUpdated: '5d',
    description: 'Allows frontend pods to communicate with the API gateway.',
    rules:
      'kind: NetworkPolicy\nspec:\n  podSelector:\n    matchLabels:\n      app: frontend\n  egress:\n  - to:\n    - podSelector:\n        matchLabels:\n          app: api-gateway\n    ports:\n    - protocol: TCP\n      port: 8080'
  }
]

const types = ['Network Policy', 'Pod Security', 'Resource Quota', 'RBAC', 'Admission Policy']
const namespaces = ['default', 'backend', 'frontend', 'kube-system']
const statuses: ('Enforced' | 'Audit' | 'Disabled')[] = [
  'Enforced',
  'Enforced',
  'Audit',
  'Disabled'
]
const modes = ['enforce', 'audit', 'warn']

const generatedPolicies: PolicyInfo[] = Array.from({ length: 40 }, (_, i) => {
  const id = i + 5
  const type = types[id % types.length]!
  const isCluster = type === 'Admission Policy' || type === 'RBAC' || id % 3 === 0
  const namespace = isCluster ? '-' : namespaces[id % namespaces.length]!
  const status = statuses[id % statuses.length]!
  const mode =
    status === 'Enforced' ? 'enforce' : status === 'Audit' ? 'audit' : modes[id % modes.length]!

  return {
    uid: `p${id}`,
    name: `policy-${type.toLowerCase().replace(' ', '-')}-${id}`,
    type,
    scope: isCluster ? 'Cluster' : 'Namespaced',
    namespace,
    status,
    mode,
    violations: id % 5 === 0 ? Math.floor(Math.random() * 50) : 0,
    lastUpdated: `${(id % 24) + 1}h`,
    description: `Auto-generated mock policy for ${type}`,
    rules: `kind: ${type.replace(' ', '')}\nmetadata:\n  name: policy-${id}`
  }
})

export const mockPolicies: PolicyInfo[] = [...basePolicies, ...generatedPolicies]
