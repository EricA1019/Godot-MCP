# Tavily API Protocol

Guidelines for when and how to use Tavily API for project clarification.

## When to Use Tavily

### Required Conditions (ALL must be met):
1. **Ambiguity exists** in project requirements or technical approach
2. **Local documentation insufficient** (checked project/docs/ thoroughly)
3. **Clarification needed** for external APIs, libraries, or best practices
4. **User explicitly requests** web research or external validation

### Specific Use Cases:
- Latest API documentation for external libraries
- Current best practices for specific technologies
- Validation of technical approaches against community standards
- Recent changes in frameworks or tools
- Security considerations for specific implementations

## When NOT to Use Tavily

### Avoid For:
- Project-specific decisions (use project/docs/ instead)
- Basic programming concepts
- Well-established patterns in your codebase
- Internal architecture decisions
- Debugging project-specific code

## Usage Protocol

### Before Calling Tavily:
1. Check project/docs/ROADMAP.md for context
2. Review project/docs/DEV_LOG.md for previous decisions
3. Verify project/docs/PROJECT_INDEX.md for existing systems
4. Confirm information not available in MCP/ documentation

### Query Format:
```
"[Technology/Framework] [specific question] best practices [year]"
```

Examples:
- "Godot 4.3 autoload singleton best practices 2025"
- "Python pytest fixture setup patterns 2025"
- "JSON schema validation libraries comparison 2025"

### After Using Tavily:
1. Document findings in project/docs/DEV_LOG.md
2. Update project/docs/PROJECT_INDEX.md if relevant
3. Consider if findings should update MCP/ documentation

## Rate Limiting
- Maximum 3 Tavily calls per hop
- Each call must be justified in project/docs/DEV_LOG.md
- Focus on actionable, project-relevant information

## Integration with Workflow
- Use during Planning & Setup phase primarily
- Avoid during Implementation phase unless critical blocker
- Document all external research in project/docs/DEV_LOG.md

#EOF
