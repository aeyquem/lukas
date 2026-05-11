<script setup lang="ts">
import { useQuery, useQueryCache } from "@pinia/colada";
import { computed, reactive, ref, watch } from "vue";

import {
  getBudgetTree,
  getMovements,
  getTags,
  type AccountDto,
  type AccountEffect,
  type CreateManualTransactionRequest,
  type TagDto,
  type TransactionKind,
  type UpdateManualTransactionRequest,
} from "@/common/api/ledger";
import {
  INCOME_ENVELOPE_ID,
  applyBudgetTree,
  useBudgetTree,
} from "@/features/budget/model/budgetTree";
import BaseAlert from "@/common/components/base/BaseAlert.vue";
import BaseBadge from "@/common/components/base/BaseBadge.vue";
import BaseButton from "@/common/components/base/BaseButton.vue";
import BaseCard from "@/common/components/base/BaseCard.vue";
import BaseCheckbox from "@/common/components/base/BaseCheckbox.vue";
import BaseField from "@/common/components/base/BaseField.vue";
import BaseMultiSelect from "@/common/components/base/BaseMultiSelect.vue";
import BaseSelect from "@/common/components/base/BaseSelect.vue";
import type { BaseSelectOption } from "@/common/components/base/BaseSelect.vue";
import BaseTextarea from "@/common/components/base/BaseTextarea.vue";
import BaseTextInput from "@/common/components/base/BaseTextInput.vue";
import LedgerMovementsTable from "@/features/ledger/components/LedgerMovementsTable.vue";
import type { LedgerMovement } from "@/features/ledger/model/movements";

const props = defineProps<{
  accounts: AccountDto[];
  createError?: string;
  createTransaction: (request: CreateManualTransactionRequest) => Promise<void>;
  hasAccountsLoadError: boolean;
  isCreating: boolean;
  isUpdating: boolean;
  period: string;
  updateError?: string;
  updateTransaction: (request: UpdateManualTransactionRequest) => Promise<void>;
}>();

type DrawerMode = "closed" | "add" | "edit";

const showOriginalCurrency = ref(false);
const baseCurrency = "CLP";
const drawerMode = ref<DrawerMode>("closed");
const editingTransactionId = ref("");
const queryCache = useQueryCache();
const { movementCategoryOptions } = useBudgetTree();
const transactionForm = reactive({
  kind: "expense" as TransactionKind,
  accountId: "",
  destinationAccountId: "",
  transactionDate: new Date().toISOString().slice(0, 10),
  amount: "",
  currency: "CLP",
  description: "",
  envelopeId: "",
  notes: "",
  tagIds: [] as string[],
  newTags: "",
});
const transactionKindOptions: BaseSelectOption[] = [
  { label: "Gasto", value: "expense" },
  { label: "Ingreso", value: "income" },
  { label: "Transferencia", value: "transfer" },
  { label: "Ajuste", value: "adjustment" },
];

const movementsQuery = useQuery({
  key: ["movements"],
  query: getMovements,
});
const budgetTreeQuery = useQuery({
  key: () => ["budget-tree", props.period],
  query: () => getBudgetTree(props.period),
});
const tagsQuery = useQuery({
  key: ["tags"],
  query: getTags,
});

const movements = computed(() => movementsQuery.data.value ?? []);
const movementCount = computed(() => movements.value.length);
const isDrawerOpen = computed(() => drawerMode.value !== "closed");
const hasRegisteredAccounts = computed(() => props.accounts.length > 0);
const drawerTitle = computed(() =>
  drawerMode.value === "edit" ? "Editar movimiento" : "Agregar movimiento",
);
const saveError = computed(() =>
  drawerMode.value === "edit" ? props.updateError : props.createError,
);
const isSaving = computed(() =>
  drawerMode.value === "edit" ? props.isUpdating : props.isCreating,
);
const submitLabel = computed(() =>
  drawerMode.value === "edit" ? "Guardar cambios" : "Crear movimiento",
);
const isTransfer = computed(() => transactionForm.kind === "transfer");
const accountSelectOptions = computed<BaseSelectOption[]>(() =>
  props.accounts.map((account) => ({
    label: account.displayName,
    value: account.id,
  })),
);
const transferDestinationAccounts = computed(() =>
  props.accounts.filter(
    (account) =>
      account.id !== transactionForm.accountId &&
      account.currency === transactionForm.currency,
  ),
);
const transferDestinationAccountOptions = computed<BaseSelectOption[]>(() =>
  transferDestinationAccounts.value.map((account) => ({
    label: account.displayName,
    value: account.id,
  })),
);
const movementEnvelopeOptions = computed<BaseSelectOption[]>(() => [
  { label: "Sin categoría", value: "" },
  ...movementCategoryOptions.value.map((envelope) => ({
    label: envelope.name,
    value: envelope.id,
  })),
]);
const knownTags = computed<TagDto[]>(() => {
  const byId = new Map<string, TagDto>();
  for (const tag of tagsQuery.data.value ?? []) {
    byId.set(tag.id, tag);
  }

  for (const movement of movements.value) {
    for (const tag of movement.tags) {
      byId.set(tag.id, tag);
    }
  }

  return [...byId.values()].sort((firstTag, secondTag) =>
    firstTag.name.localeCompare(secondTag.name, "es-CL"),
  );
});
const tagOptions = computed<BaseSelectOption[]>(() =>
  knownTags.value.map((tag) => ({
    label: tag.name,
    value: tag.id,
  })),
);
const selectedTags = computed(() =>
  knownTags.value.filter((tag) => transactionForm.tagIds.includes(tag.id)),
);
const newTagNames = computed(() => parseTagNames(transactionForm.newTags));
const newTagsToCreate = computed(() => {
  const knownNormalizedNames = new Set(
    knownTags.value.map((tag) => tag.normalizedName),
  );
  const selectedNormalizedNames = new Set(
    selectedTags.value.map((tag) => tag.normalizedName),
  );

  return newTagNames.value.filter((tagName) => {
    const normalizedName = normalizeTagName(tagName);
    return (
      !knownNormalizedNames.has(normalizedName) &&
      !selectedNormalizedNames.has(normalizedName)
    );
  });
});

watch(
  () => budgetTreeQuery.data.value,
  (tree) => {
    if (tree) {
      applyBudgetTree(tree);
      void queryCache.invalidateQueries({ key: ["movements"] });
    }
  },
  { immediate: true },
);

watch(
  () => props.accounts,
  (accounts) => {
    if (transactionForm.accountId === "" && accounts[0]) {
      transactionForm.accountId = accounts[0].id;
      transactionForm.currency = accounts[0].currency;
    }
  },
  { immediate: true },
);

watch(
  () => transactionForm.accountId,
  (accountId) => {
    const account = props.accounts.find(
      (candidate) => candidate.id === accountId,
    );
    if (account) {
      transactionForm.currency = account.currency;
    }
  },
);

watch(
  [() => transactionForm.accountId, () => transactionForm.kind],
  () => {
    if (!isTransfer.value) {
      transactionForm.destinationAccountId = "";
      return;
    }

    const destination = transferDestinationAccounts.value.find(
      (account) => account.id === transactionForm.destinationAccountId,
    );
    if (!destination) {
      transactionForm.destinationAccountId =
        transferDestinationAccounts.value[0]?.id ?? "";
    }
  },
);

watch(
  () => transactionForm.kind,
  (kind) => {
    if (kind === "income" && transactionForm.envelopeId === "") {
      transactionForm.envelopeId = INCOME_ENVELOPE_ID;
      return;
    }

    if (kind !== "income" && transactionForm.envelopeId === INCOME_ENVELOPE_ID) {
      transactionForm.envelopeId = "";
    }
  },
);

function openAddPanel() {
  resetForm();
  drawerMode.value = "add";
}

function openEditPanel(movement: LedgerMovement) {
  transactionForm.kind = movement.kind;
  transactionForm.accountId = sourceAccountId(movement);
  transactionForm.destinationAccountId = destinationAccountId(movement);
  transactionForm.transactionDate = movement.date;
  transactionForm.amount = movement.originalAmount;
  transactionForm.currency = movement.originalCurrency;
  transactionForm.description = movement.description;
  transactionForm.envelopeId = movement.envelopeId ?? "";
  transactionForm.notes = movement.notes ?? "";
  transactionForm.tagIds = movement.tags.map((tag) => tag.id);
  transactionForm.newTags = "";
  editingTransactionId.value = movement.transactionId;
  drawerMode.value = "edit";
}

function closePanel() {
  drawerMode.value = "closed";
}

async function submitTransaction() {
  const notes = transactionForm.notes.trim();
  const request = buildTransactionRequest(notes);

  try {
    if (drawerMode.value === "edit") {
      await props.updateTransaction({
        id: editingTransactionId.value,
        ...request,
      });
    } else {
      await props.createTransaction(request);
    }
  } catch {
    return;
  }

  resetForm();
  drawerMode.value = "closed";
}

function buildTransactionRequest(
  notes: string,
): CreateManualTransactionRequest {
  const envelopeId = transactionForm.envelopeId.trim();
  const request: CreateManualTransactionRequest = {
    kind: transactionForm.kind,
    accountId: transactionForm.accountId,
    destinationAccountId: isTransfer.value
      ? transactionForm.destinationAccountId
      : null,
    transactionDate: transactionForm.transactionDate,
    amount: transactionForm.amount.trim(),
    currency: transactionForm.currency.trim().toUpperCase(),
    accountEffect: defaultAccountEffect(transactionForm.kind),
    description: transactionForm.description.trim(),
  };

  if (!isTransfer.value && envelopeId !== "") {
    request.envelopeId = envelopeId;
  }

  if (notes !== "") {
    request.notes = notes;
  }

  const tags = buildTagNames();
  if (tags.length > 0) {
    request.tags = tags;
  }

  return request;
}

function buildTagNames(): string[] {
  const names = [
    ...selectedTags.value.map((tag) => tag.name),
    ...newTagNames.value,
  ];
  const normalizedNames = new Set<string>();
  const result: string[] = [];

  for (const name of names) {
    const normalizedName = normalizeTagName(name);
    if (normalizedName !== "" && !normalizedNames.has(normalizedName)) {
      normalizedNames.add(normalizedName);
      result.push(name.trim());
    }
  }

  return result;
}

function resetForm() {
  const firstAccount = props.accounts[0];

  editingTransactionId.value = "";
  transactionForm.kind = "expense";
  transactionForm.accountId = firstAccount ? firstAccount.id : "";
  transactionForm.destinationAccountId = "";
  transactionForm.transactionDate = new Date().toISOString().slice(0, 10);
  transactionForm.amount = "";
  transactionForm.currency = firstAccount ? firstAccount.currency : "CLP";
  transactionForm.description = "";
  transactionForm.envelopeId = "";
  transactionForm.notes = "";
  transactionForm.tagIds = [];
  transactionForm.newTags = "";
}

function defaultAccountEffect(kind: TransactionKind): AccountEffect {
  return kind === "expense" || kind === "transfer" ? "decrease" : "increase";
}

function sourceAccountId(movement: LedgerMovement): string {
  if (movement.kind === "transfer" && movement.accountEffect === "increase") {
    return movement.destinationAccountId ?? movement.accountId;
  }

  return movement.accountId;
}

function destinationAccountId(movement: LedgerMovement): string {
  if (movement.kind !== "transfer") {
    return "";
  }

  if (movement.accountEffect === "increase") {
    return movement.accountId;
  }

  return movement.destinationAccountId ?? "";
}

function parseTagNames(value: string): string[] {
  return value
    .split(",")
    .map((tagName) => tagName.trim())
    .filter((tagName) => tagName !== "");
}

function normalizeTagName(value: string): string {
  return value.trim().toLowerCase().split(/[\s_-]+/u).filter(Boolean).join(" ");
}
</script>

<template>
  <BaseCard class="movements-panel">
    <template #header>
      <div class="movements-heading">
        <div>
          <h2 id="movements-heading">Movimientos</h2>
          <span>Todos los movimientos del libro mayor</span>
        </div>

        <div class="movements-actions">
          <BaseBadge>{{ movementCount }}</BaseBadge>
          <BaseButton
            :disabled="!hasRegisteredAccounts"
            size="sm"
            type="button"
            @click="openAddPanel"
          >
            Agregar movimiento
          </BaseButton>
          <BaseCheckbox
            v-model:checked="showOriginalCurrency"
            label="Moneda original"
          />
        </div>
      </div>
    </template>

    <section aria-labelledby="movements-heading">
      <p class="currency-note">
        Valores mostrados en
        <strong>{{
          showOriginalCurrency ? "moneda original" : baseCurrency
        }}</strong>
      </p>

      <BaseAlert
        v-if="props.hasAccountsLoadError"
        class="movements-alert"
        tone="danger"
      >
        No se pudieron cargar las cuentas para registrar movimientos.
      </BaseAlert>

      <BaseAlert
        v-else-if="!hasRegisteredAccounts"
        class="movements-alert"
        tone="warning"
      >
        Registra una cuenta antes de agregar movimientos.
      </BaseAlert>

      <BaseAlert
        v-if="movementsQuery.error.value"
        class="movements-alert"
        tone="danger"
      >
        No se pudieron cargar los movimientos.
      </BaseAlert>

      <LedgerMovementsTable
        :movements="movements"
        :show-original-currency="showOriginalCurrency"
        @edit-movement="openEditPanel"
      />
    </section>
  </BaseCard>

  <Teleport to="body">
    <div
      v-if="isDrawerOpen"
      class="movement-drawer-backdrop"
      role="presentation"
      @click.self="closePanel"
    >
      <aside
        aria-labelledby="movement-drawer-title"
        aria-modal="true"
        class="movement-drawer"
        role="dialog"
      >
        <header class="movement-drawer__header">
          <div>
            <p class="eyebrow">Movimiento</p>
            <h2 id="movement-drawer-title">{{ drawerTitle }}</h2>
          </div>
          <BaseButton type="button" variant="secondary" @click="closePanel">
            Cerrar
          </BaseButton>
        </header>

        <form class="movement-form" @submit.prevent="submitTransaction">
          <BaseAlert v-if="saveError" tone="danger">
            {{ saveError }}
          </BaseAlert>

          <BaseField input-id="movement-kind" label="Tipo">
            <BaseSelect
              id="movement-kind"
              v-model="transactionForm.kind"
              :options="transactionKindOptions"
            />
          </BaseField>

          <BaseField
            input-id="movement-account"
            :label="isTransfer ? 'Desde' : 'Cuenta'"
          >
            <BaseSelect
              id="movement-account"
              v-model="transactionForm.accountId"
              :options="accountSelectOptions"
              required
            />
          </BaseField>

          <BaseField
            v-if="isTransfer"
            input-id="movement-destination-account"
            label="Hacia"
          >
            <BaseSelect
              id="movement-destination-account"
              v-model="transactionForm.destinationAccountId"
              :options="transferDestinationAccountOptions"
              required
            />
          </BaseField>

          <BaseField input-id="movement-date" label="Fecha">
            <BaseTextInput
              id="movement-date"
              v-model="transactionForm.transactionDate"
              required
              type="date"
            />
          </BaseField>

          <BaseField input-id="movement-amount" label="Monto">
            <BaseTextInput
              id="movement-amount"
              v-model="transactionForm.amount"
              inputmode="decimal"
              pattern="\d+(\.\d+)?"
              placeholder="0"
              required
            />
          </BaseField>

          <BaseField input-id="movement-currency" label="Moneda">
            <BaseTextInput
              id="movement-currency"
              v-model="transactionForm.currency"
              disabled
              maxlength="3"
              pattern="[A-Z]{3}"
              required
            />
          </BaseField>

          <BaseField
            v-if="!isTransfer"
            input-id="movement-category"
            label="Categoría"
          >
            <BaseSelect
              id="movement-category"
              v-model="transactionForm.envelopeId"
              :options="movementEnvelopeOptions"
            />
          </BaseField>

          <BaseField input-id="movement-description" label="Descripción">
            <BaseTextInput
              id="movement-description"
              v-model="transactionForm.description"
              maxlength="240"
              required
            />
          </BaseField>

          <BaseField input-id="movement-notes" label="Notas">
            <BaseTextarea
              id="movement-notes"
              v-model="transactionForm.notes"
              maxlength="1000"
            />
          </BaseField>

          <BaseField input-id="movement-tags" label="Etiquetas existentes">
            <BaseMultiSelect
              id="movement-tags"
              v-model="transactionForm.tagIds"
              display="chip"
              filter
              :options="tagOptions"
              placeholder="Seleccionar etiquetas"
            />
          </BaseField>

          <BaseField input-id="movement-new-tags" label="Nuevas etiquetas">
            <BaseTextInput
              id="movement-new-tags"
              v-model="transactionForm.newTags"
              maxlength="240"
              placeholder="home party, transporte"
            />
          </BaseField>

          <div v-if="newTagsToCreate.length > 0" class="new-tags-preview">
            <span
              v-for="tagName in newTagsToCreate"
              :key="normalizeTagName(tagName)"
              class="new-tags-preview__item"
            >
              {{ tagName.trim() }}
              <small>Se creará al guardar</small>
            </span>
          </div>

          <div class="movement-form__actions">
            <BaseButton type="button" variant="secondary" @click="closePanel">
              Cancelar
            </BaseButton>
            <BaseButton :loading="isSaving" type="submit">
              {{ submitLabel }}
            </BaseButton>
          </div>
        </form>
      </aside>
    </div>
  </Teleport>
</template>

<style scoped>
.movements-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--lu-space-3);
  margin-block-end: 0;
}

.movements-heading h2 {
  margin: 0;
  font-size: var(--lu-font-size-md);
}

.movements-heading span,
.currency-note {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-sm);
}

.movements-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--lu-space-3);
}

.currency-note {
  margin-block-end: var(--lu-space-4);
}

.currency-note strong {
  color: var(--lu-color-text);
}

.movements-alert {
  margin-block-end: var(--lu-space-4);
}

.movement-form {
  display: grid;
  gap: var(--lu-space-3);
}

.movement-form__actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--lu-space-2);
  padding-block-start: var(--lu-space-2);
}

.new-tags-preview {
  display: flex;
  flex-wrap: wrap;
  gap: var(--lu-space-2);
}

.new-tags-preview__item {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
  border: 1px solid var(--lu-color-border);
  border-radius: var(--lu-radius-pill);
  padding: var(--lu-space-1) var(--lu-space-3);
  color: var(--lu-color-text);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
}

.new-tags-preview__item small {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 700;
}

@media (max-width: 680px) {
  .movements-heading {
    align-items: stretch;
    flex-direction: column;
  }

  .movements-actions {
    justify-content: space-between;
  }
}
</style>

<style>
.movement-drawer-backdrop {
  position: fixed;
  z-index: var(--lu-z-drawer);
  inset: 0;
  display: flex;
  justify-content: flex-end;
  background: var(--lu-color-backdrop);
}

.movement-drawer {
  display: grid;
  align-content: start;
  gap: var(--lu-space-5);
  inline-size: min(var(--lu-size-drawer), 100%);
  block-size: 100dvh;
  overflow-y: auto;
  border-inline-start: 1px solid var(--lu-color-border);
  background: var(--lu-color-surface);
  padding: var(--lu-space-5);
  box-shadow: var(--lu-shadow-lg);
}

.movement-drawer__header {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: var(--lu-space-4);
}

.movement-drawer__header h2 {
  margin: 0;
  font-size: var(--lu-font-size-xl);
}

.movement-drawer .eyebrow {
  margin: 0 0 var(--lu-space-1);
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
  text-transform: uppercase;
}

@media (max-width: 680px) {
  .movement-drawer {
    inline-size: 100%;
  }
}
</style>
