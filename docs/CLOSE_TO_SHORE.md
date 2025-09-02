# Close-to-Shore MCP (System-Agnostic)

Short hops, loud logs, data-driven content, always-green tests.

## 0) Core Principles
- Short hops: tiny, runnable vertical slices.
- Always green: tests pass and the program boots without errors.
- Data-driven first: content lives in data files (JSON/DB/resources); systems discover content.
- Avoid hard-coding: prefer tables/registries over branches.
- Auto-populated UI: containers spawn controls based on data/state.
- Traceable logs: tagged logs; never silently fail.

## Definition of Done (per hop)
1) Unit + integration + smoke + game-flow tests green
2) Data schemas validated (JSON or DB migrations applied)
3) App boots and demonstrates the hop feature
4) Player test completed and documented
5) Logs clean of unexpected warnings/errors
6) No hard-wired paths (safe fallbacks allowed)
7) Commit + tag; docs updated

## Workflow (6 steps)
1) Planning & Setup: read project/docs/ROADMAP.md, set scope, baseline tests, boot validation
2) Test-First: write failing tests (unit/integration/game-flow), author schemas
3) Implement: iterate to green, run often
4) Integrate & Validate: full suite, schema checks, manual boot
5) Player Validation: ask human to play; document feedback in project/docs/DEV_LOG.md
6) Docs & Completion: update project docs, tag, propose improvements

## Documentation Requirements
Every project must have:
- project/docs/ROADMAP.md: high-level checklist of planned hops
- project/docs/DEV_LOG.md: chronological decisions and issues
- project/docs/HOP_SUMMARIES.md: brief summary of each completed hop
- project/docs/README.md: project overview and setup
- project/docs/PROJECT_INDEX.md: living index of systems, scenes, data, tests

## Agent Protocol
1) Always check project/docs/ folder before starting any work
2) Follow MCP workflows and protocols unless ambiguous
3) Use TAVILY_PROTOCOL.md for clarification needs
4) Document every hop in project/docs/HOP_SUMMARIES.md
5) Update project/docs/DEV_LOG.md with decisions and issues

Living document — refine as habits evolve.

#EOF
