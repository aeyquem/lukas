import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import AccountsView from '../views/AccountsView.vue'
import TransactionsView from '../views/TransactionsView.vue'
import BudgetView from '../views/BudgetView.vue'

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'home', component: HomeView },
  {
    path: '/accounts',
    name: 'accounts',
    component: AccountsView,
    meta: { title: 'Cuentas', description: 'Tus cuentas y saldos por moneda.' },
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
