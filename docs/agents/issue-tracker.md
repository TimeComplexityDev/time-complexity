# Issue tracker

Track: local markdown
CLI: none (filesystem-only)

Issues for this repo live as Markdown files under `.scratch/<feature>/`. Each file contains the frontmatter:

```yaml
---
id: <automatically assigned by triage skill>
title: <short title>
status: open|in-progress|done|wontfix
labels: [<label>, ...]
---
```

No GitHub or GitLab issues are used. The `triage`, `to-tickets`, `to-spec`, and `qa` skills should read from and write to these files.
