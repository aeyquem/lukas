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
  tagNames: [] as string[],
  tagInput: "",
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
const selectedNormalizedTags = computed(() =>
  new Set(transactionForm.tagNames.map(normalizeTagName)),
);
const tagSuggestions = computed(() => {
  const input = normalizeTagName(transactionForm.tagInput);
  if (input === "") {
    return knownTags.value.filter(
      (tag) => !selectedNormalizedTags.value.has(tag.normalizedName),
    );
  }

  return knownTags.value.filter(
    (tag) =>
      !selectedNormalizedTags.value.has(tag.normalizedName) &&
      tag.normalizedName.includes(input),
  );
});
const tagInputNormalized = computed(() =>
  normalizeTagName(transactionForm.tagInput),
);
const matchingInputTag = computed(() =>
  knownTags.value.find((tag) => tag.normalizedName === tagInputNormalized.value),
);
const shouldCreateInputTag = computed(
  () =>
    tagInputNormalized.value !== "" &&
    matchingInputTag.value === undefined &&
    !selectedNormalizedTags.value.has(tagInputNormalized.value),
);
const newTagsToCreate = computed(() => {
  const knownNormalizedNames = new Set(
    knownTags.value.map((tag) => tag.normalizedName),
  );

  return transactionForm.tagNames.filter(
    (tagName) => !knownNormalizedNames.has(normalizeTagName(tagName)),
  );
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
  transactionForm.tagNames = movement.tags.map((tag) => tag.name);
  transactionForm.tagInput = "";
  editingTransactionId.value = movement.transactionId;
  drawerMode.value = "edit";
}

function closePanel() {
  drawerMode.value = "closed";
}

async function submitTransaction() {
  commitTagInput();
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
  const names = transactionForm.tagNames;
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
  transactionForm.tagNames = [];
  transactionForm.tagInput = "";
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

function normalizeTagName(value: string): string {
  return value.trim().toLowerCase().split(/[\s_-]+/u).filter(Boolean).join(" ");
}

function displayTagName(value: string): string {
  return normalizeTagName(value)
    .split(" ")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function addTagName(value: string) {
  const normalizedName = normalizeTagName(value);
  if (normalizedName === "" || selectedNormalizedTags.value.has(normalizedName)) {
    transactionForm.tagInput = "";
    return;
  }

  const existingTag = knownTags.value.find(
    (tag) => tag.normalizedName === normalizedName,
  );
  transactionForm.tagNames = [
    ...transactionForm.tagNames,
    existingTag?.name ?? displayTagName(value),
  ];
  transactionForm.tagInput = "";
}

function removeTagName(tagName: string) {
  const normalizedName = normalizeTagName(tagName);
  transactionForm.tagNames = transactionForm.tagNames.filter(
    (candidate) => normalizeTagName(candidate) !== normalizedName,
  );
}

function commitTagInput() {
  addTagName(transactionForm.tagInput);
}

function handleTagInputKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" || event.key === ",") {
    event.preventDefault();
    commitTagInput();
  }
}

function selectTagSuggestion(tag: TagDto) {
  addTagName(tag.name);
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

          <BaseField input-id="movement-tags" label="Etiquetas">
            <div class="tag-picker">
              <div
                v-if="transactionForm.tagNames.length > 0"
                class="tag-picker__chips"
              >
                <span
                  v-for="tagName in transactionForm.tagNames"
                  :key="normalizeTagName(tagName)"
                  class="tag-picker__chip"
                  :data-state="
                    newTagsToCreate.includes(tagName) ? 'new' : 'existing'
                  "
                >
                  {{ tagName }}
                  <small v-if="newTagsToCreate.includes(tagName)">
                    Se creará al guardar
                  </small>
                  <button
                    :aria-label="`Quitar etiqueta ${tagName}`"
                    type="button"
                    @click="removeTagName(tagName)"
                  >
                    ×
                  </button>
                </span>
              </div>

              <BaseTextInput
                id="movement-tags"
                v-model="transactionForm.tagInput"
                autocomplete="off"
                maxlength="60"
                placeholder="Buscar o crear etiqueta"
                @blur="commitTagInput"
                @keydown="handleTagInputKeydown"
              />

              <div
                v-if="transactionForm.tagInput.trim() !== ''"
                class="tag-picker__suggestions"
              >
                <button
                  v-for="tag in tagSuggestions.slice(0, 5)"
                  :key="tag.id"
                  type="button"
                  @mousedown.prevent="selectTagSuggestion(tag)"
                >
                  {{ tag.name }}
                </button>
                <button
                  v-if="shouldCreateInputTag"
                  class="tag-picker__create"
                  type="button"
                  @mousedown.prevent="commitTagInput"
                >
                  Crear {{ displayTagName(transactionForm.tagInput) }}
                </button>
              </div>
            </div>
          </BaseField>

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

@media (max-width: 680px) {
  .movements-heading {
    align-items: stretch;
    flex-direction: column;
  }

  .movements-actions {
    justify-content: space-between;
  }
}

.tag-picker {
  position: relative;
  display: grid;
  gap: var(--lu-space-2);
}

.tag-picker__chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--lu-space-2);
}

.tag-picker__chip {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
  min-block-size: 1.75rem;
  border: 1px solid var(--lu-color-border);
  border-radius: var(--lu-radius-pill);
  padding-inline: var(--lu-space-3) var(--lu-space-1);
  color: var(--lu-color-text);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
}

.tag-picker__chip[data-state="new"] {
  border-color: color-mix(in srgb, var(--lu-color-warning) 36%, transparent);
  background: var(--lu-color-warning-soft);
}

.tag-picker__chip small {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 700;
}

.tag-picker__chip button {
  display: inline-grid;
  place-items: center;
  inline-size: 1.35rem;
  block-size: 1.35rem;
  border: 0;
  border-radius: var(--lu-radius-pill);
  background: transparent;
  color: var(--lu-color-text-muted);
  cursor: pointer;
  font: inherit;
}

.tag-picker__chip button:hover {
  background: var(--lu-color-surface-muted);
  color: var(--lu-color-text);
}

.tag-picker__suggestions {
  position: absolute;
  z-index: 1;
  inset-block-start: calc(100% + var(--lu-space-1));
  inset-inline: 0;
  display: grid;
  overflow: hidden;
  border: 1px solid var(--lu-color-border);
  border-radius: var(--lu-radius-md);
  background: var(--lu-color-surface);
  box-shadow: var(--lu-shadow-md);
}

.tag-picker__suggestions button {
  border: 0;
  background: transparent;
  color: var(--lu-color-text);
  cursor: pointer;
  padding: var(--lu-space-2) var(--lu-space-3);
  text-align: start;
}

.tag-picker__suggestions button:hover {
  background: var(--lu-color-surface-muted);
}

.tag-picker__create {
  font-weight: 800;
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
