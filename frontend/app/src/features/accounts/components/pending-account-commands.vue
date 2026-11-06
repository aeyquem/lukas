<script setup lang="ts">
import { usePendingAccountCommands } from '../composables/use-pending-account-commands'
const props = defineProps<{
  refreshToken: number
}>()
const { pageError, successMessage, isSaving, pendingAccountWrites, pendingError, dismissedPendingKeys, retryAccountWrite, dismissAccountWrite, pendingWriteLabel, } = usePendingAccountCommands(() => props.refreshToken)
</script>
<template>
<div>
  <p v-if="pageError" class="notice error-notice" role="alert">{{ pageError }}</p>
  <p v-if="successMessage" class="notice success-notice" role="status">{{ successMessage }}</p>    <p v-if="pendingError" class="notice error-notice" role="alert">No se pudieron revisar las operaciones guardadas: {{ pendingError }}</p>
  <section v-if="pendingAccountWrites.length > 0" class="pending-commands" aria-labelledby="pending-commands-title">
    <h2 id="pending-commands-title">Operaciones por confirmar</h2>
    <p class="help-copy">Algunas operaciones todavía no tienen un resultado confirmado. Puedes reintentarlas de forma segura o cerrar el aviso; seguirán disponibles hasta que se confirme el resultado.</p>
    <ul>
      <li v-for="command in pendingAccountWrites" :key="command.idempotencyKey" class="pending-command-row">
        <div>
          <strong>{{ pendingWriteLabel(command) }}</strong>
          <p v-if="dismissedPendingKeys.has(command.idempotencyKey)" class="muted">Aviso oculto; el comando sigue guardado y disponible aquí.</p>
          <p v-else class="muted">El resultado todavía no se confirmó.</p>
        </div>
        <div class="pending-actions">
          <button type="button" class="secondary-button" :disabled="isSaving" @click="retryAccountWrite(command)">Reintentar</button>
          <button v-if="!dismissedPendingKeys.has(command.idempotencyKey)" type="button" class="text-button" @click="dismissAccountWrite(command)">Ocultar aviso</button>
        </div>
      </li>
    </ul>
  </section>
</div>
</template>
<style scoped>
h2,
p {
  margin-top: 0;
}

h2 {
  margin-bottom: 0;
  font-size: 22px;
  letter-spacing: -0.035em;
}

.help-copy,
.muted {
  color: var(--muted);
  line-height: 1.5;
}

.help-copy {
  margin-bottom: 0;
  font-size: 12px;
}

.secondary-button {
  display: inline-flex;
  min-height: 44px;
  align-items: center;
  justify-content: center;
  padding: 10px 16px;
  border: 1px solid transparent;
  border-radius: var(--control-radius);
  font: inherit;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
}

.secondary-button {
  border-color: var(--border-strong);
  color: var(--accent);
  background: var(--surface-soft);
}

button:disabled {
  opacity: 0.6;
  cursor: wait;
}

.text-button {
  border: 0;
  padding: 7px 4px;
  color: var(--accent);
  background: transparent;
  font: inherit;
  font-size: 12px;
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 3px;
  cursor: pointer;
}

.notice {
  margin: 0 0 20px;
  padding: 12px 14px;
  border-radius: var(--control-radius);
  font-size: 14px;
}

.error-notice {
  color: var(--danger);
  background: var(--danger-bg);
}

.success-notice {
  color: var(--accent-hover);
  background: var(--success-bg);
}
</style>
