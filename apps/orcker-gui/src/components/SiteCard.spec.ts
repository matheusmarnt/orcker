import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const openInBrowser = vi.fn();
const openPath = vi.fn();
vi.mock("@/ipc/client", () => ({
  openInBrowser: (...args: unknown[]) => openInBrowser(...args),
  openPath: (...args: unknown[]) => openPath(...args),
}));

import SiteCard from "./SiteCard.vue";
import type { SiteEntry, StatusReport } from "@/ipc/types";

function wpSite(overrides: Partial<SiteEntry> = {}): SiteEntry {
  return {
    name: "blog",
    document_root: "/srv/blog",
    php: "8.3",
    secure: false,
    kind: "linked",
    is_wordpress: true,
    wp_auto_login: false,
    ...overrides,
  };
}

function boundReport(): StatusReport {
  return {
    resolver_installed: true,
    dns_unbound: null,
    tld: "test",
    http: { requested: 80, bound: 80, fell_back: false },
    https: { requested: 443, bound: 443, fell_back: false },
  } as unknown as StatusReport;
}

function mountCard(site: SiteEntry, report: StatusReport | null) {
  return mount(SiteCard, { props: { site, report, tld: "test" } });
}

/** Clicks the WPA quick-action chip, which renders for any WordPress site.
 *  The always-available WP Admin entry point also lives in the details sidebar
 *  (see SiteDetailsSidebar.spec.ts); both go through `openWpAdmin`. */
async function clickWpaChip(wrapper: ReturnType<typeof mountCard>) {
  const chip = wrapper.findAll("button").find((b) => b.text() === "WPA");
  if (!chip) throw new Error("WPA chip not rendered");
  await chip.trigger("click");
}

describe("SiteCard WP Admin chip", () => {
  beforeEach(() => {
    openInBrowser.mockReset();
    openPath.mockReset();
  });

  it("opens the plain wp-admin link in unbound mode", async () => {
    const site = wpSite();
    const wrapper = mountCard(site, null); // no report => the localhost `/~` form

    await clickWpaChip(wrapper);

    expect(openInBrowser).toHaveBeenCalledWith("http://localhost:8080/~blog.test/wp-admin/");
  });

  it("opens the plain wp-admin link when bound", async () => {
    const site = wpSite();
    const wrapper = mountCard(site, boundReport());

    await clickWpaChip(wrapper);

    expect(openInBrowser).toHaveBeenCalledWith("http://blog.test/wp-admin/");
  });

  it("shows the WPA chip on a WordPress site even with auto-login off", () => {
    const wrapper = mountCard(wpSite({ is_wordpress: true, wp_auto_login: false }), boundReport());

    expect(wrapper.findAll("button").find((b) => b.text() === "WPA")).toBeDefined();
  });

  it("hides the WPA chip on non-WordPress sites", () => {
    const wrapper = mountCard(wpSite({ is_wordpress: false }), boundReport());

    expect(wrapper.findAll("button").find((b) => b.text() === "WPA")).toBeUndefined();
  });
});

describe("SiteCard actions", () => {
  beforeEach(() => {
    openInBrowser.mockReset();
    openPath.mockReset();
  });

  it("opens the site details from the edit action", async () => {
    const site = wpSite();
    const wrapper = mountCard(site, boundReport());

    await wrapper.find('[aria-label="Edit blog"]').trigger("click");

    expect(wrapper.emitted("edit")).toEqual([[site]]);
    expect(openInBrowser).not.toHaveBeenCalled();
  });

  it("no longer renders a per-site actions menu", () => {
    const wrapper = mountCard(wpSite(), boundReport());

    expect(wrapper.find('[aria-label="Actions for blog"]').exists()).toBe(false);
  });
});
