import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { currentPeriodInSantiago } from '../components/budget/budget-model'

// The URL owns explicit selection; the clock only supplies the current-month fallback.
export function useDashboardPeriod() {
  const route = useRoute()
  const now = ref(new Date())
  let clock: ReturnType<typeof setInterval> | undefined
  onMounted(() => { clock = setInterval(() => { now.value = new Date() }, 60_000) })
  onUnmounted(() => clearInterval(clock))
  return computed(() => {
    const period = route.query.dashboardPeriod
    return typeof period === 'string' && /^\d{4}-(0[1-9]|1[0-2])$/.test(period)
      ? period : currentPeriodInSantiago(now.value)
  })
}
