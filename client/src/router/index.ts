// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

import Error404Page from '@/pages/Error404Page.vue'
import FilterPage from '@/pages/FilterPage.vue'
import OverviewPage from '@/pages/OverviewPage.vue'
import { filtersFromRoute } from '@/utils/filters'
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
        if (!filtersFromRoute(to).length) return { name: 'overview' }
      }
    },
    {
      path: '/:pathMatch(.*)*',
      name: '404',
      component: Error404Page
    }
  ]
})

export default router
