import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import HomeView from '../views/home-view.vue'
import AccountsView from '../views/accounts-view.vue'
import TransactionsView from '../views/transactions-view.vue'
import BudgetView from '../views/budget-view.vue'
import FeaturePlaceholderView from '../views/feature-placeholder-view.vue'

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'home', component: HomeView },
  {
    path: '/accounts',
    name: 'accounts',
    component: AccountsView,
    meta: { title: 'Cuentas', description: 'Tus cuentas y saldos por moneda.' },
  },
  {
    path: '/payees',
    name: 'payees',
    component: FeaturePlaceholderView,
    meta: { title: 'Payees', description: 'Los nombres y alias de comercios y personas se habilitarán cuando exista su directorio.' },
  },
  {
    path: '/projections',
    name: 'projections',
    component: FeaturePlaceholderView,
    meta: { title: 'Proyecciones', description: 'Escenarios futuros con supuestos visibles.' },
  },
  {
    path: '/objectives',
    name: 'objectives',
    component: FeaturePlaceholderView,
    meta: { title: 'Objetivos', description: 'Metas y reservas personales.' },
  },
  {
    path: '/transactions',
    name: 'transactions',
    component: TransactionsView,
    meta: { title: 'Movimientos', description: 'Ingresos, gastos, reembolsos y transferencias.' },
  },
  {
    path: '/budget',
    name: 'budget',
    component: BudgetView,
    meta: { title: 'Presupuesto', description: 'Planificación mensual y asignaciones.' },
  },
  { path: '/:pathMatch(.*)*', redirect: { name: 'home' } },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})
