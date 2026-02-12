# germanic-validate.lobster
# Deterministic GERMANIC validation pipeline for OpenClaw agents.
#
# Usage by agent:
#   run_workflow("germanic-validate", {
#     schema: "practice.schema.json",
#     input: "dr-sonnenschein.json",
#     output: "dr-sonnenschein.grm"
#   })
#
# What this does:
#   1. Compiles JSON → .grm with full schema validation
#   2. On error: stops, shows ALL violations, asks human to fix
#   3. On success: inspects the .grm header, confirms output
#
# Why Lobster instead of agent steps:
#   - Deterministic: No LLM decides "should I validate?"
#   - Atomic: Either all steps pass or nothing is produced
#   - Auditable: Every step logged with exit codes

name: germanic-validate
version: "1.0"
description: >
  Compile and validate JSON data against a GERMANIC schema.
  Produces a .grm binary file with typed, validated data.
  If validation fails, shows all errors for human review.

args:
  schema:
    required: true
    description: >
      Schema name (e.g. "practice") or path to .schema.json file.
      Use "practice" for the built-in healthcare schema.
      Use a .schema.json path for custom schemas.
  input:
    required: true
    description: "Path to the JSON data file to validate and compile."
  output:
    required: false
    description: "Path for .grm output. Default: input filename with .grm extension."

requires:
  bins:
    - germanic  # cargo install germanic

steps:
  # Step 1: Compile (validates + builds .grm)
  - id: compile
    description: "Validate JSON against schema and compile to .grm"
    command: >
      germanic compile
      --schema {{ args.schema }}
      --input {{ args.input }}
      {{ "--output " + args.output if args.output else "" }}

  # Step 2: Error gate — if compile failed, show errors and ask human
  - id: review_errors
    description: "Compilation failed — review validation errors"
    condition: "{{ steps.compile.exitCode != 0 }}"
    approval: required
    message: |
      ❌ GERMANIC validation found errors:

      {{ steps.compile.stderr }}

      The data does not satisfy the schema contract.
      Please fix the JSON and retry.

  # Step 3: Inspect the produced .grm file
  - id: inspect
    description: "Verify .grm header and show metadata"
    condition: "{{ steps.compile.exitCode == 0 }}"
    command: >
      germanic inspect
      {{ args.output or args.input | replace('.json', '.grm') }}

  # Step 4: Success confirmation
  - id: done
    description: "Compilation successful"
    condition: "{{ steps.compile.exitCode == 0 }}"
    message: |
      ✅ GERMANIC compilation successful.

      Schema:  {{ args.schema }}
      Input:   {{ args.input }}
      Output:  {{ args.output or args.input | replace('.json', '.grm') }}

      {{ steps.inspect.stdout }}
