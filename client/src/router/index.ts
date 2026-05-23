// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

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
      path: '/filter',
      name: 'filter',
      component: FilterPage,
      beforeEnter: (to) => {
        const valid = FILTER_COLUMNS.some(
          (column) => typeof to.query[column] === 'string' && to.query[column]
        )
        if (!valid) return { name: 'overview' }
      }
    },
    {
      path: '/:column/:name',
      redirect: (to) => {
        const column = String(to.params.column)
        const name = String(to.params.name)
        return { name: 'filter', query: { [column]: name } }
      }
    }
  ]
})

export default router
