<script setup lang="ts">
import type { BudgetNodeType, BudgetTreeNode } from "@/features/budget/model/treeNodes";
import BaseButton from "@/common/components/base/BaseButton.vue";
import BaseTextInput from "@/common/components/base/BaseTextInput.vue";

defineProps<{
  canDelete: boolean;
  isEditing: boolean;
  isNameEditing: boolean;
  isProtected: boolean;
  node: BudgetTreeNode;
}>();

defineEmits<{
  addEnvelope: [subcategoryId: string];
  addSubcategory: [categoryId: string];
  completeName: [nodeType: BudgetNodeType, nodeId: string];
  deleteCategory: [categoryId: string];
  deleteEnvelope: [envelopeId: string];
  deleteSubcategory: [subcategoryId: string];
  editName: [nodeId: string];
  openGoal: [envelopeId: string];
  updateName: [nodeType: BudgetNodeType, nodeId: string, name: string];
}>();

const vNameInputFocus = {
  mounted(element: HTMLElement) {
    element.focus();

    if (element instanceof HTMLInputElement) {
      element.select();
    }
  },
};
</script>

<template>
  <div class="node-name-cell">
    <BaseTextInput
      v-if="isEditing && isNameEditing"
      v-name-input-focus
      :aria-label="`Nombre para ${node.data.name}`"
      class="node-name-input"
      :model-value="node.data.name"
      type="text"
      @blur="$emit('completeName', node.data.nodeType, node.key)"
      @keydown.enter="$emit('completeName', node.data.nodeType, node.key)"
      @update:model-value="
        $emit('updateName', node.data.nodeType, node.key, $event)
      "
    />
    <span
      v-else-if="node.data.nodeType === 'envelope'"
      class="envelope-label"
      :data-envelope-id="node.key"
    >
      {{ node.data.name }}
      <BaseButton
        v-if="node.data.goalStatus"
        size="sm"
        type="button"
        variant="secondary"
        :aria-label="`Abrir meta ${node.data.name}`"
        @click="$emit('openGoal', node.key)"
      >
        Meta
      </BaseButton>
    </span>
    <span
      v-else-if="node.data.nodeType === 'subcategory'"
      class="group-label"
      data-level="subcategory"
      :data-subcategory-id="node.key"
    >
      {{ node.data.name }}
    </span>
    <span
      v-else
      class="group-label"
      data-level="category"
      :data-category-id="node.key"
    >
      {{ node.data.name }}
    </span>

    <BaseButton
      v-if="isEditing && !isProtected"
      size="sm"
      type="button"
      variant="secondary"
      :aria-label="`Renombrar ${node.data.name}`"
      @click="$emit('editName', node.key)"
    >
      Renombrar
    </BaseButton>
    <BaseButton
      v-if="isEditing && node.data.nodeType === 'category' && canDelete"
      size="sm"
      type="button"
      variant="secondary"
      :aria-label="`Agregar subcategoría en ${node.data.name}`"
      @click="$emit('addSubcategory', node.key)"
    >
      + Subcategoría
    </BaseButton>
    <BaseButton
      v-else-if="isEditing && node.data.nodeType === 'subcategory' && canDelete"
      size="sm"
      type="button"
      variant="secondary"
      :aria-label="`Agregar sobre en ${node.data.name}`"
      @click="$emit('addEnvelope', node.key)"
    >
      + Sobre
    </BaseButton>
    <BaseButton
      v-if="isEditing && node.data.nodeType === 'category' && !isProtected"
      size="sm"
      type="button"
      variant="danger"
      :aria-label="`Eliminar categoría ${node.data.name}`"
      @click="$emit('deleteCategory', node.key)"
    >
      Eliminar
    </BaseButton>
    <BaseButton
      v-else-if="
        isEditing && node.data.nodeType === 'subcategory' && !isProtected
      "
      size="sm"
      type="button"
      variant="danger"
      :aria-label="`Eliminar subcategoría ${node.data.name}`"
      @click="$emit('deleteSubcategory', node.key)"
    >
      Eliminar
    </BaseButton>
    <BaseButton
      v-else-if="isEditing && node.data.nodeType === 'envelope' && canDelete"
      size="sm"
      type="button"
      variant="danger"
      :aria-label="`Eliminar sobre ${node.data.name}`"
      @click="$emit('deleteEnvelope', node.key)"
    >
      Eliminar
    </BaseButton>
  </div>
</template>

<style scoped>
.node-name-cell {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
  max-inline-size: 100%;
}

.group-label,
.envelope-label {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
}

.group-label[data-level="subcategory"] {
  color: var(--lu-color-text-muted);
}

.node-name-input {
  min-inline-size: 10rem;
  max-inline-size: 16rem;
  min-block-size: var(--lu-size-control-sm);
}

@media (max-width: 680px) {
  .node-name-cell {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
