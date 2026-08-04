export interface DiagnosticRange {
  start: { line: number; character: number };
  end: { line: number; character: number };
}

export interface GeulLintDiagnostic {
  range: DiagnosticRange;
  source?: string;
  data?: { replacement?: unknown; safeFix?: unknown };
}

export interface SafeFixEdit {
  range: DiagnosticRange;
  newText: string;
}

export function collectSafeFixEdits(
  diagnostics: readonly GeulLintDiagnostic[],
): SafeFixEdit[] {
  return diagnostics
    .filter(
      (diagnostic) =>
        diagnostic.source === "geullint" &&
        diagnostic.data?.safeFix === true &&
        typeof diagnostic.data.replacement === "string",
    )
    .map((diagnostic) => ({
      range: diagnostic.range,
      newText: diagnostic.data?.replacement as string,
    }));
}
