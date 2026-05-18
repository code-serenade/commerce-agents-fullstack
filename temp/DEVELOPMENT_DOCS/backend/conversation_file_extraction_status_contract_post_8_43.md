# Post 8-43: Conversation File Extraction Status Contract

## Requirement Summary

The old product allowed uploaded files and materials to become part of the Q&A context after parsing. The new product already has:

- conversation file upload registration
- trusted extracted text registration
- workspace context summary
- workspace context pack injection into Agent SSE

The missing backend contract is extraction status. A file can be uploaded but not yet parsed, parsed successfully, failed, or unsupported by the current parser. Without a stable status contract, the frontend and Agent context pack can only infer this from missing text, which makes product behavior unclear.

This slice adds a status contract to the existing `conversation_files.metadata` path. It does not add OCR, PDF parsing, Office parsing, image recognition, object storage reads, or a background parser worker.

## Data Design

No new table and no migration.

Reuse:

```text
conversation_files.metadata jsonb
```

New canonical metadata shape:

```json
{
  "extractionStatus": "pending",
  "extraction": {
    "status": "pending",
    "source": "upload",
    "parser": "unassigned",
    "textChars": 0,
    "updatedAt": "2026-05-13T00:00:00Z",
    "metadata": {}
  }
}
```

Successful extracted text shape:

```json
{
  "extractionStatus": "extracted",
  "extractedText": "readable text",
  "textContent": "readable text",
  "extraction": {
    "status": "extracted",
    "contentKind": "plain_text",
    "source": "trusted_api",
    "parser": "external_adapter",
    "text": "readable text",
    "textChars": 13,
    "updatedAt": "2026-05-13T00:00:00Z",
    "metadata": {}
  }
}
```

Failure shape:

```json
{
  "extractionStatus": "failed",
  "extraction": {
    "status": "failed",
    "contentKind": "plain_text",
    "source": "adapter",
    "parser": "pdf",
    "textChars": 0,
    "failureReason": "encrypted pdf",
    "updatedAt": "2026-05-13T00:00:00Z",
    "metadata": {}
  }
}
```

Allowed statuses:

```text
pending
extracted
failed
unsupported
```

Legacy input statuses `ready` and `success` are normalized to `extracted`.

## Permission Design

Same as Post 8-16:

- endpoint requires App JWT
- `user_id` comes from JWT
- file must belong to current user
- file must be active
- file conversation must belong to current user and not be deleted

## HTTP Contract

Existing endpoint is extended:

```text
PUT /api/app/conversation-files/{fileId}/extraction
```

Request:

```json
{
  "status": "extracted",
  "text": "readable text",
  "contentKind": "plain_text",
  "source": "trusted_api",
  "parser": "external_adapter",
  "language": "zh-CN",
  "failureReason": null,
  "metadata": {}
}
```

Rules:

- `status` is optional; omitted means `extracted` for backward compatibility.
- `status=extracted` requires non-empty `text`.
- `status=pending|failed|unsupported` clears any existing readable text fields.
- `text` is capped at 100000 chars.
- `metadata` must be a JSON object.
- `source` defaults to `trusted_api`.
- `parser` defaults to `external_adapter`, or `manual` when `source=manual`.

Response remains `ConversationFileResponse`.

## Workspace Context Pack Behavior

`WorkspaceApi::context_pack` now reads extraction status before reading attachment text:

- `extracted` with text -> `contentKind=extracted_text`
- `extracted` without text -> `contentKind=metadata_only`
- `pending` -> `contentKind=extraction_pending`
- `failed` -> `contentKind=extraction_failed`
- `unsupported` -> `contentKind=extraction_unsupported`

Only `extracted` can contribute readable text to Agent context. This prevents the Agent from using stale extracted text after a parser failure.

## API/SSE Documentation

Update:

- `API_DOCS/app.md`
- `API_DOCS/sse.md`

## Frontend Scope

Only the web API adapter type is updated in this slice. No new UI entry is added because parser status display should follow backend contract stabilization and should use the existing workbench/context-pack surfaces.

## Validation Plan

Run:

```text
cargo fmt
cargo test -p service conversation_file_extraction -- --nocapture
cargo test -p service context_pack_respects_attachment_extraction_status -- --nocapture
cargo check -p service
cargo check -p web-server
cargo clippy -p service -p web-server
pnpm -C frontend/apps/web rules
pnpm -C frontend/apps/web lint
pnpm -C frontend/apps/web test
pnpm -C frontend/apps/web build
pnpm -C frontend rules
git diff --check
```
