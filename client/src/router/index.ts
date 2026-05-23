import { FILTER_COLUMNS } from '@/api/types'
import FilterPage from '@/pages/FilterPage.vue'
import OverviewPage from '@/pages/OverviewPage.vue'
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'overview',
      component: OverviewPage
    },
    {
      path: '/:column/:name',
      name: 'filter',
      component: FilterPage,
      beforeEnter: (to) => {
        const col = to.params.column as string
        if (!FILTER_COLUMNS.includes(col as never)) {
          return { name: 'overview' }
        }
      }
    }
  ]
})

export default router
