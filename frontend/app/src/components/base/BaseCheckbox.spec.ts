import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import BaseCheckbox from "@/components/base/BaseCheckbox.vue";

describe("BaseCheckbox", () => {
  it("emits checked updates", async () => {
    const wrapper = mount(BaseCheckbox, {
      props: {
        checked: false,
        label: "Mostrar moneda original",
      },
    });

    await wrapper.getComponent({ name: "Checkbox" }).vm.$emit(
      "update:modelValue",
      true,
    );

    expect(wrapper.emitted("update:checked")).toEqual([[true]]);
  });
});
