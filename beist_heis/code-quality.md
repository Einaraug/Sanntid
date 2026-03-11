Project context:
I’m building a distributed real-time elevator control system in Rust. Please use these engineering principles while reading, modifying, and proposing code.

Core design principles from the attached software-construction reading:

1. Classes/modules should have one cohesive responsibility.
- Group data + behavior around clear domain concepts.
- Maximize how much of the system can be ignored at any one time.
- Prefer strong abstractions over “bags of data + random helper functions.”

Apply to this project:
- Keep clear boundaries between modules like:
  - hardware I/O
  - network/distributed state sync
  - elevator finite-state machine
  - order assignment / hall-call distribution
  - persistence / recovery
  - timing / watchdog / fault handling
- Avoid leaking implementation details across module boundaries.

2. Design around abstract data types, not implementation details.
- Model the problem domain directly.
- Interfaces should express domain operations, not storage/layout details.

Apply to this project:
- Use domain types like ElevatorId, Floor, HallCall, CabCall, Direction, MotorCommand, DoorState, NodeState, AssignmentMap, LamportTime/Version, etc.
- Avoid APIs that expose raw arrays, magic integers, or loosely typed tuples when a domain type would be clearer.
- Prefer newtypes/enums/structs over primitive obsession.

3. Good interfaces hide complexity.
- Public interfaces should be minimal, cohesive, and hard to misuse.
- Don’t make callers rely on call ordering quirks or hidden initialization behavior.

Apply to this project:
- Encode invariants in types and constructors.
- Prefer explicit state transitions and typed commands/events.
- Make invalid states unrepresentable where practical.
- Avoid APIs like “call init first, then maybe update, unless restore was called.”
- Prefer clear ownership of responsibilities.

4. Prefer composition/containment over inheritance-style complexity.
- Composition is the workhorse.
- Reduce coupling and avoid clever hierarchies.

Apply to Rust:
- Prefer structs + trait-based seams + composition.
- Use traits for capability boundaries, not speculative abstraction.
- Keep trait surfaces small and purpose-driven.

5. The main reason to create a routine/module is to reduce complexity.
- Small, sharply named routines are good.
- A routine should do one thing well and be easy to reason about.

Apply to this project:
- Break long control-flow functions into named steps.
- Prefer routines like:
  - compute_next_action(...)
  - assign_hall_calls(...)
  - merge_peer_state(...)
  - should_open_door(...)
  - detect_node_timeout(...)
rather than giant “tick” functions with many branches.

6. Routine names should say exactly what they do.
- Use strong verb phrases.
- Weak names usually signal weak design.

Apply to this project:
- Prefer names like:
  - reconcile_remote_state
  - persist_cab_orders
  - recalculate_assignments
  - transition_to_moving
  - clear_completed_orders
- Avoid vague names like:
  - handle_data
  - process
  - update_stuff
  - do_logic

7. Variable/type names should fully describe the domain concept.
- Favor problem-oriented names over computer-oriented names.
- Optimize for read-time clarity, not write-time convenience.

Apply to this project:
- Prefer names like:
  - assigned_hall_calls
  - peer_last_seen
  - active_elevator_count
  - door_open_deadline
  - estimated_time_to_floor
  - recoverable_orders
- Avoid names like:
  - tmp, data, info, arr, state2, x, msg2
- Use consistent prefixes/suffixes when useful:
  - current_*, next_*, pending_*, assigned_*, remote_*, local_*, last_*, max_*, min_*

8. Code should be self-documenting first; comments should explain intent, not restate code.
- Improve naming/structure before adding explanatory comments.
- Comment the why, invariants, assumptions, protocol rules, failure modes, and timing constraints.

Apply to this project:
- Good comment targets:
  - why a distributed merge rule is safe
  - why a timeout threshold exists
  - what fault model is assumed
  - what happens during network partition/rejoin
  - ordering/consistency assumptions
  - safety invariants for motor/door behavior
- Bad comments:
  - comments that just narrate obvious line-by-line code

9. “Tricky” code is usually a design smell.
- If something feels hard to explain, simplify it.
- Prefer straightforward control flow over cleverness.

Apply to this project:
- Keep scheduler/assignment logic explicit and testable.
- Keep distributed reconciliation deterministic.
- Prefer boring, auditable logic over compact but opaque code.

10. Documentation effort should focus on the code itself.
- Clear structure, names, and interfaces first.
- Then add high-value comments and module docs.

Rust-specific expectations:
- Use enums and pattern matching to model states/events clearly.
- Use structs/newtypes to encode domain meaning.
- Use Result and explicit error types for recoverable failures.
- Keep concurrency/network boundaries explicit.
- Separate pure decision logic from side effects where possible.
- Write code so distributed behavior can be tested deterministically.

Architecture bias for this project:
- Favor a small number of explicit domain modules with tight interfaces.
- Keep the elevator controller as a well-defined state machine.
- Keep distributed coordination separate from local control logic.
- Keep persistence/recovery rules explicit.
- Preserve safety invariants and real-time responsiveness over abstraction for its own sake.

When changing code:
- First identify the domain responsibility of the code being changed.
- Preserve or improve cohesion.
- Reduce coupling.
- Strengthen names and types.
- Simplify routines.
- Document invariants/protocol assumptions if they are non-obvious.
- Do not introduce vague helpers or mixed-responsibility modules.

Preferred review lens:
- Is this module/routine doing exactly one coherent thing?
- Does the API expose domain intent or implementation detail?
- Are names precise and problem-oriented?
- Are invariants obvious from types and structure?
- Is distributed behavior deterministic and understandable?
- Can a reader safely ignore unrelated parts of the system?
