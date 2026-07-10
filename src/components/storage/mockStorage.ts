export interface PersistentVolumeInfo {
  name: string
  capacity: string
  accessMode: string
  reclaimPolicy: string
  status: 'Bound' | 'Available' | 'Released' | 'Failed'
  storageClass: string
  age: string
  volumeMode: 'Filesystem' | 'Block'
  reason?: string
}

export interface PersistentVolumeClaimInfo {
  name: string
  namespace: string
  status: 'Bound' | 'Lost' | 'Pending'
  volume: string
  capacity: string
  accessMode: string
  storageClass: string
  age: string
}

export interface StorageClassInfo {
  name: string
  provisioner: string
  reclaimPolicy: string
  volumeBindingMode: string
  allowVolumeExpansion: boolean
  age: string
}

export interface StorageMetrics {
  totalCapacity: string
  usedCapacity: string
  availableCapacity: string
  totalPVs: number
  totalPVCs: number
  totalStorageClasses: number
}

export const mockStorageMetrics: StorageMetrics = {
  totalCapacity: '2.5 TiB',
  usedCapacity: '1.8 TiB',
  availableCapacity: '700 GiB',
  totalPVs: 24,
  totalPVCs: 72,
  totalStorageClasses: 6
}

export const mockPVs: PersistentVolumeInfo[] = [
  {
    name: 'pvc-bf482d8c-c69c-4861-ac07-ef0218b0c950',
    capacity: '100 GiB',
    accessMode: 'ReadWriteOnce',
    reclaimPolicy: 'Delete',
    status: 'Bound',
    storageClass: 'gp3-sc',
    age: '12d',
    volumeMode: 'Filesystem'
  },
  {
    name: 'pvc-ae14b98c-02cf-4b71-9b18-ff3510cbb112',
    capacity: '500 GiB',
    accessMode: 'ReadWriteOnce',
    reclaimPolicy: 'Retain',
    status: 'Bound',
    storageClass: 'efs-sc',
    age: '24d',
    volumeMode: 'Filesystem'
  },
  {
    name: 'pvc-c951ba92-f281-42e1-88fc-cd33bf9031c2',
    capacity: '20 GiB',
    accessMode: 'ReadWriteOnce',
    reclaimPolicy: 'Delete',
    status: 'Bound',
    storageClass: 'gp2-sc',
    age: '3d 4h',
    volumeMode: 'Filesystem'
  },
  {
    name: 'pv-local-ssd-node1',
    capacity: '375 GiB',
    accessMode: 'ReadWriteOnce',
    reclaimPolicy: 'Retain',
    status: 'Available',
    storageClass: 'local-storage',
    age: '90d',
    volumeMode: 'Block'
  },
  {
    name: 'pv-local-ssd-node2',
    capacity: '375 GiB',
    accessMode: 'ReadWriteOnce',
    reclaimPolicy: 'Retain',
    status: 'Released',
    storageClass: 'local-storage',
    age: '90d',
    volumeMode: 'Block'
  },
  {
    name: 'pvc-e82a93b4-10cc-4bf1-82aa-5b23d9b01cc1',
    capacity: '1000 GiB',
    accessMode: 'ReadWriteMany',
    reclaimPolicy: 'Delete',
    status: 'Bound',
    storageClass: 'efs-sc',
    age: '18d',
    volumeMode: 'Filesystem'
  },
  {
    name: 'pvc-f712ac9d-d128-4e89-9a1b-103bc29c0df2',
    capacity: '50 GiB',
    accessMode: 'ReadWriteOnce',
    reclaimPolicy: 'Delete',
    status: 'Failed',
    storageClass: 'gp3-sc',
    age: '2h15m',
    volumeMode: 'Filesystem',
    reason: 'Failed to attach volume: volume-id not found'
  },
  {
    name: 'pvc-89bca1d2-0941-456c-8abf-40cb12bb01ad',
    capacity: '200 GiB',
    accessMode: 'ReadWriteOnce',
    reclaimPolicy: 'Delete',
    status: 'Bound',
    storageClass: 'gp3-sc',
    age: '5d 12h',
    volumeMode: 'Filesystem'
  }
]

export const mockPVCs: PersistentVolumeClaimInfo[] = [
  {
    name: 'postgres-data-db-0',
    namespace: 'backend',
    status: 'Bound',
    volume: 'pvc-bf482d8c-c69c-4861-ac07-ef0218b0c950',
    capacity: '100 GiB',
    accessMode: 'ReadWriteOnce',
    storageClass: 'gp3-sc',
    age: '12d'
  },
  {
    name: 'shared-uploads',
    namespace: 'frontend',
    status: 'Bound',
    volume: 'pvc-ae14b98c-02cf-4b71-9b18-ff3510cbb112',
    capacity: '500 GiB',
    accessMode: 'ReadWriteOnce',
    storageClass: 'efs-sc',
    age: '24d'
  },
  {
    name: 'redis-state-redis-master-0',
    namespace: 'backend',
    status: 'Bound',
    volume: 'pvc-c951ba92-f281-42e1-88fc-cd33bf9031c2',
    capacity: '20 GiB',
    accessMode: 'ReadWriteOnce',
    storageClass: 'gp2-sc',
    age: '3d 4h'
  },
  {
    name: 'app-logs-collector',
    namespace: 'logging',
    status: 'Bound',
    volume: 'pvc-e82a93b4-10cc-4bf1-82aa-5b23d9b01cc1',
    capacity: '1000 GiB',
    accessMode: 'ReadWriteMany',
    storageClass: 'efs-sc',
    age: '18d'
  },
  {
    name: 'prometheus-db-prometheus-server-0',
    namespace: 'monitoring',
    status: 'Pending',
    volume: '',
    capacity: '50 GiB',
    accessMode: 'ReadWriteOnce',
    storageClass: 'gp3-sc',
    age: '2h15m'
  },
  {
    name: 'mysql-data-storage-0',
    namespace: 'backend',
    status: 'Bound',
    volume: 'pvc-89bca1d2-0941-456c-8abf-40cb12bb01ad',
    capacity: '200 GiB',
    accessMode: 'ReadWriteOnce',
    storageClass: 'gp3-sc',
    age: '5d 12h'
  }
]

export const mockStorageClasses: StorageClassInfo[] = [
  {
    name: 'gp3-sc',
    provisioner: 'ebs.csi.aws.com',
    reclaimPolicy: 'Delete',
    volumeBindingMode: 'WaitForFirstConsumer',
    allowVolumeExpansion: true,
    age: '120d'
  },
  {
    name: 'efs-sc',
    provisioner: 'efs.csi.aws.com',
    reclaimPolicy: 'Retain',
    volumeBindingMode: 'Immediate',
    allowVolumeExpansion: false,
    age: '120d'
  },
  {
    name: 'gp2-sc',
    provisioner: 'kubernetes.io/aws-ebs',
    reclaimPolicy: 'Delete',
    volumeBindingMode: 'Immediate',
    allowVolumeExpansion: true,
    age: '365d'
  },
  {
    name: 'local-storage',
    provisioner: 'kubernetes.io/no-provisioner',
    reclaimPolicy: 'Retain',
    volumeBindingMode: 'WaitForFirstConsumer',
    allowVolumeExpansion: false,
    age: '90d'
  },
  {
    name: 'slow-sc',
    provisioner: 'kubernetes.io/aws-ebs',
    reclaimPolicy: 'Delete',
    volumeBindingMode: 'Immediate',
    allowVolumeExpansion: true,
    age: '365d'
  },
  {
    name: 'standard',
    provisioner: 'kubernetes.io/aws-ebs',
    reclaimPolicy: 'Delete',
    volumeBindingMode: 'Immediate',
    allowVolumeExpansion: true,
    age: '365d'
  }
]
