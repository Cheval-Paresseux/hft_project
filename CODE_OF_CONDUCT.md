# CODE OF CONDUCT 

Guidelines for the use of **Large Language Models (LLMs)**, AI coding agents, and code-generation tools within this repository. These guidelines apply to all contributors, crates, and development tasks.

*Context: This project is currently developed by a single contributor. These guidelines therefore primarily reflect my personal approach and the standards I apply to my own work. They also provide a reference for future contributors.*

## LLM-Assisted Development Policy

<h2 style="border-bottom: none;">
  <em>Use any tools you find useful. Take full responsibility for what you produce.</em>
</h2>

This project aims to build **industry-grade trading software**. Reliability, correctness, and technical understanding take precedence over development speed. LLMs can accelerate exploration, implementation, and research, but generated output is not a substitute for engineering judgement. The ability to understand, evaluate, and challenge a result remains essential, regardless of how it was produced.

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