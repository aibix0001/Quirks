# Weekly Development Meeting Schedule

**Recurring:** **Tuesdays, 19:00 UTC** (flexible if needed)
**Duration:** 45-60 minutes
**Location:** Discord #bot-talk
**Attendees:** aibotix, Egon, aibix (optional stakeholder)

---

## Meeting Format

### Agenda (20 min)
- **What shipped last week?** (aibotix: features, Egon: reviews/tests)
- **What's blocking?** (discuss issues, dependencies)
- **What's next?** (sprint planning for upcoming week)

### Deep Dive (20 min)
- **Technical discussion** (architecture, design decisions)
- **Code review** (discuss PRs, quality standards)
- **Performance** (benchmarks, optimization updates)

### Planning (10-15 min)
- **Task breakdown** (assign work for coming week)
- **Priorities** (what's critical vs. nice-to-have)
- **Deadlines** (release dates, milestones)

### Close (5 min)
- **Action items** (who does what by when)
- **Next week's topic** (if known)
- **Document meeting notes** (MEETING-XXX.md)

---

## Meeting Topics by Phase

### Phase 1: v0.3 Release (Weeks 1-2)
**Meeting 001 - Feb 4:**
- Status: Release binary build, human testing prep
- Decision: Ship v0.3 now or add more features?
- Next: Binary build, GitHub release, feedback collection

**Meeting 002 - Feb 11:**
- Feedback from testers
- Critical bugs (if any)
- v0.3.x patch planning

---

### Phase 2: Plugin System (Weeks 3-5)
**Meetings 003-005 - Feb 18, Feb 25, Mar 4:**
- [ ] Lua API design
- [ ] Hook system architecture
- [ ] Example plugin development
- [ ] Plugin discovery & loading

---

### Phase 3: LSP Foundation (Weeks 4-6)
**Meetings 004-006 - Feb 25, Mar 4, Mar 11:**
- [ ] LSP protocol implementation
- [ ] Server discovery strategy
- [ ] Diagnostic rendering
- [ ] Language support roadmap

---

### Phase 4: Performance (Weeks 5-7)
**Meetings 005-007 - Mar 4, Mar 11, Mar 18:**
- [ ] Performance profiling results
- [ ] Lazy rendering implementation
- [ ] Syntax caching strategy
- [ ] Benchmark review (1MB+ files)

---

### Phase 5: Customization & Theming (Weeks 6-8)
**Meetings 006-008 - Mar 11, Mar 18, Mar 25:**
- [ ] Keybinding remapping design
- [ ] Theme system architecture
- [ ] Config file format
- [ ] User feedback on UX

---

### Phase 6: v0.4 Release (Weeks 7-10)
**Meetings 007-010 - Mar 18, Mar 25, Apr 1, Apr 8:**
- [ ] Integration testing results
- [ ] Release candidate planning
- [ ] Documentation review
- [ ] Announcement & marketing

---

## Meeting Notes Template

Each meeting gets documented in `Plan/MEETING-XXX.md`:

```markdown
# Meeting #XXX — [Title]

**Date:** YYYY-MM-DD
**Attendees:** aibotix, Egon, aibix (optional)

## Executive Summary
[One paragraph recap of decisions]

## Agenda Items

### 1. What Shipped
- [Feature A] — aibotix
- [Test suite B] — Egon

### 2. Blockers
- [Issue X] — Status, plan to fix

### 3. Decisions Made
- [Decision A] — Rationale
- [Decision B] — Rationale

## Action Items
- [ ] Task 1 — Owner — Due date
- [ ] Task 2 — Owner — Due date

## Next Week's Focus
[What will be discussed/worked on]

## Links
- Related PRs
- GitHub issues
- Design docs
```

---

## Async Communication (Between Meetings)

**Daily (in Discord #bot-talk):**
- Commit summaries (automated via git)
- Quick blockers ("I'm stuck on X, need advice")
- Celebration of shipped features

**Weekly (before Tuesday meeting):**
- aibotix: Share what's been done, what's planned
- Egon: PR review status, test results
- Any agenda items for Tuesday discussion

**GitHub Issues:**
- Design discussions (features, architecture)
- Bug reports
- Feature requests

---

## Special Meetings (As Needed)

**Emergency (blocker/critical bug):**
- Call immediately when needed
- Quick fix + plan to prevent recurrence

**Architecture Review (every 2 weeks):**
- Deep dive into major design decisions
- Code quality, refactoring needs
- Performance impact assessment

**User Feedback Session (monthly):**
- Collect and discuss community feedback
- Prioritize feature requests
- Plan next phase based on user needs

---

## Current Schedule (Start: Feb 4, 2026)

| Meeting | Date | Phase | Topic |
|---------|------|-------|-------|
| 001 | Feb 4 @ 19:00 | v0.3 Release | Release binary, testing prep |
| 002 | Feb 11 @ 19:00 | v0.3 Feedback | Tester feedback, v0.3.x patches |
| 003 | Feb 18 @ 19:00 | Plugin System | Lua API design |
| 004 | Feb 25 @ 19:00 | Plugin + LSP | Both systems architecture |
| 005 | Mar 4 @ 19:00 | Performance | Profiling & optimization |
| 006 | Mar 11 @ 19:00 | Customization | Theming, keybindings |
| 007 | Mar 18 @ 19:00 | Integration | Testing, RC1 planning |
| 008 | Mar 25 @ 19:00 | Release Prep | Release candidate, docs |
| 009 | Apr 1 @ 19:00 | v0.4 Release | v0.4 ship date, feedback |
| 010 | Apr 8 @ 19:00 | v0.5 Planning | Next major version roadmap |

---

## Ground Rules

1. **Start on time.** Respect everyone's schedule.
2. **Come prepared.** Share what you've done before the meeting.
3. **Be async-friendly.** If someone can't attend, they read notes and add feedback.
4. **Decisions matter.** Record them in meeting notes.
5. **Action items.** Owner, clear deadline, track until done.
6. **Follow up.** Check completed actions at next meeting.

---

## Escalation Path

**If a decision can't be made in meeting:**
1. Get stakeholder (aibix) input
2. Schedule follow-up discussion
3. Make decision by Thursday (3 days max)
4. Communicate decision to team

**If a blocker prevents progress:**
1. Raise immediately (don't wait for Tuesday)
2. aibotix & Egon pair on solution
3. Resume when unblocked

---

**Meetings start at 19:00 UTC. Calendar invite will be posted.**

**Current timezone:** 19:00 UTC = 20:00 CET = 14:00 EST

**Next meeting:** Feb 4 @ 19:00 UTC (same day!)

Let's ship Quirks. 🚀
