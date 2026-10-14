# Security Policy

## Reporting a vulnerability

Please do not report security vulnerabilities in public issues.

Use GitHub private vulnerability reporting: open the repository's **Security** tab and choose **Report a vulnerability**.

Include a description of the problem, steps to reproduce, the affected version and platform, and the potential impact if known.

## Supported versions

Only the latest release receives security fixes.

## Scope

Aether can load third-party plugins that run code on your machine. Plugins are not sandboxed beyond the permissions the app grants them, so install only plugins you trust. Vulnerabilities in the app's plugin host or permission handling are in scope; malicious behavior of a third-party plugin should be reported to that plugin's author.
