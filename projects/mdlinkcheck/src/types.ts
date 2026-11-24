export type Severity = "error" | "warning";

export interface Problem {
  file: string;
  line: number;
  column: number;
  target: string;
  rule: "missing-file" | "missing-anchor" | "broken-url" | "unreachable-url";
  message: string;
  severity: Severity;
}
