# Test Policy (System-Agnostic)

Standards and criteria for testing in Close-to-Shore projects.

## Test Suite Structure

### Required Test Types:
1. **Unit Tests**: Component isolation and logic validation
2. **Integration Tests**: Cross-system workflow validation
3. **Smoke Tests**: Critical boot paths and basic functionality
4. **Game-Flow Tests**: End-to-end user journey validation

### Test Organization:
```
tests/
├── unit/           # Isolated component tests
├── integration/    # Cross-system tests
├── smoke/          # Boot and critical path tests
├── game_flow/      # User journey tests
└── scratch/        # Temporary tests (git-ignored)
```

## Testing Standards

### Unit Tests:
- Test public API only
- No dependencies on external systems
- Fast execution (< 100ms per test)
- Clear, descriptive test names
- One assertion per test concept

### Integration Tests:
- Test system interactions
- Mock external dependencies
- Validate data flow between components
- Test error handling and edge cases

### Smoke Tests:
- Application boots without errors
- Critical systems initialize properly
- Basic user actions complete successfully
- Data loading and saving works

### Game-Flow Tests:
- Complete user scenarios
- Test real user workflows
- Validate business logic end-to-end
- Include happy path and error cases

## Definition of Green

### All Tests Must:
- Pass consistently (no flaky tests)
- Run in reasonable time (< 30 seconds total)
- Provide clear failure messages
- Not leave artifacts or side effects

### Test Promotion Criteria:
1. All existing tests pass
2. New feature has complete test coverage
3. No test debt introduced
4. Manual smoke test completed

## Test-First Protocol

### Before Implementation:
1. Write failing unit tests for new functionality
2. Write failing integration tests for system interactions
3. Write failing game-flow tests for user scenarios
4. Ensure tests fail for correct reasons

### During Implementation:
- Run tests frequently (every 5-10 minutes)
- Fix tests immediately when broken
- Refactor tests alongside code
- Add tests for discovered edge cases

### After Implementation:
- Full test suite passes
- Manual validation completed
- Performance regression check
- Test coverage review

## Test Data Management

### Test Data Principles:
- Self-contained test data
- No dependencies on external files
- Predictable, minimal datasets
- Clean up after each test

### Data Isolation:
- Each test creates its own data
- No shared mutable state between tests
- Use factories or builders for test data
- Mock external dependencies

## Continuous Testing

### Local Development:
- Tests run on every save (if possible)
- Pre-commit hooks run test suite
- Fast feedback on test failures

### CI/CD Integration:
- All tests run on pull requests
- No merge if tests fail
- Performance regression detection
- Test coverage reporting

#EOF
