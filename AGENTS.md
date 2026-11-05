# AGENTS

Guidelines for the use of **Large Language Models (LLMs)**, AI coding agents, and code-generation tools within this repository. These guidelines apply to all contributors, crates, and development tasks.

*Context: This project is currently developed by a single contributor. These guidelines therefore primarily reflect my personal approach to LLM-assisted development and the standards I apply to my own work. They also provide a reference for future contributors.*

## Part I - Guiding Principles

<h2 style="border-bottom: none;">
  <em>Use any tools you find useful. Take full responsibility for what you produce.</em>
</h2>

This project aims to build **industry-grade trading software**. Reliability, correctness, and technical understanding take precedence over development speed.

LLMs can accelerate exploration, implementation, and research, but generated output is not a substitute for engineering judgement. The ability to understand, evaluate, and challenge a result remains essential, regardless of how it was produced.

### Appropriate Use

LLMs and coding agents may assist with:

- **Exploration:** Architectural designs, technical trade-offs, and alternative approaches.
- **Understanding:** Unfamiliar code, concepts, and failure modes.
- **Review:** Bugs, edge cases, and invariant violations.
- **Generation:** Boilerplate, scaffolding, tests, and documentation.
- **Development:** Implementation, debugging, and refactoring.
- **Research:** Technical summaries and critical assessment of assumptions.

### Ownership and Responsibility

**You own your code, regardless of who or what produced it.**

Disclosure of LLM assistance is not required. However, every contribution must be understood, evaluated, and validated by its author. Generated code is subject to exactly the same expectations as manually written code. This responsibility covers correctness, security, licensing, performance, maintainability, and operational behaviour. Neither the use of an LLM nor its assurances provide an exemption from these expectations.

### Engineering Standards

All contributions must meet the same engineering standards:

- **Correctness first:** Prioritise reliability and deterministic behaviour over development speed.
- **Measured performance:** Support performance claims with reproducible benchmarks.
- **Explicit failure handling:** Consider overflow, concurrency, latency, errors, and failure modes.
- **Architectural consistency:** Follow established repository conventions and design principles.
- **Verified changes:** Run appropriate formatting, linting, and tests before integration.

> **The development process is yours to choose. The quality of the result is yours to defend.**

## Part II - Agent Guidelines

These guidelines apply to LLM-based coding agents and other automated tools that interact directly with the repository. They are intended to promote consistent, reliable, and maintainable contributions without prescribing a particular development workflow.

### General Behaviour

- **Understand the context:** Inspect the relevant code, architecture, and existing conventions before proposing or making changes.
- **Stay within scope:** Focus on the requested task. Avoid unrelated modifications, speculative features, and unnecessary refactoring.
- **Prefer simplicity:** Favour straightforward, maintainable solutions over unnecessary abstractions, dependencies, or complexity.
- **Preserve consistency:** Follow existing naming conventions, coding styles, architectural patterns, and domain definitions.
- **Question assumptions:** Identify ambiguities, inconsistencies, and potential design issues rather than silently making assumptions.
- **Avoid unnecessary changes:** Do not modify working code or established interfaces without a clear reason.

### Code Quality

- **Respect existing abstractions:** Reuse established types, interfaces, and components where appropriate.
- **Maintain separation of concerns:** Avoid introducing unnecessary coupling between modules and crates.
- **Handle failures explicitly:** Consider error propagation, invalid states, resource management, and unexpected behaviour.
- **Consider performance:** Be mindful of allocations, copying, synchronisation, and unnecessary runtime overhead, particularly in performance-sensitive components.
- **Preserve compatibility:** Consider the impact of changes on existing interfaces, dependencies, and downstream components.

### Validation

- **Verify changes:** Run relevant tests, formatting, and linting where possible.
- **Test edge cases:** Consider boundary conditions, invalid inputs, and failure scenarios.
- **Do not conceal failures:** Never remove or weaken tests, suppress errors, or alter checks merely to obtain a passing result.
- **Distinguish facts from assumptions:** Do not claim that code has been tested, benchmarked, or verified unless it actually has.
- **Report limitations:** Clearly identify incomplete work, unresolved issues, and validation that could not be performed.

### Communication

- **Explain meaningful decisions:** Briefly justify non-trivial implementation choices and architectural changes.
- **Identify trade-offs:** Highlight relevant limitations, risks, and alternative approaches when they matter.
- **Be precise:** Prefer concrete technical explanations over generic assurances.
- **Summarise modifications:** Clearly state what was changed, why it was changed, and how it was validated.

### Repository Integrity

- **Do not modify** project configuration, dependencies, or build infrastructure without a relevant reason.
- **Do not introduce** credentials, secrets, generated artefacts, or unrelated files into the repository.
- **Do not overwrite** existing work or discard uncommitted changes without explicit authorisation.
- **Do not perform** destructive operations or interact with external production systems.
