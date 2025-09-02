# Project Workflow Template

Customize this template for your specific project's tools, CI/CD, and development environment.

## Development Environment Setup

### Prerequisites:
- [ ] Language runtime installed (specify version)
- [ ] Package manager configured
- [ ] IDE/Editor configured with project settings
- [ ] Version control initialized

### Local Setup Commands:
```bash
# Example commands - customize for your project
# git clone <repository>
# cd <project>
# <install dependencies>
# <run initial setup>
# <run tests to verify setup>
```

## Daily Development Workflow

### Starting Work:
1. Pull latest changes from main branch
2. Check project/docs/ROADMAP.md for current priorities
3. Review project/docs/DEV_LOG.md for recent decisions
4. Run full test suite to ensure clean baseline
5. Create feature branch for work

### During Development:
1. Follow MCP/CLOSE_TO_SHORE.md workflow steps
2. Write tests first (TDD approach)
3. Implement minimal changes to pass tests
4. Run tests frequently (every 5-10 minutes)
5. Commit small, logical changes

### Completing Work:
1. Full test suite passes
2. Manual smoke test completed
3. Update project/docs/HOP_SUMMARIES.md
4. Update project/docs/DEV_LOG.md with decisions
5. Create pull request with clear description

## Build and Test Commands

### Local Development:
```bash
# Run all tests
<test command>

# Run specific test suite
<unit test command>
<integration test command>

# Build application
<build command>

# Run application locally
<run command>
```

### VS Code Tasks:
- Configure tasks.json for one-key testing
- Set up launch configurations for debugging
- Add file watchers for automatic test runs

## CI/CD Pipeline

### Automated Checks:
- [ ] Lint and code style validation
- [ ] Full test suite execution
- [ ] Build verification
- [ ] Security scanning (if applicable)

### Deployment Process:
- [ ] Staging deployment for testing
- [ ] Manual validation in staging
- [ ] Production deployment process
- [ ] Rollback procedures

## Quality Gates

### Before Merge:
- All tests pass
- Code review completed
- Documentation updated
- No lint errors or warnings

### Before Release:
- Full regression testing
- Performance validation
- Security review (if applicable)
- Documentation up to date

## Project-Specific Tools

### Development Tools:
- [ ] Debugger configuration
- [ ] Profiling tools setup
- [ ] Database tools (if applicable)
- [ ] API testing tools

### Monitoring and Logging:
- [ ] Local logging configuration
- [ ] Error tracking setup
- [ ] Performance monitoring
- [ ] Health check endpoints

## Troubleshooting

### Common Issues:
- [ ] Environment setup problems
- [ ] Test failures and debugging
- [ ] Build issues
- [ ] Runtime problems

### Getting Help:
- Check project/docs/DEV_LOG.md for previous solutions
- Review project/docs/PROJECT_INDEX.md for system overview
- Consult team documentation or resources

#EOF
