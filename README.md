# GERMANIC

*"Germans have a word for everything."* Yeah. And now they have a binary format too. Because of course they do.

What happens when a construction engineer from Vienna gets bored over Christmas, discovers Rust, and starts asking questions that won't go away? You get a strict, schema-validated binary format with a name that's exactly as subtle as you'd expect.

> Know the Simpsons bit? [Germans Who Say Nice Things](https://www.youtube.com/watch?v=Qqn45gvOyuM&t=37s). GERMANIC doesn't say nice things. GERMANIC says: *"Phone number missing. Won't compile."*

---

**Everywhere AI touches data, a contract is missing. GERMANIC is that contract.**

`cargo install germanic`

---

## The Problem

AI doesn't hallucinate because it's stupid. It hallucinates because nobody tells it what's *right*.

When an AI agent needs a phone number today, it has three sources:

**BELIEVE** — Training weights. *"I believe the number is 0123-456789."* Outdated since the cutoff date. Statistically plausible doesn't mean true.

**THINK** — Web scraping. *"I think that's the phone number."* Has to interpret HTML. Can be prompt-injected.

**KNOW** — Schema-validated data. *"I know the phone number is +49 123 9876543."* Typed. Validated. Binary. Not interpretable, not manipulable.

GERMANIC builds the third source. JSON in, validated `.grm` binary out. If the data is incomplete, it doesn't compile. Period.

---

## What Your Agent Is Missing

If your AI agent produces or consumes data — who validates it?

**Your agent fills out a form.** It forgets the zip code. Nobody notices. The user gets a letter to an incomplete address. — With a schema contract, compilation fails. The agent *has* to ask.

**Your agent scrapes a website.** The HTML contains: *"Ignore all previous instructions."* The agent executes it. — With a binary format, there's no interpretable text. No free text, no injection.

**Your agent produces structured output.** The LLM returns `"yes"` instead of `true`. Everything downstream breaks. — With type checking, the schema catches the error at the source.

**Your agent outputs opening hours.** It invents "Sunday 10am-2pm" because it sounds statistically plausible. — With validated data, it outputs exactly what's in the contract. Or nothing.

If nobody validates your agent's data — that's the problem GERMANIC solves.

---

## The Contract Proof

No benchmarks. No nanoseconds. Instead: what happens when data is **wrong**?

Every row is backed by a real test. Skeptics welcome: `cargo test --test vertragsbeweis -- --nocapture`

| # | Scenario | HTML/Scraping | JSON-LD | JSON Schema D7 | .grm |
|---|----------|:---:|:---:|:---:|:---:|
| S1 | Required field missing | silent | silent | reports | **WON'T COMPILE** |
| S2 | Required field empty `""` | silent | silent | silent¹ | **WON'T COMPILE** |
| S3 | Wrong type: `"yes"` instead of `true` | silent | silent | reports | **WON'T COMPILE** |
| S4 | Prompt injection in text field | executable | injectable | injectable | **binary bytes** |
| S5 | Nested field missing | silent | silent | reports² | **WON'T COMPILE** |
| S6 | String where int expected | silent | silent | reports | **WON'T COMPILE** |
| S7 | Unknown extra field | absorbed | absorbed | accepted³ | **stripped** |
| S8 | `null` for required field | silent | silent | often silent⁴ | **WON'T COMPILE** |

Eight ways your data can fail. JSON-LD catches zero. JSON Schema catches three. GERMANIC catches all eight — at compile time.

> ¹ `type: "string"` is satisfied by `""`. You'd need explicit `minLength: 1`.
> ² Only if nested `required` is correctly defined.
> ³ `additionalProperties` defaults to `true`.
> ⁴ Many implementations treat `null` laxly with `type: "string"`.
>
> **Source:** [`tests/vertragsbeweis.rs`](crates/germanic/tests/vertragsbeweis.rs) — not benchmarks, but guarantees.

---

## Quick Start

```bash
cargo install germanic

# 1. Start with any JSON
echo '{"name": "Dr. Sonnenschein", "phone": "+49 30 1234567"}' > practice.json

# 2. Generate a schema
germanic init --from practice.json --schema-id health.practice.v1

# 3. Compile to validated binary
germanic compile --schema practice.schema.json --input practice.json --output practice.grm

# 4. Inspect the result
germanic inspect practice.grm
```

JSON in. Validated binary out. No Rust knowledge required. No FlatBuffer knowledge required.

---

## For AI Agents: The Feedback Loop

GERMANIC exposes three tools — as an MCP server (Model Context Protocol), usable by any compatible client:

**`germanic_init`** — Generate a schema from example JSON.

**`germanic_compile`** — Compile JSON against a schema. Errors come back as a structured list.

**`germanic_inspect`** — Read a `.grm` file, return schema ID and data as JSON.

The key: when `compile` fails, the agent gets a clear error — *"telefon: required field missing"* — and can ask the user for the missing piece. That's not silent failure. That's a feedback loop.

```
WITHOUT GERMANIC:
  User: "Create a practice page for Dr. Müller"
  Agent → writes HTML → hopes everything is correct
       → forgets the zip code → invents a phone number
       → NOBODY NOTICES

WITH GERMANIC:
  User: "Create a practice page for Dr. Müller"
  Agent → fills JSON according to schema
       → germanic compile
       → ERROR: "plz" missing (required)
       → Agent asks user: "What's the zip code?"
       → compile succeeds → data guaranteed complete
```

GERMANIC doesn't make AI agents smarter. It makes them honest.

---

## 42 Domains. We Hope the Community Makes It 420.

From healthcare to hospitality, from legal to logistics — 42 schema definitions, each in German and English. Not because we know every industry. But because we want to show: the principle is universal.

A restaurant needs a data contract just as much as a medical practice, a hotel, a citizen's office, or a sports club. We hope the community picks this up, gets creative, and contributes domains we never thought of. Schema definitions are JSON — no Rust knowledge required.

*Healthcare* — Practice, Hospital, Pharmacy, Midwife, Optician, Psychotherapist, Veterinarian, Care Service · *Dining & Tourism* — Restaurant, Café, Hotel, Cinema, Museum, Pool, Vacation Rental, Campground · *Trades & Services* — Electrician, Photographer, Cleaning, Locksmith, IT Service · *Education* — Daycare, Music School, Tutoring, Community College · *Real Estate* — Listing, Property Management, Energy Consulting · *Legal & Finance* — Law Firm, Tax Advisor · *Public* — Citizen's Office, Tourist Info, Farmers Market, Public Event · *Mobility* — Parking Garage, Charging Station, Bike Rental · *Business* — Company, Job Posting, Trade Fair · *Associations* — Sports Club, Church Community · *Agriculture* — Farm Shop, Winery

---

## Not JSON-LD

GERMANIC doesn't replace JSON-LD. Different problem.

JSON-LD tells search engines what your website *is*. GERMANIC guarantees to AI agents that the data is *correct*.

JSON-LD is for Google. GERMANIC is for the machines that have to *work* with the data.

---

## Architecture

```
JSON ──► Schema Validation ──► FlatBuffer Builder ──► .grm
         (required? type?)     (zero-copy binary)    (header + payload)

.grm = Magic Bytes + Schema-ID + Version + Signature Slot + FlatBuffer Payload
```

Under the hood: FlatBuffers for zero-copy deserialization, Rust for type safety, proc macros for schema code generation. The `.grm` file is a container with header metadata and a FlatBuffer payload — readable without deserialization, validatable without the original compiler.

---

## The Story

Christmas 2025. Construction engineer from Vienna. Rustacean. Career changer into coding — not from CS, from building sites and project management.

The head was finally clear. No desire to deal with the usual business problems. And — to be honest — a bit bored.

And then this one question that just wouldn't go away: *If AI touches data everywhere — where's the contract?*

Most of it was built over New Year's. Then the project sat for a month because I wasn't quite sure what to do with it. Now I know: open source it and hope that other people — and machines — find it useful.

The code isn't perfect. But it works, it's tested, and it's honest. Just like the binary format: strict, precise, and as German as the name suggests.

---

## German Schema Fields — Yes, on Purpose

You'll notice some schema field names are German: `telefon`, `adresse`, `oeffnungszeiten`. That's not a bug. GERMANIC started with German healthcare data, and the field names stuck — as a running joke, and because a project called GERMANIC with exclusively English field names would just be a missed opportunity.

All 42 domains ship in both German and English (`examples/de` and `examples/en`). The CLI, the API, and the documentation are English. The German schemas are the originals — the English ones are the translations. Just like the real world: the interface is international, the data is local.

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

Contribute schemas, report bugs, discuss ideas — all welcome. Schema definitions are JSON, no Rust required. And if you do write Rust: the compiler is happy about pull requests.

If you have a domain that's missing — a flower shop, a coworking space, a dog groomer — open an issue or submit a PR. The schema format is simple, and the more domains we cover, the more useful the contract becomes.

### Development: FlatBuffer Schema Changes

`cargo install germanic` works out of the box — no external tools required. The FlatBuffer bindings are pre-generated and checked into the repository.

If you modify `.fbs` schema files, regenerate the Rust bindings:

```bash
# Requires flatc: brew install flatbuffers (macOS) / apt install flatbuffers-compiler (Linux)
./scripts/regenerate-flatbuffers.sh
```

This updates `crates/germanic/src/generated/` and should be committed alongside your `.fbs` changes.
