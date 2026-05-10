# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| latest | ✅ |
| < latest | ❌ |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report vulnerabilities privately to: **matheus.marnt@gmail.com**

Include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

You will receive a response within **72 hours**. If the vulnerability is confirmed, a fix will be prioritized and a new release published. You will be credited in the release notes unless you prefer to remain anonymous.

## M8 Sandbox — Additional Notes

The Sandbox module (M8) intentionally exposes local projects to the internet. Security controls include mandatory session expiration, optional password protection, IP allowlist, and read-only mode. If you discover a bypass for any of these controls, treat it as a high-severity vulnerability and report it privately.

The bundled `cloudflared` binary is verified against its SHA-256 hash on every execution. If you discover a supply chain issue with this binary, report it immediately.
