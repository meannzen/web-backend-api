# TPA Platform — Product Overview

**Product:** TPA Platform — Teacher Performance Appraisal SaaS
**Market:** Ontario school boards
**Built by:** ZINation
**Last updated:** September 23, 2026

> This document pulls together what's in the TPA-BOS workspace: the TPA requirements set (extracted from the Ministry's TPA Technical Requirements Manual), the role and permissions mapping, System Architecture v3.4, the NFR and integration decisions, the UI screen inventory, the risks and open questions log, the roadmap, the presentation talking points and the v3 prototypes. It's a single overview. For detailed rules, the source documents are the reference (see [Sources](#11-sources)).

---

## Contents

1. [What the TPA Platform is](#1-what-the-tpa-platform-is)
2. [Why it exists](#2-why-it-exists)
3. [Scope and phases](#3-scope-and-phases)
4. [Roles](#4-roles)
5. [Core use cases](#5-core-use-cases)
6. [Functional requirements by portal](#6-functional-requirements-by-portal)
7. [Business rules](#7-business-rules)
8. [Integrations and technical environment](#8-integrations-and-technical-environment)
9. [Non-functional requirements](#9-non-functional-requirements)
10. [Open items](#10-open-items)
11. [Sources](#11-sources)

---

## 1. What the TPA Platform is

The TPA Platform digitizes Ontario's **Teacher Performance Appraisal** process from end to end. Every appraisal is a *ticket* that moves through the steps the Ministry requires:

1. The **principal** schedules the appraisal and notifies the teacher.
2. The principal and teacher hold a **pre-observation meeting** and agree on the focus competencies.
3. The principal does the **classroom observation**.
4. They hold a **post-observation meeting** to review the results.
5. The principal writes and signs the **summative report** with a rating. The teacher signs to acknowledge it.
6. The rating triggers the right **next steps**: the next appraisal, an Enrichment or Improvement Plan, Review Status, or in rare cases Termination.

Teachers can see where they are at every step. Experienced teachers also keep their **Annual Learning Plan (ALP)** in the platform, and the **school board** oversees compliance across all its schools.

**Main features**

- Guided creation of each appraisal (type, rating scale and deadlines set automatically)
- The four mandatory appraisal steps, each with its ministry form
- Automatic deadline tracking in school days
- NTIP tracking for new teachers, including OCT notification
- 5-year evaluation cycle scheduling for experienced teachers
- Annual Learning Plans with digital signatures
- Enrichment Plans, Improvement Plans, Review Status and Termination workflows
- Board-wide compliance dashboards and Ministry export
- 6-year document retention with audit trail
- Fully bilingual (English / French), AODA accessible, responsive web

---

## 2. Why it exists

### 2.1 The problem

TPA is mandatory and defined by the Ministry, but most school boards still run it on **paper forms, email and spreadsheets**:

- **Principals** have trouble tracking stages and deadlines across their staff.
- **Teachers** can't see their own appraisal progress.
- **Boards** lack the dashboards they need to prove they follow Ministry rules.

The result is extra admin work, compliance risk, and a process that creates friction instead of supporting growth.

### 2.2 Purpose of TPA (from the Ministry)

1. Promote teacher development
2. Provide meaningful appraisals that encourage professional learning and growth
3. Identify where teachers need extra support
4. Provide accountability to the public

### 2.3 Legal basis

| Source | Covers |
|---|---|
| Education Act, Part X.2 | Teacher Performance Appraisal (primary legislation) |
| O. Reg. 98/02 | Appraisal requirements for experienced teachers |
| O. Reg. 99/02 | Appraisal requirements for new teachers; the competency statements |
| O. Reg. 266/06 | Additional provisions |
| Education Act, s. 277.17–277.40 | Delegation, timelines, plans |
| Education Act, s. 272 | NTIP completion notification to the Ontario College of Teachers (OCT) |

Every business rule in the platform must trace back to one of these sources. The full mapping is in `15-traceability-matrix.md`.

### 2.4 Positioning

The TPA Platform is built for Ontario school boards. It follows the Ministry TPA workflow from day one instead of adapting a generic HR tool, and it frames the appraisal as professional growth, not just an evaluation.

---

## 3. Scope and phases

### 3.1 Portals

Four role-based portals share one responsive web app:

| Portal | Screens | Purpose |
|---|---|---|
| **Principal** | 17 | Create and run appraisals, sign reports, manage ALPs, Review Status and Termination |
| **New Teacher (NTIP)** | 8 | Follow NTIP progress, appraisals, meetings, plans and resources |
| **Experienced Teacher** | 9 | 5-year cycle status, the ALP, history, documents and PD |
| **Board Admin** (Technical Administrator / HR) | 15 | Board-wide compliance, NTIP and cycle tracking, users, schools, configuration, Ministry export |

### 3.2 Out of scope (from the NFR decisions)

- Native iOS / Android apps (responsive web instead)
- A full form builder (the TPA forms are fixed by the Ministry)
- Importing user identities from CSV (identities come through SSO only)

### 3.3 Roadmap

Early estimates from the TPA SaaS Estimation Draft v3. Exact dates are still to be confirmed.

| Phase | Scope | Estimated timing |
|---|---|---|
| **1 — Discovery & Architecture** | Entity model, state machines, architecture, UI/UX requirements, environments | Q1 2026 (Mar–Apr) |
| **2 — UI/UX Design** | 25+ responsive screens, interactive prototypes, usability and accessibility checks | Q2 2026 (Apr–May) |
| **3 — Core Development** | Ticket engine and APIs, Principal portal, Teacher portals, RBAC and SSO, scheduling and notifications | Q2–Q3 2026 (May–Sep) |
| **4 — Integrations, Analytics & Security** | Integration layer (SSO, PDF forms, calendar), analytics, security hardening, pen testing, retention and audit | Q3–Q4 2026 (Sep–Nov) |
| **5 — QA, UAT & Launch** | Full QA, user acceptance testing, launch | Q4 2026 (Nov–Dec) |

**P1 release target:** September 2026. What "P1" means (internal pilot or production release) is still open. See [§10](#10-open-items).

---

## 4. Roles

### 4.1 Role summary

| Role | Access level | Portal | Scope | Main job |
|---|---|---|---|---|
| **Technical Administrator** | Level 1 — full access | Board Admin | All schools, all appraisals, system config | Run the platform: SSO, users, schools, board settings, audit logs |
| **Business Owner (HR / Board)** | Level 1 — board-wide (read) | Board Admin | All schools in the board | Oversee compliance, reports, user–school access, board policies, OCT notifications, teacher transfers |
| **Principal** | Level 2 — school | Principal | Their school | Create and run appraisals, sign summative reports, review ALPs, delegate |
| **Deputy Principal** (VP or Supervisory Officer) | Level 4 — assigned only | Principal (delegated) | Only teachers assigned to them | Run the appraisals delegated to them |
| **New Teacher** | Level 3 — own records | New Teacher (NTIP) | Own records | Take part in 2 appraisals, sign reports, follow NTIP |
| **Experienced Teacher** | Level 3 — own records | Experienced Teacher | Own records | Take part in the 5-year cycle, keep the ALP, sign reports |

**Multi-role:** one person can hold several roles. For example, a Deputy Principal who also teaches has both roles under one account.

**Not covered by TPA** (no appraisals can be created for them): occasional teachers, continuing education teachers, vice-principals, principals, supervisory officers, directors of education, and teacher-education instructors.

### 4.2 Access matrix

| Feature / data | Tech Admin | Business Owner | Principal | Deputy Principal | Teacher |
|---|:-:|:-:|:-:|:-:|:-:|
| System configuration (SSO, tenants) | ✅ | ❌ | ❌ | ❌ | ❌ |
| All schools in the board | ✅ | ✅ | ❌ | ❌ | ❌ |
| All appraisals (board-wide) | ✅ | ✅ read | ❌ | ❌ | ❌ |
| Own school's appraisals | ✅ | ✅ | ✅ | Assigned only | ❌ |
| Own appraisals | ✅ | ✅ | ✅ | ✅ | ✅ |
| Create an appraisal | ✅ | ❌ | ✅ | ✅ delegated | ❌ |
| Sign summative report | ✅ | ❌ | ✅ | ✅ delegated | ✅ acknowledge |
| ALP | ✅ | ✅ read | ✅ | ❌ | ✅ |
| User management | ✅ | ✅ own board | ❌ | ❌ | ❌ |
| Compliance reports | ✅ | ✅ | ❌ | ❌ | ❌ |
| Board configuration | ✅ | ✅ | ❌ | ❌ | ❌ |
| Delegate appraisals | ✅ | ❌ | ✅ | ❌ | ❌ |

### 4.3 Teacher types

| | New Teacher | Experienced Teacher |
|---|---|---|
| **Who** | First permanent teaching job in Ontario (including teachers trained outside the province) | Completed NTIP, held a permanent Ontario position before Sept 2006, or teaches on a Letter of Permission |
| **Appraisals** | 2 in the first 12 months | 1 every 5 years (the evaluation year) |
| **Competencies** | 8 of 16, required | All 16 considered |
| **Rating scale** | S / Development Needed first, then S / Unsatisfactory | S / Unsatisfactory |
| **ALP** | No | Yes, every year |
| **Stays in this type until** | NTIP done (2 Satisfactory ratings) or 24 months pass | — |

When a teacher completes NTIP, the platform switches them from **New to Experienced** automatically and starts their 5-year cycle.

---

## 5. Core use cases

### UC-1 — Create an appraisal (Principal)

1. The principal clicks **New Appraisal** and picks a teacher.
2. The system detects the **appraisal type** (NTIP 1st or 2nd, Experienced, additional, review), the **rating scale** and the **deadlines**.
3. The appraiser defaults to the principal. It can be **delegated** to a vice-principal or supervisory officer.
4. The principal schedules the **pre-observation meeting**, and the teacher is notified.

### UC-2 — Run the four appraisal steps (Principal + Teacher)

1. **Pre-observation meeting:** covers 8 discussion points (class context, lesson, focus competencies, logistics, student engagement, assessment, the teacher's growth focus, post-obs timing). Both people initial the form.
2. **Classroom observation:** date, time, setting type, notes against the focus competencies.
3. **Post-observation meeting:** within **5 school days** of the observation. Each competency gets feedback and the teacher can add comments.
4. **Summative report:** the Appendix A form (new teachers) or Appendix B (experienced), with competency comments, a rating and a written justification. It's due within **20 school days** of the observation, and the teacher signs to acknowledge receipt and can add comments.

Each step must be complete, with evidence, before the next one can start.

### UC-3 — New teacher completes NTIP

1. 1st appraisal: Satisfactory → the 2nd appraisal is scheduled within 12 months.
2. 2nd appraisal: Satisfactory → **NTIP complete**.
3. The board sends the **OCT notification within 60 calendar days** (the admin records the OCT certificate number and ticks the NTIP participation boxes).
4. The teacher becomes **Experienced** and enters the 5-year cycle.

### UC-4 — Development Needed (new teacher)

1. The principal gives a Development Needed rating.
2. An **Enrichment Plan** task is created, due within **15 school days**, with supportive, growth-focused content.
3. The next appraisal is due within 12 months, and its rating scale changes to Satisfactory / Unsatisfactory.

### UC-5 — Unsatisfactory rating

1. An **Improvement Plan** task is created, due within **15 school days**. It covers causes, required improvements, supports, success criteria, monitoring and consequences.
2. The next appraisal is scheduled: **60 school days** for experienced teachers, **120 school days** for new teachers.

### UC-6 — Review Status

1. Placed **automatically** after a second non-satisfactory result: Development Needed then Unsatisfactory (new teachers), or two Unsatisfactory in a row (experienced).
2. The system generates the **formal notification letter**. The teacher's acknowledgment starts the **120-school-day** clock for the next appraisal.
3. The principal holds regular meetings and gives feedback, documents support, and **consults the supervisory officer**.
4. Satisfactory → Review Status ends. Unsatisfactory → termination recommendation.

### UC-7 — Termination

1. Triggered after an Unsatisfactory while on Review Status, or through a **fast-track** when the principal and SO agree it's in the students' best interests.
2. The system prepares the **recommendation letter** filled in with the teacher's full record. The teacher receives every document and can respond in writing.
3. The teacher is suspended or reassigned while the board decides.
4. The **board votes within 60 days**: approve, reject or defer. The OCT complaint filing is tracked, including when a teacher resigns during review.

### UC-8 — Annual Learning Plan (Experienced Teacher)

1. The teacher writes the ALP: 1–5 goals, an action plan, timelines, rationale and resources.
2. The principal reviews it, comments and acknowledges it. **Both sign** digitally.
3. In the evaluation year, the ALP is linked to the appraisal and kept for 6 years.
4. In non-evaluation years, there's an **annual review** every 12 months.

### UC-9 — Board compliance monitoring (Board Admin)

- Appraisals in progress, completion rate by school, overdue items, and an upcoming-deadlines timeline
- Compliance dashboard: appraisals scheduled on time, NTIP completion, 5-year cycle adherence
- **NTIP Tracker** with OCT notification alerts and a **Send notification** action
- **ET Cycle Scheduler** that flags special cases: new to the board, back from leave, secondments
- Oversight of Review Status and Termination cases
- **Ministry export** of compliance and NTIP completion reports

### UC-10 — Special scheduling cases

- **Experienced teacher new to the board:** appraised in their first year, and the 5-year cycle starts after a Satisfactory rating.
- **Return from extended leave:** an appraisal is required within **60 school days**.
- **Qualifying absences** pause the cycle clock.
- **Secondments** keep the cycle going at the receiving board.
- **Extension** of the 24-month new-teacher period: up to 90 school days, tracked.
- **Transfers** between schools or boards: records move with the teacher (within 30 days for board transfers).

---

## 6. Functional requirements by portal

### 6.1 Principal portal (17 screens)

**Screens:** Dashboard · User Management · Appraisals · Appraisal Detail · Pre-Observation · Classroom Observation · Post-Observation · Parental & Student Input · End-of-Cycle (Summative) · Appraisal Plans · Annual Learning Plans · Review Status · Termination · Competency Standards · Forms & Documentation · Schedule · Reports

| # | Requirement |
|---|---|
| P-1 | Dashboard: new and experienced teacher counts, teachers on review, appraisals in progress, overdue actions, upcoming deadlines, teacher progress |
| P-2 | 3-step **New Appraisal** wizard that sets the type, rating scale and deadlines automatically, and allows delegation |
| P-3 | Appraisals list grouped by priority: **Action required / In progress / Completed** |
| P-4 | Complete each step's form in order, with notes, evidence uploads and competency comments |
| P-5 | Show only the valid rating options for the teacher type and appraisal number |
| P-6 | Guidance that Satisfactory is the default, and that Development Needed or Unsatisfactory needs clear evidence |
| P-7 | Create Enrichment and Improvement Plans from templates, with all required sections |
| P-8 | Review, comment on and sign teachers' ALPs |
| P-9 | Review Status case: improvement plan, 120-day countdown, SO consultation log |
| P-10 | Termination case with the required checklist |
| P-11 | Schedule meetings with calendar invites |
| P-12 | Record parental and student input (the collection method is still open) |

### 6.2 New Teacher portal — NTIP (8 screens)

**Screens:** Dashboard · My Appraisals · Schedule · Past Meetings · My Plans · PD Resources · NTIP Guidelines · Documents & Forms

| # | Requirement |
|---|---|
| N-1 | Dashboard: month of NTIP, appraisals done (e.g., 1 of 2), next meeting, competency ratings, NTIP completion tracker |
| N-2 | My Appraisals: where you are now plus step-by-step progress, and past appraisals |
| N-3 | See and sign the summative report, and add comments |
| N-4 | Access Enrichment and Improvement Plans when they apply |
| N-5 | NTIP guidelines and PD resources |
| N-6 | Request a meeting to discuss a report after it's delivered |

### 6.3 Experienced Teacher portal (9 screens)

**Screens:** Dashboard · Appraisal · History · Schedule · Meeting Notes · Annual Learning Plan · Plans · PD Resources · Documents

| # | Requirement |
|---|---|
| E-1 | Dashboard: cycle status (evaluation or non-evaluation year, year X of 5), next evaluation date, ALP progress, pending actions, recent activity |
| E-2 | Create and maintain the **ALP** (goals, reflections, PD log) and sign it |
| E-3 | History: past appraisals, ratings and summative reports to download |
| E-4 | Documents: read-only unless a form needs the teacher's signature |
| E-5 | Appraisal-related screens stay **greyed out** until the principal starts the cycle, with tooltips explaining why |
| E-6 | PD Resources is always available |

### 6.4 Board Admin portal (15 screens)

**Screens:** Appraisals Dashboard · Scheduling Status · Observation Tracker · Compliance Dashboard · Post-Obs Monitor · NTIP Tracker · Review Status · Termination Workflow · ET Cycle Scheduler · Audit Reports · TPA Completion · Ministry Export · User & Role Management · School Management · Board Configuration

| # | Requirement |
|---|---|
| A-1 | Board-wide appraisal status by school, overdue items and alerts |
| A-2 | Compliance metrics with flagged schools and principals |
| A-3 | NTIP Tracker with OCT notifications and deadlines |
| A-4 | ET Cycle Scheduler with the 5-year cycle and special cases |
| A-5 | Oversight of Review Status and Termination |
| A-6 | Audit reports, TPA completion and Ministry export |
| A-7 | Manage users, roles and schools. Appraisals **stay with the school** when the principal changes. |
| A-8 | Board configuration: cycle rules, notifications, SSO, data retention, and board-specific competencies and steps |
| A-9 | Watch rating patterns and flag unusual results by school |

---

## 7. Business rules

### 7.1 Rating scales

| Teacher | Appraisal | Options |
|---|---|---|
| New | 1st | Satisfactory / Development Needed |
| New | 2nd, after Satisfactory | Satisfactory / Development Needed |
| New | 2nd, after Development Needed | Satisfactory / Unsatisfactory |
| New | 3rd and later | Satisfactory / Unsatisfactory |
| Experienced | Any | Satisfactory / Unsatisfactory |

- A new teacher's first appraisal can **never** be rated Unsatisfactory.
- **Satisfactory is the default.** A lower rating needs clear evidence.
- The principal must write a justification for every rating.

### 7.2 Competencies

There are 16 competencies across 5 domains (from the OCT Standards of Practice). They're set by the Ministry and can't be changed by the board, although boards can **add** their own.

| Domain | Competencies | New Teacher (required) |
|---|---|---|
| Commitment to pupils and pupil learning | C01–C04 | C01, C02, C03, C04 |
| Professional knowledge | C05–C08 | C05 |
| Teaching practice | C09–C13 | C09, C10, C11 |
| Leadership in learning communities | C14–C15 | — |
| Ongoing professional learning | C16 | — |

- **New teachers:** each of the 8 required competencies must have a comment.
- **Experienced teachers:** all 16 are considered, and the pre-observation meeting picks the 4–8 to focus on.

### 7.3 Timelines

| Timeline | Duration |
|---|---|
| Post-observation meeting | 5 school days after the observation |
| Summative report delivered | **20 school days** after the observation |
| Evaluation-year notice (experienced) | 20 school days after the teacher starts in the evaluation year |
| Enrichment Plan | 15 school days after Development Needed |
| Improvement Plan | 15 school days after Unsatisfactory |
| Next appraisal after Unsatisfactory (experienced) | 60 school days |
| Next appraisal on Review Status | **120 school days** from notification |
| Appraisal after return from leave | 60 school days |
| OCT notification after NTIP completion | **60 calendar days** |
| Board vote on termination | **60 days** from the recommendation |
| New teacher period | 24 months (+ up to 90 school days' extension) |
| ALP annual review | Every 12 months |

All timelines count **school days**, so the platform needs the board calendar. If a deadline is missed, the step must be completed as soon as possible and the timelines after it recalculated.

### 7.4 Workflow rules

- The four steps must happen **in order**, and none can be marked complete without evidence.
- An appraisal **can't reach Completed** without all 7 components: competencies, observation, pre-observation meeting, post-observation meeting, summative report, rating, and post-rating steps.
- An appraiser can be **reassigned** without losing earlier steps, and the change is logged.
- Different people can complete different steps of the same appraisal (for example, when the principal is absent).
- An ALP **can't be marked complete** until both people have signed. If they disagree, the principal can finalize it, and the disagreement is documented.

### 7.5 Records

- **6-year retention**, counted from the date each summative report is delivered.
- Each summative report is kept as a **locked package** with its pre- and post-observation forms, observation notes, plans, the evaluation-year ALP, Review Status and Termination documents, and teacher comments.
- Records that come from another board are kept separately and marked **"Background Information — Confidential."**

---

## 8. Integrations and technical environment

### 8.1 Architecture (v3.4)

- **Ticket-based appraisal engine:** each appraisal is a ticket with a stage and transition rules, instead of a workflow orchestration engine.
- **App Service #1 — Ticket Engine API** (Node.js, REST): the ticket engine, notifications, reports and scheduling.
- **App Service #2 — KeystoneJS CMS** (Node.js, GraphQL): the forms library, competencies, PD catalog, board configuration and NTIP guidelines.
- **Data:** Azure SQL (main database), Azure Blob Storage (documents), Azure Synapse (analytics pipeline).
- **Front end:** one React single-page app with role-based routing (responsive, AODA, FR/EN).

### 8.2 Identity and access

- **Azure AD B2C** handles sign-in only. It connects to the board's Azure AD and other supported providers, and each deployment chooses which providers to enable.
- **Permissions (RBAC) live in the TPA platform** (Azure SQL), not in Azure AD. This covers access levels L1–L4, school scope and delegation, and every change is logged.
- Identity is managed **at the board level**: one account can belong to several schools.

### 8.3 Data and integrations

- User and school data **syncs from the client's data warehouse** on a schedule (read-only).
- Identities come **through SSO only**, with no CSV import of users (D1).
- **Manual CSV upload** is the fallback for other outside data that doesn't arrive daily (D2).
- Integration is done **through APIs** (R6), with **CSV export and import** available (R7).
- **Calendar integration** is used for meeting invites.
- There's a **Ministry export** for compliance reporting.

### 8.4 Deployment

- **Multi-deployment:** each school board gets its **own isolated deployment**. No database or infrastructure is shared between boards (R1), and each has its own DevOps pipeline (R2).
- Hosted on **Microsoft Azure**.

---

## 9. Non-functional requirements

| Area | Requirement |
|---|---|
| Devices | Responsive web app for desktop, tablet (including iPad) and phone; no native apps |
| Accessibility | AODA compliant, with accessibility testing required (R4) |
| Language | Fully bilingual English / French |
| Privacy | Data isolated per board; Ontario privacy rules (FIPPA) |
| Retention | 6-year retention with locked archives |
| Audit | Every change to a ticket and to permissions is logged |
| Security | Security hardening and penetration testing before UAT |
| Tone | Growth-focused language, matching the Ministry manual |

---

## 10. Open items

| # | Item | Status |
|---|---|---|
| 1 | What **P1 (September 2026)** means: internal pilot or production release. Security hardening and UAT are scheduled after September. | Open, critical |
| 2 | 44-week schedule with no buffer: delays in Phases 1–3 push back the September and December dates | Risk |
| 3 | Deployment model wording: the vision doc says "multi-tenant," while NFR decision R1 says multi-deployment. The number of deployments at launch is also unconfirmed. | To confirm |
| 4 | How parental and student input is collected (Principal screen P-08) | Open |
| 5 | Whether the principal must go through every appraisal step, or can jump straight to the summative report | Open |
| 6 | Where each teacher's cycle start date comes from, for automatic scheduling | Open |
| 7 | Whether Business Owner (HR) has read-only or write access to appraisals | Open (currently read-only) |
| 8 | Whether principals can see other schools in the same board | Open (currently school-only) |
| 9 | Which view a Deputy Principal who also teaches sees for their own appraisal | Open |
| 10 | Whether AODA testing time is included in the QA plan | Risk |
| 11 | Whether bilingual effort is fully estimated in Phase 3 | Risk |
| 12 | API availability from the central system for incoming data | Risk |

---
