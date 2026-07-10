export interface UsedByPod {
  name: string
  status: 'Running' | 'Pending' | 'Failed' | 'Completed'
}

export interface ConfigMapInfo {
  name: string
  namespace: string
  labels: Record<string, string>
  annotations: number
  created: string
  age: string
  resourceVersion: string
  immutable: boolean
  keysCount: number
  size: string
  mountedPods: number
  usedBy: UsedByPod[]
  data: Record<string, string>
}

export interface SecretInfo {
  name: string
  namespace: string
  labels: Record<string, string>
  annotations: number
  type: string
  created: string
  age: string
  resourceVersion: string
  immutable: boolean
  keysCount: number
  size: string
  mountedPods: number
  usedBy: UsedByPod[]
  data: Record<string, string>
}

export const mockConfigMaps: ConfigMapInfo[] = [
  {
    name: 'app-config',
    namespace: 'backend',
    labels: {
      app: 'api-gateway',
      tier: 'backend',
      env: 'production'
    },
    annotations: 2,
    created: 'May 12, 2025, 10:21 AM',
    age: '7d 4h',
    resourceVersion: '4529381',
    immutable: false,
    keysCount: 12,
    size: '24.1 KiB',
    mountedPods: 3,
    usedBy: [
      { name: 'api-gateway-76fbbf8fdd-5k2jh', status: 'Running' },
      { name: 'api-gateway-76fbbf8fdd-xc9pz', status: 'Running' },
      { name: 'api-gateway-76fbbf8fdd-9d7nl', status: 'Running' }
    ],
    data: {
      'database.host': 'postgres-db-service.backend.svc.cluster.local',
      'database.port': '5432',
      'database.name': 'production_gateway_db',
      'api.timeout': '30s',
      'api.max-connections': '1000',
      'api.rate-limit': '100/min',
      'cache.redis.host': 'redis-cache-service.backend.svc.cluster.local',
      'cache.redis.port': '6379',
      'cache.ttl': '3600',
      'log.level': 'info',
      'features.enable-registration': 'true',
      environment: 'production'
    }
  },
  {
    name: 'db-config',
    namespace: 'backend',
    labels: {
      app: 'user-service',
      tier: 'backend'
    },
    annotations: 1,
    created: 'May 12, 2025, 10:25 AM',
    age: '7d 4h',
    resourceVersion: '4529812',
    immutable: false,
    keysCount: 8,
    size: '8.2 KiB',
    mountedPods: 2,
    usedBy: [
      { name: 'user-service-589df4fc7c-xyz98', status: 'Running' },
      { name: 'user-service-589df4fc7c-wuv77', status: 'Running' }
    ],
    data: {
      'db.pool.max-size': '20',
      'db.pool.min-idle': '5',
      'db.pool.idle-timeout-ms': '30000',
      'db.pool.connection-timeout-ms': '20000',
      'db.dialect': 'org.hibernate.dialect.PostgreSQLDialect',
      'db.ddl-auto': 'validate',
      'db.show-sql': 'false',
      'db.format-sql': 'true'
    }
  },
  {
    name: 'nginx-config',
    namespace: 'frontend',
    labels: {
      app: 'web-frontend',
      tier: 'frontend'
    },
    annotations: 0,
    created: 'May 10, 2025, 03:45 PM',
    age: '9d 1h',
    resourceVersion: '4481021',
    immutable: false,
    keysCount: 6,
    size: '6.3 KiB',
    mountedPods: 2,
    usedBy: [
      { name: 'web-frontend-7f4c59b6d8-abc12', status: 'Running' },
      { name: 'web-frontend-7f4c59b6d8-def34', status: 'Running' }
    ],
    data: {
      'nginx.conf': `user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;
events {
    worker_connections 1024;
}`,
      'http.conf': `include /etc/nginx/mime.types;
default_type application/octet-stream;
sendfile on;
keepalive_timeout 65;`,
      'default.conf': `server {
    listen 80;
    server_name localhost;
    location / {
        root /usr/share/nginx/html;
        index index.html index.htm;
        try_files $uri $uri/ /index.html;
    }
}`,
      'security.conf': `add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header X-Content-Type-Options "nosniff" always;`,
      'gzip.conf': `gzip on;
gzip_types text/plain text/css application/json application/javascript;`,
      'proxy.conf': `proxy_set_header Host $host;
proxy_set_header X-Real-IP $remote_addr;`
    }
  },
  {
    name: 'feature-flags',
    namespace: 'backend',
    labels: {
      app: 'platform',
      env: 'production'
    },
    annotations: 1,
    created: 'May 07, 2025, 08:12 AM',
    age: '12d 10h',
    resourceVersion: '4392109',
    immutable: false,
    keysCount: 15,
    size: '12.7 KiB',
    mountedPods: 4,
    usedBy: [
      { name: 'api-gateway-76fbbf8fdd-5k2jh', status: 'Running' },
      { name: 'user-service-589df4fc7c-xyz98', status: 'Running' },
      { name: 'order-service-67bc7d45f9-plk90', status: 'Running' },
      { name: 'payment-service-59dfc6cd78-mnb54', status: 'Running' }
    ],
    data: {
      'new-checkout-flow': 'false',
      'promo-banner-enabled': 'true',
      'ai-recommendations-alpha': 'false',
      'dark-mode-default': 'true',
      'maintenance-mode': 'false',
      'beta-users-list': 'user_1,user_2,user_3',
      'stripe-migration-active': 'true',
      'max-file-upload-mb': '10',
      'support-chat-enabled': 'true',
      'geo-restrictions-active': 'false',
      'new-navbar-v2': 'true',
      'slack-notifications-enabled': 'true',
      'analytics-sample-rate': '0.1',
      'feature-preview-flag': 'false',
      'experimental-scheduler': 'false'
    }
  },
  {
    name: 'logging-config',
    namespace: 'kube-system',
    labels: {
      'k8s-app': 'fluentd',
      component: 'config'
    },
    annotations: 2,
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4102910',
    immutable: false,
    keysCount: 4,
    size: '3.1 KiB',
    mountedPods: 4,
    usedBy: [
      { name: 'fluentd-g2jk9', status: 'Running' },
      { name: 'fluentd-k8sl2', status: 'Running' },
      { name: 'fluentd-m7op3', status: 'Running' },
      { name: 'fluentd-p2qr4', status: 'Running' }
    ],
    data: {
      'fluent.conf': `<match pattern>
  @type stdout
</match>`,
      'match.conf': `<match **>
  @type elasticsearch
  host elasticsearch-logging
  port 9200
  logstash_format true
</match>`,
      'system.conf': `<system>
  log_level info
</system>`,
      'source.conf': `<source>
  @type tail
  path /var/log/containers/*.log
  pos_file /var/log/fluentd-containers.log.pos
  tag kubernetes.*
  format json
</source>`
    }
  },
  {
    name: 'prometheus-config',
    namespace: 'monitoring',
    labels: {
      app: 'prometheus',
      component: 'server'
    },
    annotations: 3,
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4103810',
    immutable: false,
    keysCount: 23,
    size: '45.8 KiB',
    mountedPods: 1,
    usedBy: [{ name: 'prometheus-server-79d84bf4bc-g8w2k', status: 'Running' }],
    data: {
      'prometheus.yml': `global:
  scrape_interval: 15s
  evaluation_interval: 15s
scrape_configs:
  - job_name: 'kubernetes-apiservers'
    kubernetes_sd_configs:
      - role: apiserver`
    }
  },
  {
    name: 'alertmanager-config',
    namespace: 'monitoring',
    labels: {
      app: 'alertmanager',
      component: 'config'
    },
    annotations: 1,
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4103892',
    immutable: false,
    keysCount: 7,
    size: '9.7 KiB',
    mountedPods: 1,
    usedBy: [{ name: 'alertmanager-main-0', status: 'Running' }],
    data: {
      'alertmanager.yml': `global:
  resolve_timeout: 5m
route:
  group_by: ['job']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 12h
  receiver: 'webhook'`
    }
  },
  {
    name: 'grafana-datasources',
    namespace: 'monitoring',
    labels: {
      app: 'grafana',
      component: 'config'
    },
    annotations: 1,
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4104012',
    immutable: false,
    keysCount: 3,
    size: '2.1 KiB',
    mountedPods: 1,
    usedBy: [{ name: 'grafana-84d79df9cc-2j2sl', status: 'Running' }],
    data: {
      'datasources.yaml': `apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus-server
    isDefault: true`
    }
  },
  {
    name: 'kube-proxy',
    namespace: 'kube-system',
    labels: {
      'k8s-app': 'kube-proxy'
    },
    annotations: 1,
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4102120',
    immutable: true,
    keysCount: 2,
    size: '1.6 KiB',
    mountedPods: 6,
    usedBy: [
      { name: 'kube-proxy-2j2l2', status: 'Running' },
      { name: 'kube-proxy-8s2l1', status: 'Running' },
      { name: 'kube-proxy-dfj8l', status: 'Running' },
      { name: 'kube-proxy-sk2l1', status: 'Running' },
      { name: 'kube-proxy-wks82', status: 'Running' },
      { name: 'kube-proxy-zls99', status: 'Running' }
    ],
    data: {
      'config.conf': `apiVersion: kubeproxy.config.k8s.io/v1alpha1
kind: KubeProxyConfiguration
bindAddress: 0.0.0.0
clientConnection:
  acceptContentTypes: ""
  burst: 10
  contentType: application/vnd.kubernetes.protobuf
  kubeconfig: /var/lib/kube-proxy/kubeconfig.conf
mode: "iptables"`,
      'kubeconfig.conf': `apiVersion: v1
kind: Config
clusters:
- cluster:
    certificate-authority: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
    server: https://10.96.0.1:443
  name: default
contexts:
- context:
    cluster: default
    namespace: default
    user: default
  name: default
current-context: default
users:
- name: default
  user:
    tokenFile: /var/run/secrets/kubernetes.io/serviceaccount/token`
    }
  },
  {
    name: 'coredns',
    namespace: 'kube-system',
    labels: {
      'k8s-app': 'kube-dns'
    },
    annotations: 1,
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4102111',
    immutable: false,
    keysCount: 2,
    size: '1.2 KiB',
    mountedPods: 2,
    usedBy: [
      { name: 'coredns-78fcdf6894-gjsks', status: 'Running' },
      { name: 'coredns-78fcdf6894-sjkws', status: 'Running' }
    ],
    data: {
      Corefile: `.:53 {
    errors
    health {
       lameduck 5s
    }
    ready
    kubernetes cluster.local in-addr.arpa ip6.arpa {
       pods insecure
       fallthrough in-addr.arpa ip6.arpa
       ttl 30
    }
    prometheus :9153
    forward . /etc/resolv.conf {
       max_concurrent 1000
    }
    cache 30
    loop
    reload
    loadbalance
}`
    }
  },
  {
    name: 'app-settings',
    namespace: 'backend',
    labels: {
      app: 'order-service',
      tier: 'backend'
    },
    annotations: 1,
    created: 'May 17, 2025, 08:15 AM',
    age: '2d 6h',
    resourceVersion: '4892019',
    immutable: false,
    keysCount: 10,
    size: '14.3 KiB',
    mountedPods: 3,
    usedBy: [
      { name: 'order-service-67bc7d45f9-plk90', status: 'Running' },
      { name: 'order-service-67bc7d45f9-wskd2', status: 'Running' },
      { name: 'order-service-67bc7d45f9-xss92', status: 'Running' }
    ],
    data: {
      'order.processing.threads': '10',
      'order.max-items': '50',
      'order.expiry-minutes': '60',
      'shipping.free-threshold': '100.00',
      'shipping.default-carrier': 'FEDEX',
      'inventory.service.url': 'http://inventory-service.backend.svc.cluster.local:8080',
      'notifications.retry-attempts': '3',
      'notifications.backoff-ms': '2000',
      'payment.service.url': 'http://payment-service.backend.svc.cluster.local:8080',
      'features.enable-express-shipping': 'true'
    }
  },
  {
    name: 'payment-config',
    namespace: 'backend',
    labels: {
      app: 'payment-service',
      tier: 'backend'
    },
    annotations: 1,
    created: 'May 15, 2025, 02:00 PM',
    age: '5d 2h',
    resourceVersion: '4782012',
    immutable: false,
    keysCount: 9,
    size: '11.4 KiB',
    mountedPods: 2,
    usedBy: [
      { name: 'payment-service-59dfc6cd78-mnb54', status: 'Running' },
      { name: 'payment-service-59dfc6cd78-skd88', status: 'Running' }
    ],
    data: {
      'gateway.provider': 'stripe',
      'gateway.timeout-ms': '15000',
      'gateway.retries': '3',
      'currency.default': 'USD',
      'currency.supported': 'USD,EUR,GBP,JPY',
      'invoice.template-path': '/templates/invoice.html',
      'invoice.prefix': 'INV-',
      'security.pci-compliant': 'true',
      'debug.log-payloads': 'false'
    }
  }
]

export const mockSecrets: SecretInfo[] = [
  {
    name: 'db-credentials',
    namespace: 'backend',
    labels: {
      app: 'user-service',
      tier: 'backend'
    },
    annotations: 1,
    type: 'Opaque',
    created: 'May 12, 2025, 10:25 AM',
    age: '7d 4h',
    resourceVersion: '4529813',
    immutable: false,
    keysCount: 3,
    size: '1.2 KiB',
    mountedPods: 2,
    usedBy: [
      { name: 'user-service-589df4fc7c-xyz98', status: 'Running' },
      { name: 'user-service-589df4fc7c-wuv77', status: 'Running' }
    ],
    data: {
      username: 'cGxhdGZvcm1fYWRtaW4=', // platform_admin
      password: 'c3VwZXJfc2VjcmV0X3Bhc3N3b3JkXzIwMjY=', // super_secret_password_2026
      'database-name': 'cHJvZHVjdGlvbl9kYg==' // production_db
    }
  },
  {
    name: 'jwt-secret',
    namespace: 'backend',
    labels: {
      app: 'api-gateway',
      tier: 'backend'
    },
    annotations: 1,
    type: 'Opaque',
    created: 'May 12, 2025, 10:21 AM',
    age: '7d 4h',
    resourceVersion: '4529382',
    immutable: false,
    keysCount: 1,
    size: '256 B',
    mountedPods: 4,
    usedBy: [
      { name: 'api-gateway-76fbbf8fdd-5k2jh', status: 'Running' },
      { name: 'api-gateway-76fbbf8fdd-xc9pz', status: 'Running' },
      { name: 'api-gateway-76fbbf8fdd-9d7nl', status: 'Running' },
      { name: 'user-service-589df4fc7c-xyz98', status: 'Running' }
    ],
    data: {
      'secret-key':
        'ZDI4YTVmYzg1YzE3NDk2N2I2OWI5NmQ3MzkxODlkNDc1MTI1MTcyNjMxZGU4NGRkMWQzNmQ3YWQ5NmExZmU2Zg==' // d28a5fc85c174967b69b96d739189d47512172631de84dd1d36d7ad96a1fe6f
    }
  },
  {
    name: 'tls-cert',
    namespace: 'frontend',
    labels: {
      app: 'web-frontend',
      tier: 'frontend'
    },
    annotations: 1,
    type: 'kubernetes.io/tls',
    created: 'May 10, 2025, 03:45 PM',
    age: '9d 1h',
    resourceVersion: '4481022',
    immutable: false,
    keysCount: 2,
    size: '4.8 KiB',
    mountedPods: 2,
    usedBy: [
      { name: 'web-frontend-7f4c59b6d8-abc12', status: 'Running' },
      { name: 'web-frontend-7f4c59b6d8-def34', status: 'Running' }
    ],
    data: {
      'tls.crt':
        'LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSURkekNDQWx1Z0F3SUJBZ0lVQndkc2d2V1RkZjdWcXJzMnp3M0lSU0F4QmdvckJnRUVBYjR0QWdFRUJRTXcKTVFzd0NRWURWUVFHRXdKQlV6RU9NQXdHQTFVRUNDynamicExampleData...',
      'tls.key':
        'LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JSUV2Z0lCQURBTkJna3Foa2lHOXcwQkFRRUZBQVNDQktnd2dnU2tBZ0VBQW9JQkFRRGVlTmt2M2dyc0RybDAKZXh0cmVtZWx5U2VjcmV0S2V5RGF0YS...'
    }
  },
  {
    name: 'aws-credentials',
    namespace: 'backend',
    labels: {
      app: 'platform',
      env: 'production'
    },
    annotations: 1,
    type: 'Opaque',
    created: 'May 07, 2025, 08:12 AM',
    age: '12d 10h',
    resourceVersion: '4392110',
    immutable: false,
    keysCount: 2,
    size: '850 B',
    mountedPods: 3,
    usedBy: [
      { name: 'api-gateway-76fbbf8fdd-5k2jh', status: 'Running' },
      { name: 'order-service-67bc7d45f9-plk90', status: 'Running' },
      { name: 'payment-service-59dfc6cd78-mnb54', status: 'Running' }
    ],
    data: {
      'aws-access-key-id': 'QUtJQVNJT05GQURNSU5TVEFDSzI2', // AKIASIONFADMINSTACK26
      'aws-secret-access-key': 'OE9NTEs5RjIzU2Q0RnU1ZzdIOGpLMmxNM25PbDRwNVFyc1R1dld4WQ==' // 8OMLK9F23Sd4Fu5Vs7H8jK2lM3nOl4p5QrsTuVWxY
    }
  },
  {
    name: 'fluentd-token',
    namespace: 'kube-system',
    labels: {
      'k8s-app': 'fluentd'
    },
    annotations: 2,
    type: 'kubernetes.io/service-account-token',
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4102911',
    immutable: false,
    keysCount: 3,
    size: '2.4 KiB',
    mountedPods: 4,
    usedBy: [
      { name: 'fluentd-g2jk9', status: 'Running' },
      { name: 'fluentd-k8sl2', status: 'Running' },
      { name: 'fluentd-m7op3', status: 'Running' },
      { name: 'fluentd-p2qr4', status: 'Running' }
    ],
    data: {
      token:
        'ZXlKaGJHY2lPaUpTVXpJMU5pSXNJbXRwWkNJNkZqRnFiSE5oT1dSb2JtSmxkbTlzWTI5MWJtUWlMQ0p1Y21pT2lKb2RIUndPaTh2TDI0aGFXTm9kMmh2WTNBdmRIbHdaV1J5Y21WMElpd2lhM0lpT2lKcllYcGxjbTVsZEdsemRHVnliR2x1WnkxM2JtNWhiSFVpZlEuc2VjdXJldG9rZW5leGFtcGxlZGF0YQ==',
      'ca.crt':
        'LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUN5RENDQWJDZ0F3SUJBZ0lCQURBTkJna3Foa2lHOXcwQkFRc0ZBREFOTVFzd0NRWURWUVFHRXdKQlV6QWUKTURReEZ6QVZCZ05WQkFvTURrTnJZWEJzWlhSdmJpQnpZWFJsTTAxQk1SRXdEd1lEVlFRRERBaDFkV0psY25WdQpkR1Z6TUI0WERURXdNRFV4T0RFeU1USXpORm9YRFRJMU1EVXhPREV5TVRJek5Gb3dEekVOTUFzR0ExVUVBhZ...',
      namespace: 'a3ViZS1zeXN0ZW0=' // kube-system
    }
  },
  {
    name: 'prometheus-token',
    namespace: 'monitoring',
    labels: {
      app: 'prometheus'
    },
    annotations: 1,
    type: 'kubernetes.io/service-account-token',
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4103811',
    immutable: false,
    keysCount: 3,
    size: '2.4 KiB',
    mountedPods: 1,
    usedBy: [{ name: 'prometheus-server-79d84bf4bc-g8w2k', status: 'Running' }],
    data: {
      token:
        'ZXlKaGJHY2lPaUpTVXpJMU5pSXNJbXRwWkNJNkZqRnFiSE5oT1dSb2JtSmxkbTlzWTI5MWJtUWlMQ0p1Y21pT2lKb2RIUndPaTh2TDI0aGFXTm9kMmh2WTNBdmRIbHdaV1J5Y21WMElpd2lhM0lpT2lKcmV5MXpaWEp2WlhKZmRXTnJMbTFsZEdseg==',
      'ca.crt':
        'LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUN5RENDQWJDZ0F3SUJBZ0lCQURBTkJna3Foa2lHOXcwQkFRc0ZBREFOTVFzd0NRWURWUVFHRXdKQlV6QWUKTURReEZ6QVZCZ05WQkFvTURrTnJZWEJzWlhSdmJpQnpZWFJsTTAxQk1SRXdEd1lEVlFRRERBaDFkV0psY25WdQpkR1Z6TUI0WERURXdNRFV4T0RFeU1USXpORm9YRFRJMU1EVXhPREV5TVRJek5Gb3dEekVOTUFzR0ExVUV...',
      namespace: 'bW9uaXRvcmluZw==' // monitoring
    }
  },
  {
    name: 'alertmanager-webhook',
    namespace: 'monitoring',
    labels: {
      app: 'alertmanager'
    },
    annotations: 1,
    type: 'Opaque',
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4103893',
    immutable: false,
    keysCount: 1,
    size: '512 B',
    mountedPods: 1,
    usedBy: [{ name: 'alertmanager-main-0', status: 'Running' }],
    data: {
      'webhook-url':
        'aHR0cHM6Ly9jaGF0Lmdvb2dsZS5jb20vdjEvc3BhY2VzL0FBQUFBMjZzay93ZWJob29rcy9jdXN0b20/a2V5PVNlY3JldEtleSZ0b2tlbj1TZWN1cmVUb2tlbg=='
    }
  },
  {
    name: 'grafana-admin',
    namespace: 'monitoring',
    labels: {
      app: 'grafana'
    },
    annotations: 1,
    type: 'Opaque',
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4104013',
    immutable: false,
    keysCount: 2,
    size: '512 B',
    mountedPods: 1,
    usedBy: [{ name: 'grafana-84d79df9cc-2j2sl', status: 'Running' }],
    data: {
      'admin-user': 'YWRtaW4=', // admin
      'admin-password': 'R3JhZmFuYV9QYXNzd29yZF9TZWN1cmVfMjAyNg==' // Grafana_Password_Secure_2026
    }
  },
  {
    name: 'kube-proxy-token',
    namespace: 'kube-system',
    labels: {
      'k8s-app': 'kube-proxy'
    },
    annotations: 1,
    type: 'kubernetes.io/service-account-token',
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4102121',
    immutable: false,
    keysCount: 3,
    size: '2.4 KiB',
    mountedPods: 6,
    usedBy: [
      { name: 'kube-proxy-2j2l2', status: 'Running' },
      { name: 'kube-proxy-8s2l1', status: 'Running' },
      { name: 'kube-proxy-dfj8l', status: 'Running' },
      { name: 'kube-proxy-sk2l1', status: 'Running' },
      { name: 'kube-proxy-wks82', status: 'Running' },
      { name: 'kube-proxy-zls99', status: 'Running' }
    ],
    data: {
      token:
        'ZXlKaGJHY2lPaUpTVXpJMU5pSXNJbXRwWkNJNkZqRnFiSE5oT1dSb2JtSmxkbTlzWTI5MWJtUWlMQ0p1Y21pT2lKb2RIUndPaTh2TDI0aGFXTm9kMmh2WTNBdmRIbHdaV1J5Y21WMElpd2lhM0lpT2lKcmV5MXpkV0p5YjNocGVtOTFiaTFwYm5ScGJtYw==',
      'ca.crt':
        'LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUN5RENDQWJDZ0F3SUJBZ0lCQURBTkJna3Foa2lHOXcwQkFRc0ZBREFOTVFzd0NRWURWUVFHRXdKQlV6QWUKTURReEZ6QVZCZ05WQkFvTURrTnJZWEJzWlhSdmJpQnpZWFJsTTAxQk1SRXdEd1lEVlFRRERBaDFkV0psY25WdQpkR1Z6TUI0WERURXdNRFV4T0RFeU1USXpORm9YRFRJMU1EVXhPREV5TVRJek5Gb3dEekVOTUFzR0ExVUV...',
      namespace: 'a3ViZS1zeXN0ZW0=' // kube-system
    }
  },
  {
    name: 'coredns-token',
    namespace: 'kube-system',
    labels: {
      'k8s-app': 'kube-dns'
    },
    annotations: 1,
    type: 'kubernetes.io/service-account-token',
    created: 'May 01, 2025, 02:14 AM',
    age: '18d 4h',
    resourceVersion: '4102112',
    immutable: false,
    keysCount: 3,
    size: '2.4 KiB',
    mountedPods: 2,
    usedBy: [
      { name: 'coredns-78fcdf6894-gjsks', status: 'Running' },
      { name: 'coredns-78fcdf6894-sjkws', status: 'Running' }
    ],
    data: {
      token:
        'ZXlKaGJHY2lPaUpTVXpJMU5pSXNJbXRwWkNJNkZqRnFiSE5oT1dSb2JtSmxkbTlzWTI5MWJtUWlMQ0p1Y21pT2lKb2RIUndPaTh2TDI0aGFXTm9kMmh2WTNBdmRIbHdaV1J5Y21WMElpd2lhM0lpT2lKcmV5MXpkV0pyWlhKbWJ5MTNhbTVoYkhV=',
      'ca.crt':
        'LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUN5RENDQWJDZ0F3SUJBZ0lCQURBTkJna3Foa2lHOXcwQkFRc0ZBREFOTVFzd0NRWURWUVFHRXdKQlV6QWUKTURReEZ6QVZCZ05WQkFvTURrTnJZWEJzWlhSdmJpQnpZWFJsTTAxQk1SRXdEd1lEVlFRRERBaDFkV0psY25WdQpkR1Z6TUI0WERURXdNRFV4T0RFeU1USXpORm9YRFRJMU1EVXhPREV5TVRJek5Gb3dEekVOTUFzR0ExVUV...',
      namespace: 'a3ViZS1zeXN0ZW0=' // kube-system
    }
  },
  {
    name: 'stripe-keys',
    namespace: 'backend',
    labels: {
      app: 'payment-service',
      tier: 'backend'
    },
    annotations: 1,
    type: 'Opaque',
    created: 'May 15, 2025, 02:00 PM',
    age: '5d 2h',
    resourceVersion: '4782013',
    immutable: false,
    keysCount: 2,
    size: '640 B',
    mountedPods: 2,
    usedBy: [
      { name: 'payment-service-59dfc6cd78-mnb54', status: 'Running' },
      { name: 'payment-service-59dfc6cd78-skd88', status: 'Running' }
    ],
    data: {
      'publishable-key': 'cGtfdGVzdF81MU15U2VjdXJlUHVibGlzaGFibGVLZXkyMDI2', // pk_test_51MySecurePublishableKey2026
      'secret-key': 'c2tfdGVzdF81MU15U2VjdXJlU2VjcmV0S2V5RGF0YTIwMjY=' // sk_test_51MySecureSecretKeyData2026
    }
  },
  {
    name: 'redis-password',
    namespace: 'backend',
    labels: {
      app: 'api-gateway',
      tier: 'backend'
    },
    annotations: 1,
    type: 'Opaque',
    created: 'May 12, 2025, 10:21 AM',
    age: '7d 4h',
    resourceVersion: '4529383',
    immutable: false,
    keysCount: 1,
    size: '128 B',
    mountedPods: 3,
    usedBy: [
      { name: 'api-gateway-76fbbf8fdd-5k2jh', status: 'Running' },
      { name: 'api-gateway-76fbbf8fdd-xc9pz', status: 'Running' },
      { name: 'api-gateway-76fbbf8fdd-9d7nl', status: 'Running' }
    ],
    data: {
      password: 'Y2hvc2VuLXNlY3VyZS1yZWRpcy1wYXNzd29yZC0yMDI2' // chosen-secure-redis-password-2026
    }
  }
]
