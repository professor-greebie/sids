# Security Policy

## Supported Versions

We take security seriously. The following versions of SIDS are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.7.x   | :white_check_mark: |
| < 0.7.0 | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in SIDS, please report it responsibly:

### How to Report

**Please do NOT open a public issue.** Instead:

1. **Email**: Send details to the maintainers (check GitHub profile for contact)
2. **GitHub Security Advisory**: Use the [Security Advisory](https://github.com/professor-greebie/sids/security/advisories/new) feature

### What to Include

Please provide:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Any suggested fixes (if you have them)
- Your contact information (if you'd like updates)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Varies by severity
  - Critical: 1-7 days
  - High: 7-14 days
  - Medium: 14-30 days
  - Low: 30-90 days

### What to Expect

1. Acknowledgment of your report
2. Assessment of the vulnerability
3. Discussion of remediation steps
4. Development and testing of a fix
5. Release of a security patch
6. Public disclosure (coordinated with you)

### Recognition

We appreciate security researchers who help keep SIDS safe. With your permission, we'll credit you in:

- Release notes
- Security advisory
- CHANGELOG.md

## Security Best Practices

When using SIDS in production:

1. **Keep Updated**: Always use the latest stable version
2. **Monitor Dependencies**: Use `cargo audit` to check for vulnerable dependencies
3. **Follow Guidelines**: Implement proper error handling as documented
4. **Limit Exposure**: Run with minimal required permissions
5. **Validate Input**: Always validate and sanitize actor messages

## Known Security Considerations

### Actor System Design

- **Message Passing**: Messages between actors should not contain sensitive data in plaintext
- **Error Handling**: Errors may contain stack traces - sanitize before logging
- **Resource Limits**: Set appropriate timeouts and limits to prevent resource exhaustion
- **Supervision**: Use supervision strategies to handle actor failures gracefully

### Streaming Module

- **Backpressure**: Ensure proper backpressure handling to prevent memory exhaustion
- **Data Validation**: Validate data at source boundaries
- **File Handling**: Be cautious with file paths in streaming sources

Thank you for helping keep SIDS and its users safe!
