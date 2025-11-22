# Meeru Email Client - MVP Roadmap

A comprehensive roadmap for building a Superhuman-inspired email client using Tauri v2, SvelteKit, and TypeScript.

## Phase 1: Core Email Infrastructure
**Milestone: Basic Email Connectivity**

- [ ] IMAP/SMTP client implementation in Rust
  - IMAP connection and authentication (OAuth2 + password support)
  - Email fetching and syncing
  - SMTP sending capabilities
  - Mailbox/folder structure parsing
- [ ] Local database/cache layer
  - SQLite schema for emails, contacts, threads
  - Indexing for fast search
  - Offline support and sync queue
- [ ] Account management
  - Multi-account support
  - Credential storage (OS keychain integration via Tauri)

## Phase 2: Email Reading & Thread View
**Milestone: Superhuman-style Reading Experience**

- [ ] Thread-based conversation view
  - Group emails by conversation ID
  - Collapsible message cards
  - Smart thread splitting
- [ ] Message rendering
  - HTML email rendering (sanitized)
  - Inline image loading
  - Attachment handling and previews
  - Dark mode support
- [ ] Split pane layout
  - Left: Email list (compact, information-dense)
  - Right: Email detail view
  - Responsive resizing

## Phase 3: Inbox Management (The "Superhuman" Part)
**Milestone: Lightning-Fast Triage**

- [ ] Keyboard-first navigation
  - Vim-style keybindings (j/k for navigation)
  - Single-key actions (e: archive, #: delete, etc.)
  - Command palette (Cmd+K) for all actions
  - Quick reply (r), reply-all (a), forward (f)
- [ ] Inbox Zero workflows
  - Done/Archive (swipe or keyboard)
  - Snooze with smart presets (later today, tomorrow, next week)
  - Remind me if no reply
  - Mark as spam/unsubscribe
- [ ] Email categorization
  - Auto-categorize (Primary, Social, Promotions, etc.)
  - Custom labels/tags
  - Starred/important markers
  - Priority inbox view

## Phase 4: Composition & Sending
**Milestone: Beautiful Email Composer**

- [ ] Rich text editor
  - Markdown support with preview
  - Formatting toolbar (minimal, keyboard-accessible)
  - @ mentions and autocomplete
  - Emoji picker
- [ ] Smart composition features
  - Contact autocomplete with fuzzy search
  - Attachment drag-and-drop
  - Inline image pasting
  - Email templates/snippets
  - Scheduled sending
  - Send later
- [ ] Composer UX
  - Modal overlay design (like Superhuman)
  - Multiple drafts management
  - Auto-save drafts

## Phase 5: Search & Filters
**Milestone: Instant Search**

- [ ] Full-text search engine
  - Index all email content
  - Search-as-you-type (sub-100ms)
  - Advanced search operators (from:, to:, subject:, has:attachment)
  - Search within threads
- [ ] Smart filters
  - Unread filter
  - Starred filter
  - Has attachments
  - Date ranges
  - Custom saved searches

## Phase 6: Superhuman Polish
**Milestone: Premium UX Details**

- [ ] Animations & transitions
  - Smooth list scrolling (60fps)
  - Swipe gestures for actions
  - Fade transitions between views
  - Loading states and skeletons
- [ ] Speed optimizations
  - Virtual scrolling for large email lists
  - Lazy loading of email content
  - Prefetching next email
  - Progressive image loading
- [ ] Visual design
  - Clean, minimal interface
  - Beautiful typography (system fonts)
  - Subtle shadows and depth
  - Status indicators (read/unread, sent, etc.)
  - Avatar generation for contacts

## Phase 7: Advanced Features
**Milestone: Power User Tools**

- [ ] Contact management
  - Smart contact extraction
  - Contact profiles with email history
  - VIP/important contacts
- [ ] Notifications
  - Desktop notifications (Tauri)
  - Smart notification filtering
  - Badge counts
  - Notification sounds
- [ ] Email tracking (optional)
  - Read receipts
  - Link tracking
  - Analytics dashboard
- [ ] AI features
  - Smart reply suggestions
  - Email summarization
  - Auto-categorization improvements
  - Spam detection

## Phase 8: Cross-Platform & Sync
**Milestone: Unified Experience**

- [ ] Cross-device sync
  - Read/unread state sync
  - Draft sync
  - Settings sync
  - Custom filters/labels sync
- [ ] Platform-specific optimizations
  - macOS: Menu bar integration
  - Windows: System tray
  - Linux: Desktop environment integration

---

## Technical Implementation Priorities

### For Meeru's Stack (Tauri + SvelteKit)

#### 1. Start with Rust backend (`src-tauri/`)
- Email protocol implementation (use `async-imap`, `lettre` crates)
- Database layer (use `sqlx` with SQLite)
- Expose Tauri commands for frontend

#### 2. Frontend structure (`src/routes/`)
- `/` - Inbox view (split pane)
- `/compose` - Email composer modal
- `/settings` - Account & app settings
- `/search` - Search results view

#### 3. State management
- Svelte 5 runes for reactive state
- Consider Tauri events for real-time sync updates

#### 4. Keyboard shortcuts
- Global keyboard handler in root layout
- Command palette component

---

## MVP Focus (Phases 1-3)

For the initial MVP, prioritize:
1. **Phase 1**: Get basic email connectivity working
2. **Phase 2**: Build the core reading experience
3. **Phase 3**: Implement keyboard navigation and inbox management

This creates a functional, fast email client that captures the essence of Superhuman's approach.
