import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import BaseButton from "@/common/components/base/BaseButton.vue";

describe("BaseButton", () => {
  it("renders its default slot", () => {
    const wrapper = mount(BaseButton, {
      slots: {
        default: "Guardar",
      },
    });

    expect(wrapper.text()).toContain("Guardar");
  });

  it("uses submit type when requested", () => {
    const wrapper = mount(BaseButton, {
      props: {
        type: "submit",
      },
    });

    expect(wrapper.get("button").attributes("type")).toBe("submit");
  });

  it("disables the button while loading", () => {
    const wrapper = mount(BaseButton, {
      props: {
        loading: true,
      },
    });

    expect(wrapper.get("button").attributes("disabled")).toBeDefined();
    expect(wrapper.get("button").attributes("aria-busy")).toBe("true");
  });
});
