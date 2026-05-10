<script setup lang="ts">
import { computed } from "vue";

import BaseButton from "@/common/components/base/BaseButton.vue";
import BaseDatePicker from "@/common/components/base/BaseDatePicker.vue";
import { currentPeriod } from "@/common/formatters";

const props = defineProps<{
  inputId: string;
  modelValue: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const selectedDate = computed(() => periodToDate(props.modelValue));
const selectedPeriodLabel = computed(() =>
  new Intl.DateTimeFormat("es-CL", {
    month: "long",
    year: "numeric",
  }).format(selectedDate.value),
);
const currentPeriodValue = computed(() => currentPeriod());
const isCurrentPeriod = computed(() => props.modelValue === currentPeriodValue.value);

function periodToDate(period: string): Date {
  const match = /^(\d{4})-(\d{2})$/.exec(period);
  if (!match) {
    return currentMonthDate();
  }

  const year = Number.parseInt(match[1], 10);
  const monthIndex = Number.parseInt(match[2], 10) - 1;
  if (!Number.isInteger(year) || monthIndex < 0 || monthIndex > 11) {
    return currentMonthDate();
  }

  return new Date(year, monthIndex, 1);
}

function currentMonthDate(): Date {
  const today = new Date();
  return new Date(today.getFullYear(), today.getMonth(), 1);
}

function dateToPeriod(date: Date): string {
  const year = date.getFullYear().toString();
  const month = (date.getMonth() + 1).toString().padStart(2, "0");
  return `${year}-${month}`;
}

function updateDate(value: unknown) {
  if (!(value instanceof Date)) {
    return;
  }

  emit("update:modelValue", dateToPeriod(value));
}

function movePeriod(monthOffset: number) {
  const date = selectedDate.value;
  const nextDate = new Date(date.getFullYear(), date.getMonth() + monthOffset, 1);
  emit("update:modelValue", dateToPeriod(nextDate));
}

function goToCurrentPeriod() {
  if (isCurrentPeriod.value) {
    return;
  }

  emit("update:modelValue", currentPeriodValue.value);
}
</script>

<template>
  <div class="period-picker">
    <BaseButton
      aria-label="Periodo anterior"
      class="period-picker__step"
      size="sm"
      type="button"
      variant="secondary"
      @click="movePeriod(-1)"
    >
      <span aria-hidden="true">‹</span>
    </BaseButton>

    <BaseDatePicker
      :input-id="props.inputId"
      :manual-input="false"
      :model-value="selectedDate"
      :aria-label="selectedPeriodLabel"
      class="period-picker__date"
      date-format="MM yy"
      view="month"
      @update:model-value="updateDate"
    />

    <BaseButton
      aria-label="Periodo siguiente"
      class="period-picker__step"
      size="sm"
      type="button"
      variant="secondary"
      @click="movePeriod(1)"
    >
      <span aria-hidden="true">›</span>
    </BaseButton>

    <BaseButton
      aria-label="Ir al periodo actual"
      class="period-picker__today"
      :disabled="isCurrentPeriod"
      size="sm"
      title="Periodo actual"
      type="button"
      variant="secondary"
      @click="goToCurrentPeriod"
    >
      <span aria-hidden="true" class="period-picker__today-icon"></span>
    </BaseButton>
  </div>
</template>

<style scoped>
.period-picker {
  display: grid;
  grid-template-columns:
    var(--lu-size-control-sm) minmax(9rem, 1fr) var(--lu-size-control-sm)
    var(--lu-size-control-sm);
  align-items: center;
  gap: var(--lu-space-2);
}

.period-picker__step,
.period-picker__today {
  inline-size: var(--lu-size-control-sm);
  padding-inline: 0;
}

.period-picker__step span {
  font-size: var(--lu-font-size-xl);
  line-height: var(--lu-line-tight);
}

.period-picker__date {
  min-inline-size: 0;
}

.period-picker__date :deep(.p-inputtext) {
  inline-size: 100%;
  text-align: center;
  text-transform: capitalize;
}

.period-picker__today-icon {
  position: relative;
  display: inline-block;
  inline-size: 1rem;
  block-size: 1rem;
  border: 2px solid currentcolor;
  border-radius: 0.2rem;
}

.period-picker__today-icon::before {
  position: absolute;
  inset-block-start: 0.2rem;
  inset-inline: 0;
  border-block-start: 2px solid currentcolor;
  content: "";
}

.period-picker__today-icon::after {
  position: absolute;
  inset-block-end: 0.18rem;
  inset-inline-start: 50%;
  inline-size: 0.32rem;
  block-size: 0.32rem;
  border-radius: var(--lu-radius-pill);
  background: currentcolor;
  content: "";
  transform: translateX(-50%);
}
</style>
