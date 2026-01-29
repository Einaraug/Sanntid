TTK4145 - Progress report Preliminary 2026-01-29 Authors: Matheus
Ullmann Einar Augestad Fredrik Spalder

NTNU 2026 Group x

Contents 1

2

Description . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . ⁠3 1.1

Introduction . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
. . . . . . . . . . . . . . . . . . . . . . . . . . ⁠3

1.2

System design and fault tolerance . . . . . . . . . . . . . . . . . . .
. . . . . . . . . . . . . . . ⁠3

1.3

Failure handling . . . . . . . . . . . . . . . . . . . . . . . . . . . .
. . . . . . . . . . . . . . . . . . . . . . . . ⁠3

1.4

Communication and implementation . . . . . . . . . . . . . . . . . . . .
. . . . . . . . . . . ⁠3

Figures . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
. . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . ⁠5

1 Description 1.1 Introduction This report describes our preliminary
design for a distributed elevator control system. The main focus of the
design is to ensure fault tolerance in the presence of network
unreliability, spontaneous crashes, and unscheduled restarts, ensuring
no calls are lost.

1.2 System design and fault tolerance The system uses a fully
decentralized peer-to-peer architecture where all elevators act as
backups for each other. No single node has global responsibility, and
all nodes maintain a local (global) view of the system state. This
avoids single points of failure and simplifies recovery from faults.
Hall orders are global and may be served by any elevator, while cab
orders are associated with a specific elevator. Both hall and cab orders
are broadcast to all nodes to enable redundancy and recovery after
failures. Only after all active peers acknowledge a new hall order can
we guarantee that the order will be served, and the light will turn on.
This way we fulfill the button light contract.

1.3 Failure handling Node availability is monitored using heartbeat
messages. Missing heartbeats indicate that a node is unavailable and
should be excluded from order assignment. If a node becomes unavailable,
its active hall orders are redistributed among the remaining elevators.
Cab orders are not reassigned, but remain stored redundantly. When a
node restarts, it announces its presence and synchronizes its state with
the network, recovering its cab orders from other nodes.

1.4 Communication and implementation UDP was chosen as the mesaage
transport protocol since the system relies on broadcast communication,
while TCP is strictly point-to-point and would require additional
coordination. Reliability is implemented at the application level using
acknowledgements and retransmissions.

3

The system will be implemented in Go, chosen for its built-in
concurrency primitives and networking support, which is well suited for
for our distributed real-time system.

4

2 Figures The following figures describe our planned implementation of
the project.

Figure 1 : Class diagram of the system modules

Figure 2 : Sequence diagram illustrating hall order flow

5

Figure 3 : Activity diagram illustrating message flow for a hall order 6


