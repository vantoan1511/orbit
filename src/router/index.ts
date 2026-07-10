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
    }
  ]
})

export default router
