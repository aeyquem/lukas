<script setup lang="ts">
import {
  amountParts,
  amountTone,
  signedSpentAmountParts,
} from "@/features/budget/model/amounts";

const props = withDefaults(
  defineProps<{
    amount: string;
    signed?: boolean;
    toned?: boolean;
  }>(),
  {
    signed: false,
    toned: false,
  },
);
</script>

<template>
  <span
    v-if="props.signed"
    class="numeric"
    :aria-label="signedSpentAmountParts(props.amount).text"
    :data-direction="signedSpentAmountParts(props.amount).direction"
  >
    <span class="numeric__prefix">
      {{ signedSpentAmountParts(props.amount).prefix }}
    </span>
    <span class="numeric__amount">
      {{ signedSpentAmountParts(props.amount).number }}
    </span>
  </span>
  <span
    v-else
    class="numeric"
    :aria-label="amountParts(props.amount).text"
    :data-tone="props.toned ? amountTone(props.amount) : undefined"
  >
    <span class="numeric__prefix">
      {{ amountParts(props.amount).prefix }}
    </span>
    <span class="numeric__amount">
      {{ amountParts(props.amount).number }}
    </span>
  </span>
</template>

<style scoped>
.numeric {
  display: inline-flex;
  align-items: baseline;
  justify-content: flex-end;
  gap: var(--lu-space-1);
  color: var(--lu-color-text);
  inline-size: 100%;
  white-space: nowrap;
}

.numeric__amount {
  text-align: end;
}

.numeric[data-tone="negative"] {
  color: var(--lu-color-danger);
}

.numeric[data-direction="in"] {
  color: var(--lu-color-positive);
}

.numeric[data-direction="out"] {
  color: var(--lu-color-danger);
}
</style>
