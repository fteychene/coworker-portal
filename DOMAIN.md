# Domain design

## Core aggregates

User : A user of the space
```typescript
type User = {
    id: int
    username: string
    password: string
    firstname: string
    lastName: string
    email: string
    billingAddress: string
}
```

Service : A commercial offer that user subscribe. Example : (10 days oupons, 1 Month full access, 1 Month 10 days access, ...)
```typescript
type Service = {
    id: int
    name: string
    description: string
    price: float
    voucherSpec: VoucherSpec
}

type VoucherSpec = 
    | {kind: "Monthly"} // duration for Monthly service is to create one voucher that as a duration until end of current month (example March 3th  should createa Voucher with validity until March 31th midnight)
    | {
        kind: "Book"
        amount: int
        duration: int
    } 
```

Bill: A user buy of a service.
```typescript
type Bill = {
    id: int
    number: string   // format: FYYYYMMNNN — see LastNumberCompute below
    user: User       // ref
    service: Service // ref
    date: DateTime
    amount: float
    issuerAddress: string
    billingAddress: string
    vouchers: [Voucher] // ref

    // invariant: number is computed at creation
    // invariant: date is computed at creation as system time
    // invariant: amount is a copy of service.price at creation time
    // invariant: once created Bill is immutable
}
```

Voucher: Represent access coupon a User can use in the coworking space.
Vouchers are persisted locally after being provisioned on Unify at invoice creation time. The stored `unifyId` enables later validity checks against the Unify edge without re-querying by note.
```typescript
type Voucher = {
    id: int           // local DB id
    unifyId: string   // Unify _id (MongoDB ObjectId) — immutable reference
    code: string      // 10-digit code from Unify, display as XXXXX-XXXXX
    createdAt: DateTime
    duration: int     // in hours
    status: VoucherStatus

    // invariant: unifyId and code are set at creation from Unify response and never updated
    // invariant: createdAt is computed at creation as system time
    // invariant: duration unit is always hours
    // invariant: duration is computed based on Service voucher spec — see MonthlyVoucherDuration
    // invariant: status is refreshed by the validity check command, not at creation
}

type VoucherStatus =
    | { kind: "Valid" }
    | { kind: "Used" }
    | { kind: "Expired" }
    | { kind: "Unknown" }  // Unify unreachable or voucher not found
```

## Business rules

### Monthly voucher duration derivation

**MonthlyVoucherDuration** — Duration computation for `Monthly` vouchers

When creating a voucher for a `Monthly` service, the duration is the number of hours from the creation instant to midnight at the end of the current month.

Rule:
- `end = last day of creation month, at 23:59:59 local time`
- `duration = ceil((end - createdAt) in hours)`

Example: created on March 3rd at 10:00 → end is March 31st 23:59:59 → duration = 686 hours.

> **Invariant:** the duration is computed at creation time and frozen. A voucher created mid-month has a shorter duration than one created at the start.

### Bill last number compute

**LastNumberCompute** — Bill number generation rule

Format: `F` + `YYYYMM` + `NNN`
- `F` — fixed prefix (Facture)
- `YYYYMM` — year and month of the bill date
- `NNN` — 3-digit zero-padded incremental counter

The counter is computed at creation time by fetching the last saved bill and incrementing its counter:
- If no bill exists yet → `001`
- Otherwise → parse the last 3 chars of the latest bill number as an integer, increment by 1, zero-pad to 3 digits

The counter is **global** (not per-month): a new month does not reset it. Example sequence: `F202604003` → `F202605004`.

> **Invariant:** `number` is immutable once set. It must be assigned at creation and never updated.
> **Caution:** the increment must be computed atomically (DB sequence or row lock) to prevent duplicates under concurrent bill creation.


## Edges


### Postgres — owned by this app

All int are stored as int4.

This app owns the schema and data for:
- `VoucherSpec` — stored in `voucher_spec` table
- `Voucher` — stored in `voucher` table (id, bill_id, unify_id, code, created_at, duration, status). `status` is the only mutable field, updated by the validity check command.

### Postgres — owned by external app (django-billjobs)

> **Anti-pattern:** we directly access the database of the existing invoicing app (django-billjobs). This is intentional and temporary — the target is to switch to an API contract later.

This app reads from (no schema ownership):

**`auth_user`** → `User`
| Column | Type | Notes |
|--------|------|-------|
| `id` | int4 | |
| `username` | varchar(150) | |
| `password` | varchar(128) | Django PBKDF2-SHA256 format |
| `first_name` | varchar(150) | |
| `last_name` | varchar(150) | |
| `email` | varchar(254) | |
| `is_active` | boolean | filter on login |

**`billjobs_service`** → `Service`
| Column | Type | Notes |
|--------|------|-------|
| `id` | int4 | |
| `reference` | varchar(5) | |
| `name` | varchar(128) | |
| `description` | varchar(256) | |
| `price` | float8 | |
| `is_available` | boolean | filter on listing |

This app writes to (anti-pattern, compatible with shared invoicing schema):

**`billjobs_bill`** → `Bill`
| Column | Type | Notes |
|--------|------|-------|
| `id` | int4 | |
| `number` | varchar(16) | unique, see LastNumberCompute |
| `user_id` | int4 | FK → auth_user.id |
| `billing_date` | date | set at creation |
| `amount` | float8 | copy of service.price |
| `issuer_address` | varchar(1024) | |
| `billing_address` | varchar(1024) | copied from user profile |
| `isPaid` | boolean | always false at creation |

**`billjobs_billline`** → persistence detail for `Bill`, not in core domain
| Column | Type | Notes |
|--------|------|-------|
| `id` | int4 | |
| `bill_id` | int4 | FK → billjobs_bill.id |
| `service_id` | int4 | FK → billjobs_service.id |
| `quantity` | int2 | always 1 |
| `total` | float8 | = amount (service.price at creation) |
| `note` | varchar(1024) | left empty |


### Unify

Vouchers are managed by Unify via a cookie-authenticated API. The app must login first and carry the `unifises` session cookie on every request.

#### Authentication
`POST /api/login` → receive `unifises` cookie → attach to all subsequent calls.

#### Create vouchers
`POST /api/s/{site}/cmd/hotspot`

Domain → Unify field mapping:

| Domain | Unify field | Notes |
|--------|-------------|-------|
| `VoucherSpec.amount` (Book) / `1` (Monthly) | `n` | number of vouchers to generate |
| `VoucherSpec` duration | `expire_number` | hours value (see below) |
| *(fixed)* | `expire_unit` | always `60` (hour multiplier) |
| *(fixed)* | `quota` | always `2` (phone + computer) |
| `FYYYYMMNNN_FirstName` | `note` | bill number + `_` + user firstname |

Duration mapping per spec kind:
- `Book`: `expire_number = VoucherSpec.duration` (already in hours)
- `Monthly`: `expire_number = MonthlyVoucherDuration` (hours until end of month)

The response only returns a `create_time` Unix timestamp — **not** the voucher codes. The `create_time` must be used immediately to retrieve the batch (see below).

#### Retrieve vouchers
`POST /api/s/{site}/stat/voucher` with body `{ "create_time": <unix_timestamp> }`

Returns all vouchers created at or after that timestamp. Filter by `note` matching `FYYYYMMNNN_FirstName` to isolate the batch for a given bill.

Unify → Domain field mapping:

| Unify field | Domain | Notes |
|-------------|--------|-------|
| `_id` | `Voucher.unifyId` | MongoDB ObjectId |
| `code` | `Voucher.code` | 10 digits, display as `XXXXX-XXXXX` |
| `create_time` | `Voucher.createdAt` | Unix timestamp → DateTime |
| `duration` | `Voucher.duration` | Unify stores minutes; convert to hours (`/ 60`) |
| `status` | `Voucher.status` | `VALID_ONE`/`VALID_MULTI` → Valid, `USED_MULTIPLE` → Used |

#### Two-step creation flow
1. Call create → capture `create_time` from response
2. Call list with `create_time` → filter by `note` → map to `Voucher` domain objects


## Features

### Login as a user
As a coworker I want to connect to this application.

Acceptance criteria :
 - User can connect with a user / password
 - Logged user have a token that provide authentication for all other features

### Create a bill
As a logged user I would like to create a bill for myself.
I provide the service type I want and the application executes the following steps in a single transaction:
 - Receive a bill creation command with the service type
 - Create the bill (number, date, amount snapshot)
 - Create the vouchers on Unify
 - Store the bill and vouchers
 - Return the bill with all computed information

Acceptance:
 - Bill number follows LastNumberCompute
 - Vouchers are created on Unify with the correct duration and note
 - Vouchers are stored locally with status Valid
 - The bill is always created for the authenticated user — no impersonation
 - If Unify voucher creation fails, the bill is not persisted (DB transaction rolled back)

### List my bills
As a logged user I can see my own bills with pagination and filtering.

Acceptance:
 - Results are scoped to the authenticated user only
 - Supports offset/limit pagination
 - Filterable by date range and bill number

### Generate voucher PDF
As a logged user I can generate a voucher PDF for a bill containing the codes and duration in a compact way.

> **Template TBD** — the PDF layout and content (beyond voucher code + duration) will be defined in a future iteration. Implementation should keep the rendering logic isolated behind a template abstraction so it can be swapped without touching the domain.

### Voucher check
As a logged user I can get the live status of vouchers for a bill by querying Unify directly.

Acceptance:
 - Status is fetched live from Unify using the stored `unifyId` for each voucher
 - No side effects — local `Voucher.status` in DB is not updated by this call
 - Returns current Unify status mapped to `VoucherStatus` for each voucher of the bill