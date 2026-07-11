import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '../views/DashboardView.vue'
import NodesView from '../views/NodesView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView
    },
    {
      path: '/nodes',
      name: 'nodes',
      component: NodesView
    },
    {
      path: '/workloads',
      name: 'workloads',
      component: () => import('../views/WorkloadsView.vue')
    },
    {
      path: '/pods',
      name: 'pods',
      component: () => import('../views/PodsView.vue')
    },
    {
      path: '/services',
      name: 'services',
      component: () => import('../views/ServicesView.vue')
    },
    {
      path: '/config',
      name: 'configmaps-secrets',
      component: () => import('../views/ConfigView.vue')
    },
    {
      path: '/storage',
      name: 'storage',
      component: () => import('../views/StorageView.vue')
    },
    {
      path: '/namespaces',
      name: 'namespaces',
      component: () => import('../views/NamespacesView.vue')
    },
    {
      path: '/events',
      name: 'events',
      component: () => import('../views/EventsView.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue')
    },
    {
      path: '/policies',
      name: 'policies',
      component: () => import('../views/PoliciesView.vue')
    }
  ]
})

export default router
