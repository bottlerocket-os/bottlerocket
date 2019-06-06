- Feature Name: Build System Tenets
- Start Date: 2019-06-06
- RFC PR: [amazonlinux/thar#35](https://github.com/amazonlinux/thar/pull/35)

# Summary
[summary]: #summary

This RFC establishes the tenets held the build system that supports
building, developing, and releasing Thar. These tenets describe the
characteristics each aspect ought to display and internalize in how it
is to meet the needs in project builds, aiding security processes,
facilitating automated build-test-release processes, and the
developers working on and with Thar.

# Motivation
[motivation]: #motivation

This RFC is intended to establish tenets on which we may direct effort
and thought when considering how package structure, build system and
automation integration, and overall OS lifecycle integration points
(testing, staging, release).

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

The build system internalizes the tenets of being:

* **Tractable:**

  The build system is malleable, versatile, and comprehensively
  documented for developers to make needed changes.

* **Configurable:**

  The build system accommodates configurable inputs and outputs to
  construct.

* **Simple, Not Easy:**

  The build system prioritizes simple solutions that aren't
  necessarily easy to arrive at.

* **Scalable For Needs:**

  The build system scales to meet the needs of building Thar and
  reduces efforts required by developers to maintain and grow it.

The build system, under these tenets, should be able to consider
improvements and upholding of features. This allows for exploring the
existing build system in the same breath as encouraging trains of
thought and experimentation equally.

The build system adhering to these tenets, today and in the future
implementations, is empowered and capable of such tasks as:

* Handling partial build of images and components

* Enabling user configurable builds

* Providing simple, tractable means to add new packages

* Handling cross architecture builds

* Prioritizing overall improvements developer productivity, like:

    * Minimizing the number of tools required

    * Minimizing the steps required to develop and contribute

    * Minimizing the time to develop and contribute

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

No technical implementation for reference as this is a wider goal and
consensus document.

# Drawbacks
[drawbacks]: #drawbacks

These tenets do exclude a subset of endeavors where deviation is
required - which may stifle or otherwise prevent contributions that
may otherwise improve the quality of the project (e.g. quality of
time-spent-developing, quality of code in build system, quantity of
code). Despite establishing these tenets, I would expect that
justifiable compromise will always be an option depending on
circumstances.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

- Tenets grant a framework for evaluating and measuring decisions and
  implementation moving forward

- Tenets reduce rationalizations required on the part of the developer
  at the time of prototyping and proof-of-concept-ing.

- Solutions produced with tenets in mind are likely in alignment with
  what core contributors would accept as a contribution, in general.

- In contrast, without tenets there would be ambiguity and, in it,
  room for spontaneous and organic growth.

# Prior art
[prior-art]: #prior-art

- Tenets and principles drive decisions, discussions, and
  disagreements through to conclusions and, often, results.

  - Amazon's Leadership Principles are an example where biases,
    disagreements, and ingenuity may be at odds but still encourage an
    informed decision.

    https://www.amazon.jobs/en/principles

# Unresolved questions
[unresolved-questions]: #unresolved-questions

This document **does not** intend to resolve or address the following
questions:

- How does this evaluate against the current implementation of the
  build system?

  This document intends to offer kinds of "priorities" for making
  further changes and improvements.

- Do we need to make changes to the build system?

- What concrete measurements can be made against these tenets? Should
  there be?

  These tenets are inherently qualitative and still leave room for
  others' standards to demand more or less in areas.

# Future possibilities
[future-possibilities]: #future-possibilities

Under these tenets the build system's evolution, improvement, and/or
replacement is warranted at any time. I would expect to see design
proposals and suggestions on an wide range of ways to improve
ourselves.
