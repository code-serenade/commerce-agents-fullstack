# utils crate boundary

`utils` is the backend workspace crate for pure helper code only.

Allowed:

- Pure functions and deterministic data conversions.
- Small format helpers, such as MIME normalization and data URL building.
- Generic validation helpers that do not know about business tables or API flows.
- Code that can be tested without a database, network, S3 client, or service context.

Not allowed:

- Business concepts such as conversations, products, workspaces, users, roles, or permissions.
- Database access, migrations, repositories, transactions, or entity models.
- Service orchestration, HTTP handlers, SSE logic, storage lifecycle management, or S3 clients.
- Helpers that hide side effects such as network calls, file IO, process spawning, or global config reads.

Rule of thumb:

If a helper needs to know why the data exists, who owns it, where it is stored, or when it should be deleted, it does not belong in `utils`.
