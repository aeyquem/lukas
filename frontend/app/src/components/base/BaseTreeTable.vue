<script setup lang="ts">
import TreeTable from "primevue/treetable";

defineOptions({
  inheritAttrs: false,
});
</script>

<template>
  <TreeTable v-bind="$attrs">
    <template
      v-for="(_, slotName) in $slots"
      #[slotName]="slotProps"
      :key="slotName"
    >
      <slot :name="slotName" v-bind="slotProps ?? {}" />
    </template>
  </TreeTable>
</template>
