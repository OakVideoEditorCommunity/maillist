# Manual Test Plan

## Objective

This document outlines the manual testing required to validate areas that automated tests cannot practically cover, including real browser authenticator flows, live SMTP transactions, third-party AI API integrations, and complex end-to-end user journeys.

## Areas Not Covered by Automated Tests

| Module | Auto Coverage | Why Manual Testing is Needed |
|--------|--------------|------------------------------|
| `src/smtp/processor.rs` | 0% | Requires real SMTP client/server interaction |
| `src/smtp/server.rs` | 0% | Requires TCP socket-level email injection |
| `src/ai/client.rs` | Partial | Requires live Aliyun API credentials and real content |
| `src/api/v1/admin.rs` | 8% | Admin dashboard UI interactions, visual validation |
| `src/api/v1/lists.rs` | 24% | Complex multi-step subscriber workflows |
| `src/services/notification_service.rs` | 0% | Requires real SMTP relay and template rendering |
| WebAuthn finish paths | N/A | Requires hardware/software authenticator (browser) |
| Frontend E2E | N/A | Vitest only tests components in isolation |

---

## Test Environment Setup

### Minimum Setup

```bash
# 1. Start backend with production-like config
cargo run --release

# 2. Start frontend dev server
cd frontend && npm run dev

# 3. Or use the built frontend served by backend
# Ensure frontend/dist/ exists after `npm run build`
```

### Required External Services

| Service | Purpose | How to Mock/Fake |
|---------|---------|------------------|
| SMTP Outgoing | Send notification emails | MailHog, Mailpit, or real SMTP relay |
| SMTP Incoming | Receive list emails | `swaks`, `telnet`, or real MX record |
| Aliyun Green | AI content moderation | Must use real credentials (controlled test content) |
| PostgreSQL | Production database | Docker: `postgres:16-alpine` |

### Test Data Preparation

1. Create 3+ domains (`example.com`, `test.org`, `demo.net`)
2. Create 5+ mailing lists with varying policies (open, confirm, subscriber-only)
3. Seed 100+ subscribers across lists using CSV import
4. Prepare test email samples: plain text, HTML, multipart, with attachments

---

## Test Categories

### Category 1: Passkey / WebAuthn (Critical)

**Rationale**: Automated tests can mock the challenge/response start, but the browser's `navigator.credentials.create()` and `get()` calls require a real authenticator.

#### 1.1 Passkey Registration

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Log in with password, go to Profile → Passkey | "Register Passkey" button visible |
| 2 | Click "Register Passkey", enter device name | Browser shows system Passkey prompt |
| 3 | Authenticate with system dialog (TouchID/FaceID/Windows Hello) | Success toast, Passkey appears in list |
| 4 | Log out | Session cleared |
| 5 | Log in using "Sign in with Passkey" | Browser prompt → authenticated directly |
| 6 | Repeat on same browser | Should allow multiple Passkeys |

**Browsers to test**: Safari (macOS/iOS), Chrome (Windows/macOS), Firefox, Edge

#### 1.2 Cross-Device Passkey Login

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Register Passkey on Laptop A | Success |
| 2 | On Phone B, go to login page | "Sign in with Passkey" available |
| 3 | Choose "Use a different device" if prompted | QR code or nearby device option |
| 4 | Complete authentication | Logged in, JWT tokens issued |

#### 1.3 Passkey Revocation

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Register 2 Passkeys | Both listed |
| 2 | Delete Passkey #1 | Removed, Passkey #2 still works |
| 3 | Delete last Passkey | Account falls back to password-only |

#### 1.4 WebAuthn RP ID Validation

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Set `webauthn_rp_id` to `mail.example.com` | Registration works on `mail.example.com` |
| 2 | Try to register from `localhost` | Browser rejects (RP ID mismatch) |
| 3 | Try from `sub.mail.example.com` | Works (subdomain matches) |

---

### Category 2: SMTP End-to-End (Critical)

**Rationale**: The SMTP server accepts TCP connections and parses raw RFC 822 bytes. This cannot be fully validated without real email clients or MTAs.

#### 2.1 Incoming SMTP — Basic Delivery

**Tool**: `swaks` (Swiss Army Knife for SMTP)

```bash
# Test 1: Basic delivery to an open list
swaks --to test-list@example.com \
      --from subscriber@example.com \
      --server localhost:2525 \
      --header "Subject: Hello from swaks" \
      --body "This is a test email body."
```

| Scenario | Expected Result |
|----------|-----------------|
| Valid subscriber sends to list | Email archived, delivered to all active subscribers |
| Non-subscriber sends to subscriber-only list | Email rejected or enters moderation queue |
| Email to non-existent list | SMTP 550 error or silently dropped (check logs) |
| Empty body email | Archived with empty body_text |
| Subject-only email | Archived, body empty |

#### 2.2 Incoming SMTP — MIME & Attachments

```bash
# Test with attachment
swaks --to test-list@example.com \
      --from subscriber@example.com \
      --server localhost:2525 \
      --attach /path/to/test.pdf \
      --header "Subject: Email with attachment"
```

| Scenario | Expected Result |
|----------|-----------------|
| PDF attachment | Attachment saved, checksum computed, downloadable via API |
| Image attachment (PNG/JPG) | Content-Type preserved, preview possible |
| Multiple attachments | All saved, individually accessible |
| Attachment > max size | Rejected or truncated based on config |

#### 2.3 Incoming SMTP — Multipart

```bash
# Create a multipart email manually or use swaks with --data
```

| Scenario | Expected Result |
|----------|-----------------|
| multipart/alternative (text + HTML) | Both body_text and body_html extracted |
| multipart/mixed (body + attachment) | Body extracted, attachments saved separately |
| multipart/related (HTML + inline images) | Inline images handled as attachments with content-id |
| Malformed MIME | Graceful fallback, raw content preserved |

#### 2.4 Incoming SMTP — Threading

| Scenario | Expected Result |
|----------|-----------------|
| Reply with matching In-Reply-To | Thread ID assigned, appears in thread list |
| Reply with matching References | Thread ID assigned |
| New email (no In-Reply-To) | New thread ID generated |
| Broken Message-Id references | Treated as new thread |

#### 2.5 Outbound SMTP — Notification Delivery

**Setup**: Configure outgoing SMTP to MailHog or real relay.

| Scenario | Expected Result |
|----------|-----------------|
| User subscribes to list | Welcome email sent within 5 seconds |
| User unsubscribes | Unsubscribe confirmation email sent |
| Message held for moderation | Moderation notice sent to list moderators |
| Digest enabled subscriber | Digest email sent at scheduled interval |
| SMTP server unreachable | Error logged, message queued for retry (if implemented) |

#### 2.6 VERP Bounce Handling

| Scenario | Expected Result |
|----------|-----------------|
| Send to invalid address | Bounce received at `list-bounces+token@domain` |
| Hard bounce | Bounce log created, subscriber bounce_count incremented |
| 3rd hard bounce | Subscriber auto-unsubscribed |
| Soft bounce | Bounce log created, subscriber status unchanged |

---

### Category 3: AI Content Moderation (High)

**Rationale**: The Aliyun API requires real credentials and returns non-deterministic scores. Automated tests mock the client, so real API behavior is untested.

#### 3.1 AI Flagging Flow

**Prerequisite**: Enable `ai_moderation_enabled = true` on a list.

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Send clean email to list | Archived normally, no moderation entry |
| 2 | Send mildly suspicious email | May be flagged based on threshold |
| 3 | Send clearly spam/offensive email | Enters moderation queue with `source = "ai_flagged"` |
| 4 | Check moderation queue via API/UI | Shows AI risk score, labels, raw response |
| 5 | Approve AI-flagged message | Message archived and delivered |
| 6 | Reject AI-flagged message | Message not archived, optional note saved |

#### 3.2 AI Risk Score Accuracy

Prepare test emails with varying content:

| Content Type | Expected AI Score Range |
|--------------|------------------------|
| "Meeting minutes from last week" | 0-20 (clean) |
| "Buy cheap products now!!!" | 60-90 (caution/flagged) |
| "Click this link for free money" | 80-100 (flagged) |
| Non-ASCII Chinese spam | Should still be detected |
| Very long text (> max_text_length) | Truncated before sending to API |

#### 3.3 AI False Positive Feedback

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Flag a clean email incorrectly | Email enters moderation queue |
| 2 | Submit false-positive feedback | API returns success |
| 3 | Check AI moderation stats | Stats endpoint reflects feedback count |

---

### Category 4: Frontend End-to-End (High)

**Rationale**: Vitest tests components in isolation. Real user flows involve page navigation, state persistence, and browser-specific behavior.

#### 4.1 Complete User Journey — New Admin

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Open `/register` | Form renders, CookieConsent banner visible |
| 2 | Register with email + password | Redirect to login or dashboard |
| 3 | Log in | Dashboard loads, sidebar shows admin links |
| 4 | Create domain | Domain appears in domain list |
| 5 | Create mailing list | List appears, email address shown |
| 6 | Invite subscriber via UI | Subscriber appears with "pending" status |
| 7 | Confirm subscription (click email link or API) | Status changes to "active" |
| 8 | Send test email via SMTP to list | Email appears in archive |
| 9 | View archive | Message body, headers, attachments visible |
| 10 | Moderate message | Approve/reject buttons work |

#### 4.2 Language Switching

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Open site with browser locale `zh-CN` | UI renders in Chinese |
| 2 | Switch language to English via dropdown | All text changes instantly |
| 3 | Log in | User's saved language overrides browser locale |
| 4 | Change language in profile | Persists after logout/login |
| 5 | Reject cookies | Cookie banner hidden, site still functional |

#### 4.3 Mobile Responsiveness

| Device | Viewport | Checks |
|--------|----------|--------|
| iPhone 14 Pro | 393×852 | Sidebar collapses to hamburger, tables scroll horizontally |
| iPad Pro | 1024×1366 | Sidebar visible, two-column layouts |
| Android (Chrome) | 360×800 | Touch targets ≥ 44px, no horizontal overflow |

#### 4.4 File Upload & Download

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Export subscribers as CSV | File downloads, UTF-8 encoded |
| 2 | Import subscribers via CSV | Preview shows, duplicates handled |
| 3 | Download attachment from archive | Original file, checksum matches |
| 4 | Download raw email (RFC 822) | `.eml` file opens in mail client |

---

### Category 5: Security & Authorization (High)

#### 5.1 JWT Token Lifecycle

| Scenario | Action | Expected Result |
|----------|--------|-----------------|
| Valid token | Call `/users/me` with Bearer token | 200, user data returned |
| Expired token | Wait 15+ minutes, call again | 401 Unauthorized |
| Invalid signature | Modify token payload | 401 Unauthorized |
| Missing token | Call protected endpoint | 401 Unauthorized |
| Refresh token reuse | Use refresh token twice | Second use rejected, all tokens revoked |
| Global logout | Call `/auth/logout-all` | All sessions invalidated |

#### 5.2 Role-Based Access Control

| Role | Action | Expected Result |
|------|--------|-----------------|
| Regular user | Access `/admin/dashboard` | 403 Forbidden |
| Regular user | Delete another user | 403 Forbidden |
| Site admin | Access `/admin/dashboard` | 200, stats visible |
| Site admin | Delete any user | 200, user deactivated |
| List owner | Moderate their list | 200 |
| List owner | Moderate another's list | 403 Forbidden |

#### 5.3 Input Validation & Injection

| Input | Endpoint | Expected Result |
|-------|----------|-----------------|
| `"<script>alert(1)</script>"` in name | `/users/me` (PUT) | Script sanitized or rejected |
| `../etc/passwd` in filename | Attachment download | Path traversal blocked |
| 10MB attachment | SMTP DATA | Rejected if > max size |
| 10,000 character subject | SMTP DATA | Truncated or stored safely |
| SQL injection in search | `/lists/{id}/search` | Parameterized query prevents injection |
| Unicode homoglyphs in email | `/auth/register` | Normalized and validated |

#### 5.4 Rate Limiting & Brute Force

| Scenario | Action | Expected Result |
|----------|--------|-----------------|
| Login attempts | 10 failed logins in 1 minute | Account temporarily locked or rate-limited |
| Registration spam | 100 registrations from same IP | Rate limited or CAPTCHA triggered |
| Password guess | Common passwords ("password123") | Rejected by minimum length/strength rules |

---

### Category 6: Performance & Scale (Medium)

#### 6.1 Large List Performance

| Metric | Test Method | Acceptable Threshold |
|--------|-------------|----------------------|
| Subscriber list (10,000) | Open `/lists/{id}/subscribers` | Page loads < 2s |
| Message archive (50,000) | Open `/lists/{id}/messages` | Page loads < 3s |
| Search across archive | Query with broad keyword | Results in < 2s |
| CSV export (10,000 subs) | Click export | Download starts < 5s |
| Bulk import (5,000 subs) | Upload CSV | Processing completes < 30s |

#### 6.2 Concurrent Operations

| Scenario | Action | Expected Result |
|----------|--------|-----------------|
| 10 users subscribe simultaneously | All call `/lists/{id}/subscribe` | All succeed, no duplicate entries |
| 2 admins edit same template | Simultaneous PUT to `/templates/{name}` | Last write wins or conflict detected |
| Digest generation during heavy load | Trigger digest while receiving emails | No deadlock, queue processed |

#### 6.3 Memory & Resource Usage

| Scenario | Monitoring | Acceptable Threshold |
|----------|------------|----------------------|
| Run for 24 hours | Check RSS memory | Stable, no leak > 50MB growth |
| Receive 1,000 emails/hour | Check CPU | < 50% on 2-core VM |
| Database connections | Check `pg_stat_activity` | Never exceeds `max_connections` |

---

### Category 7: Backup, Recovery & Migration (Medium)

| Scenario | Action | Expected Result |
|----------|--------|-----------------|
| Database backup | `pg_dump` while running | Consistent dump, no locking issues |
| Restore from backup | Restore to new DB, restart app | All data intact, app starts normally |
| Migration rollback | Run `sea-orm-cli migrate down` | Schema reverts, data preserved |
| Zero-downtime deploy | Blue/green with shared DB | No dropped requests |

---

### Category 8: Disaster Scenarios (Medium)

| Scenario | Action | Expected Result |
|----------|--------|-----------------|
| Database connection loss | Stop PostgreSQL | App logs errors, retries connection |
| SMTP relay down | Block outbound port 587 | Emails queue or fail gracefully |
| Disk full | Fill archive partition | New emails rejected, clear error logged |
| JWT secret rotation | Change secret, old tokens present | Old tokens rejected, users re-login |
| Frontend build missing | Delete `frontend/dist/` | Backend API still works, root path 404s |

---

## Test Execution Schedule

| Phase | When | Scope | Duration |
|-------|------|-------|----------|
| Smoke Test | After every deploy | Login, create list, send email | 15 min |
| Passkey Regression | Weekly | All browsers, registration + login | 30 min |
| SMTP E2E | Weekly | Send/receive 20 test emails | 45 min |
| AI Moderation | Monthly | 10 test contents across risk levels | 30 min |
| Security Audit | Monthly | RBAC, injection, token lifecycle | 60 min |
| Performance | Before major release | 10K subscribers, 50K messages | 2 hours |
| Full Regression | Before release | All categories above | 1 day |

---

## Sign-Off Checklist

Before declaring a release ready:

- [ ] Passkey registration works on Safari, Chrome, Firefox
- [ ] Passkey login works on at least 2 different devices
- [ ] SMTP server receives and archives 5 test emails successfully
- [ ] Outbound notifications deliver to MailHog/real inbox
- [ ] AI moderation flags at least 1 test spam message
- [ ] Admin dashboard loads with real data
- [ ] CSV import/export round-trip preserves all subscribers
- [ ] JWT expiration and refresh work correctly
- [ ] Mobile UI usable on iPhone and Android
- [ ] No console errors or 500 responses during normal usage
