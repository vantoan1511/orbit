import ConfigView from '@/views/ConfigView.vue'
import DashboardView from '@/views/DashboardView.vue'
import EditView from '@/views/EditView.vue'
import EventsView from '@/views/EventsView.vue'
import LogsView from '@/views/LogsView.vue'
import NamespacesView from '@/views/NamespacesView.vue'
import NodesView from '@/views/NodesView.vue'
import PoliciesView from '@/views/PoliciesView.vue'
import NetworkView from '@/views/NetworkView.vue'
import SettingsView from '@/views/SettingsView.vue'
import StorageView from '@/views/StorageView.vue'
import WorkloadsView from '@/views/WorkloadsView.vue'
import { createRouter, createWebHistory } from 'vue-router'
import WelcomeView from '../views/WelcomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView
    },
    {
      path: '/welcome',
      name: 'welcome',
      component: WelcomeView
    },
    {
      path: '/nodes',
      name: 'nodes',
      component: NodesView
    },
    {
      path: '/workloads',
      name: 'workloads',
      component: WorkloadsView
    },
    {
      path: '/workloads/:kind/:namespace/:name/edit',
      name: 'edit-workload',
      component: EditView,
      props: true
    },
    {
      path: '/pods',
      redirect: '/workloads?tab=pods'
    },
    {
      path: '/network',
      name: 'network',
      component: NetworkView
    },
    {
      path: '/config',
      name: 'configmaps-secrets',
      component: ConfigView
    },
    {
      path: '/storage',
      name: 'storage',
      component: StorageView
    },
    {
      path: '/namespaces',
      name: 'namespaces',
      component: NamespacesView
    },
    {
      path: '/events',
      name: 'events',
      component: EventsView
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView
    },
    {
      path: '/policies',
      name: 'policies',
      component: PoliciesView
    },
    {
      path: '/logs',
      name: 'logs',
      component: LogsView
    }
  ]
})

export default router
