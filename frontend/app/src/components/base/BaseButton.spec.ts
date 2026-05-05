import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import BaseButton from "./BaseButton.vue";

describe("BaseButton", () => {
  it("renders its default slot", () => {
    const wrapper = mount(BaseButton, {
      slots: {
        default: "Guardar",
      },
    });

    expect(wrapper.text()).toContain("Guardar");
  });
});
