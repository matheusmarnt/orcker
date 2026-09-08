import { describe, expect, it } from "vitest";
import { isContractMirror } from "./contractMirror.mjs";

describe("isContractMirror", () => {
  it("exempts an Extract<Response> alias whose wire tag the Rust side still defines", () => {
    const types = `export type SitesResponse = Extract<Response, { type: "sites" }>;\n`;
    const rust = `pub enum Response {\n    Sites(SitesReport),\n}\n`;
    expect(isContractMirror("SitesResponse", types, rust)).toBe(true);
  });

  it("does not exempt an Extract<Response> alias whose Rust variant is gone", () => {
    const types = `export type ServicesResponse = Extract<Response, { type: "services" }>;\n`;
    const rust = `pub enum Response {\n    Sites(SitesReport),\n}\n`;
    expect(isContractMirror("ServicesResponse", types, rust)).toBe(false);
  });

  it("exempts a plain interface/type by matching its own name in Rust", () => {
    const types = `export interface JobState {\n  id: string;\n}\n`;
    const rust = `pub enum JobState {\n    Running,\n    Done,\n}\n`;
    expect(isContractMirror("JobState", types, rust)).toBe(true);
  });

  it("does not exempt a plain type with no Rust identifier at all", () => {
    const types = `export interface DatabaseSummary {\n  name: string;\n}\n`;
    const rust = `pub enum Response {\n    Sites(SitesReport),\n}\n`;
    expect(isContractMirror("DatabaseSummary", types, rust)).toBe(false);
  });
});
