# skills/

Drop-in skill files. Each `*.md` here gets loaded by `SkillRegistry::load_dir`
at server startup and concatenated into the system prompt the model sees.

Format (matches Claude Code skills):

```markdown
---
name: my-skill                 # required, unique
description: one sentence      # required
components: [Page, Card]       # optional, narrows the component manifest
triggers: [keyword1, keyword2] # optional, future selective loading
---

Body in plain markdown. Concatenated verbatim into the system prompt.
```

Add a skill: drop a new `.md` here and restart the server. Remove a skill:
delete the file. No code changes; no rebuild.

This README is skipped automatically by the loader because it has no YAML
frontmatter — only files starting with `---` are treated as skills.
