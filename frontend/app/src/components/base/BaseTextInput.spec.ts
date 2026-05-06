import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import BaseTextInput from "./BaseTextInput.vue";

describe("BaseTextInput", () => {
  it("emits value updates", async () => {
    const wrapper = mount(BaseTextInput, {
      props: {
        modelValue: "",
      },
    });

    await wrapper.get("input").setValue("Cuenta corriente");

    expect(wrapper.emitted("update:modelValue")).toEqual([
      ["Cuenta corriente"],
    ]);
  });
});
