import { describe, expect, it } from "vitest";

import { humaniseBytes, humaniseUptime } from "./utils";

describe("humaniseBytes", () => {
  it("renders base-2 units", () => {
    expect(humaniseBytes(512)).toBe("512 B");
    expect(humaniseBytes(1536)).toBe("1.5 KB");
    expect(humaniseBytes(5 * 1024 * 1024)).toBe("5.0 MB");
  });
  it("dashes on null/undefined", () => {
    expect(humaniseBytes(null)).toBe("-");
    expect(humaniseBytes(undefined)).toBe("-");
  });
});

describe("humaniseUptime", () => {
  it("composes d/h/m", () => {
    expect(humaniseUptime(90061)).toBe("1d 1h 1m");
    expect(humaniseUptime(3661)).toBe("1h 1m");
    expect(humaniseUptime(0)).toBe("0s");
  });
});
