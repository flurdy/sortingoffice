# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 3.x.x   | :white_check_mark: |
| 2.x.x   | :x:                |
| 1.x.x   | :x:                |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you believe you have found a security vulnerability, please report it to us as described below.

**Please do not report security vulnerabilities through public GitHub issues.**

### How to Report

1. **Create a private GitHub Security Advisory** at [github.com/flurdy/sortingoffice/security/advisories](https://github.com/flurdy/sortingoffice/security/advisories)
2. **Contact us via our contact form** at [ltrbx.io/flurdy.com](https://ltrbx.io/flurdy.com)
3. **PGP keys are available** at [keybase.io/flurdy](https://keybase.io/flurdy) for encrypted communication

**Include detailed information** about the vulnerability:
- Description of the issue
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### What to Expect

- **Initial response** within 48 hours
- **Regular updates** on the status of the issue
- **Credit** in the security advisory (if desired)
- **Coordination** for responsible disclosure

### Responsible Disclosure

We follow responsible disclosure practices:

1. **Private reporting** - Report vulnerabilities privately first
2. **Timeline** - We aim to fix critical issues within 30 days
3. **Coordination** - We work with reporters on disclosure timing
4. **Credit** - We credit security researchers in advisories

## Security Features

### Authentication & Authorization

- **Session-based authentication** with secure cookies
- **Role-based access control** (ReadOnly, Edit, Admin)
- **Session expiration** and secure session management
- **Password hashing** using bcrypt

### Data Protection

- **Input validation** and sanitization
- **SQL injection prevention** through parameterized queries
- **XSS protection** through proper output encoding
- **CSRF protection** for state-changing operations

### Database Security

- **Connection encryption** for database connections
- **Prepared statements** for all database queries
- **Access control** at the application level
- **Audit logging** for sensitive operations

### Network Security

- **HTTPS enforcement** in production
- **Secure headers** implementation
- **Content Security Policy** (CSP) headers
- **Rate limiting** for API endpoints

## Security Best Practices

### For Users

1. **Keep updated** - Always use the latest stable version
2. **Secure deployment** - Use HTTPS in production
3. **Strong passwords** - Use complex passwords for admin accounts
4. **Regular backups** - Maintain secure backups of your data
5. **Access control** - Limit access to authorized personnel only

### For Developers

1. **Security reviews** - All code changes undergo security review
2. **Dependency scanning** - Regular vulnerability scans
3. **Testing** - Comprehensive security testing
4. **Documentation** - Security considerations documented

## Security Advisories

Security advisories are published on:

- **GitHub Security Advisories**: [https://github.com/flurdy/sortingoffice/security/advisories](https://github.com/flurdy/sortingoffice/security/advisories)
- **Project website**: [Coming soon]
- **Email notifications**: For critical vulnerabilities

## Security Team

Our security team can be reached at:

- **GitHub Security Advisories**: [github.com/flurdy/sortingoffice/security/advisories](https://github.com/flurdy/sortingoffice/security/advisories)
- **Contact Form**: [ltrbx.io/flurdy.com](https://ltrbx.io/flurdy.com)
- **PGP Keys**: [keybase.io/flurdy](https://keybase.io/flurdy)

## Bug Bounty

We currently do not have a formal bug bounty program, but we do offer:

- **Recognition** in security advisories
- **Swag** for significant security contributions
- **Credit** in release notes

## Compliance

Sorting Office is designed with security in mind and can be deployed in environments requiring:

- **GDPR compliance** - Data protection and privacy features
- **SOC 2** - Security controls and audit trails
- **ISO 27001** - Information security management

## Security Checklist

Before deploying Sorting Office in production:

- [ ] Use HTTPS with valid certificates
- [ ] Configure secure session settings
- [ ] Set up proper firewall rules
- [ ] Enable database encryption
- [ ] Configure secure backup procedures
- [ ] Set up monitoring and alerting
- [ ] Review and customize access controls
- [ ] Test security features thoroughly

## Security Resources

- **OWASP Top 10**: We follow OWASP security guidelines
- **Rust Security**: We follow Rust security best practices
- **Database Security**: We implement database security best practices

---

*Security is a shared responsibility. Thank you for helping keep Sorting Office secure!* 
