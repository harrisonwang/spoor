export interface ParseOptions {
  sourceName?: string;
  contentType?: string;
  format?: string;
  maxParseBytes?: number;
}

export interface DocumentResult {
  source: string;
  format: string;
  markdown: string;
}

export interface TableResult {
  tables: Array<Record<string, unknown>>;
  serialized_bytes: number;
}

export type ParseContent =
  | { kind: 'document'; value: DocumentResult }
  | { kind: 'tables'; value: TableResult };

export type WarningLocation =
  | { kind: 'page'; number: number }
  | { kind: 'slide'; number: number };

export type WarningCode =
  | 'pdf_page_no_text_layer'
  | 'pdf_page_suspicious_text_layer'
  | 'merged_table_structure_not_preserved'
  | 'embedded_visuals_omitted';

export interface SpoorWarning {
  code: WarningCode;
  message: string;
  location?: WarningLocation;
}

export interface ParseResult {
  content: ParseContent;
  warnings: SpoorWarning[];
  stats: {
    input_bytes: number;
    output_bytes: number;
    format: string;
  };
}

export interface SpoorError extends Error {
  is_error: true;
  code: string;
  reason: string;
  hint: string;
  recoverable: boolean;
  stage?: string;
}

export function detectFormat(data: Buffer, sourceName?: string | null): string;
export function parseBytes(data: Buffer, options?: ParseOptions | null): ParseResult;
