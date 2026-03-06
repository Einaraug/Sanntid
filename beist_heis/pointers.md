# Code Review Feedback

## Critical Bug: `to_fsm` Channel Never Used

The `run` function in `world_view.rs` **never sends anything to FSM**.

### The Problem

Look at the `run` function (lines 248-301):

```rust
pub fn run(
    mut self,
    from_buttons: cbc::Receiver<ButtonEvent>,
    from_fsm: cbc::Receiver<Elevator>,
    from_network: cbc::Receiver<WorldView>,
    to_fsm: cbc::Sender<ConfirmedOrder>,  // <-- Accepted but NEVER USED
    to_network: cbc::Sender<WorldView>,
) {
```

The channel `to_fsm` is passed in but **never called**. Additionally:

1. `confirm_and_assign_orders()` is **never called** in the run loop
2. Even if it were, there's no code to send `ConfirmedOrder` to the FSM

The loop handles:
- ✅ Button presses → creates Unconfirmed orders
- ✅ FSM state updates → stores elevator state
- ✅ Network messages → merges WorldViews
- ✅ Periodic broadcast → sends to network
- ❌ **Never confirms orders**
- ❌ **Never sends orders to FSM**

### The Fix

After each event (or in the broadcast interval), you need to:
1. Call `confirm_and_assign_orders()`
2. Send confirmed orders for this node to FSM

Here's what's missing at the end of the loop (around line 299):

```rust
// After broadcasting to network, check for confirmed orders to send to FSM
self.confirm_and_assign_orders();

// Send confirmed orders assigned to this node to FSM
for floor in 0..N_FLOORS {
    for button in [Button::HallUp, Button::HallDown] {
        let order = self.order_table.get_hall_order(floor, button as usize);
        if order.get_state() == OrderState::Confirmed
           && order.get_node_id() == self.self_id {
            let _ = to_fsm.send(ConfirmedOrder { floor, button });
            // Clear or mark as dispatched...
        }
    }
    // Also handle cab orders for self
    let cab = self.order_table.get_cab_order(floor, self.self_id);
    if cab.get_state() == OrderState::Confirmed {
        let _ = to_fsm.send(ConfirmedOrder { floor, button: Button::Cab });
    }
}
```

You'll also need logic to avoid re-sending the same confirmed order repeatedly (either clear it after sending, or track "dispatched" state).

---

## Structure & Interface Feedback

### 1. Naming Collision: Two `Elevator` Types

You have `elevio::elev::Elevator` (hardware TCP connection) and `elev_algo::elevator::Elevator` (state struct). This is confusing:

```rust
// elevio/elev.rs
pub struct Elevator { socket: Arc<Mutex<TcpStream>>, ... }

// elev_algo/elevator.rs
pub struct Elevator { pub floor: i32, pub dirn: Dirn, ... }
```

**Suggestion:** Rename the hardware one to `HardwareConnection`, `ElevatorDriver`, or `ElevatorIO`.

### 2. Inconsistent Floor/Button Types

Floors are `i32` in some places, `u8` in others, `usize` in yet others:

```rust
// elev_algo/elevator.rs
pub floor: i32

// elevio/poll.rs
pub floor: u8

// orders.rs - indexing uses usize
pub hall: [[HallOrder; 2]; N_FLOORS]
```

**Suggestion:** Pick one canonical type (probably `usize` for indexing, or a `Floor` newtype) and convert at boundaries.

### 3. Magic Sentinel Value

```rust
pub const UNASSIGNED_NODE: usize = 100;
```

This is fragile. If you ever have 100+ nodes, it breaks silently.

**Suggestion:** Use `Option<usize>` for `node_id`, or define a proper `NodeId` enum:
```rust
pub enum NodeAssignment {
    Unassigned,
    Assigned(usize),
}
```

### 4. Duplicated Constants

`N_FLOORS = 4` is defined in both `main.rs` and `elev_algo/elevator.rs`. `N_NODES = 3` is in `world_view.rs`.

**Suggestion:** Create a `config.rs` or `constants.rs` at the crate root and import everywhere.

### 5. Large `WorldView` Interface

`WorldView` has many public setters that all need to coordinate counter increments:

```rust
pub fn set_elevator(...)
pub fn set_peer_availability(...)
pub fn set_hall_order_state(...)
pub fn set_hall_order_node_id(...)
pub fn set_cab_order_state(...)
```

**Suggestion:** Consider grouping related operations. For example, `set_hall_order_state` and `set_hall_order_node_id` could be a single `assign_hall_order(floor, button, state, node_id)` that atomically updates both.

### 6. `seen_by` Array is Fixed Size

```rust
pub seen_by: [bool; N_NODES]
```

This hardcodes node count at compile time. If you want dynamic clusters, this won't scale.

**Not necessarily a problem** if 3 nodes is a hard requirement, but worth noting.

### 7. FSM Output is a Grab Bag

```rust
pub struct FsmOutput {
    pub motor_direction: Option<Dirn>,
    pub door_light: Option<bool>,
    pub floor_indicator: Option<i32>,
    pub start_door_timer: bool,
    pub clear_lights: Vec<(usize, Button)>,
    pub set_lights: Vec<(usize, Button)>,
}
```

Mixing `Option` fields with `Vec` and `bool` makes it unclear what's "set" vs "unchanged".

**Suggestion:** Consider an explicit command pattern:
```rust
pub enum FsmCommand {
    SetMotor(Dirn),
    SetDoorLight(bool),
    SetFloorIndicator(i32),
    StartDoorTimer,
    ClearLight(usize, Button),
    SetLight(usize, Button),
}
// Then: Vec<FsmCommand>
```

### 8. Error Handling Inconsistency

Some functions return `Result`:
```rust
pub fn assign_hall_requests(...) -> Result<[[bool; 2]; N_FLOORS]>
```

Others use `unwrap()` freely in thread code. If the external assigner binary fails, what happens?

**Suggestion:** Define a clear error strategy. Critical threads should probably log and continue rather than panic.

### 9. Module Re-exports Could Be Cleaner

Consider whether all internal types need to be public. A flatter public API with internal implementation hidden would be easier to reason about.

---

## Architecture Feedback

**Good decisions:**
- Event-driven FSM with timer in `select!` is clean
- Counter-based CRDT merge handles network partitions gracefully
- Separating polling threads from logic threads prevents blocking

**Consider:**
- The 100ms broadcast interval is hardcoded. Making it configurable would help tuning.
- `confirm_and_assign_orders()` calls the external assigner binary synchronously. If that hangs, your WorldView thread blocks. Consider a timeout or async subprocess handling.
- Peer timeout (500ms appears in the code) should probably be a named constant near the broadcast interval.

---

## Summary

| Category | Rating | Notes |
|----------|--------|-------|
| Module organization | Good | Clear separation, just needs constant consolidation |
| Type safety | Mixed | Strong in FSM, weak at boundaries (floor/button types) |
| Naming | Needs work | `Elevator` collision, some magic numbers |
| Interface design | Good | Could reduce WorldView surface area |
| Error handling | Needs work | Inconsistent panic vs Result |
| Distributed logic | Strong | CRDT merge is well thought out |

The core architecture is sound. The main improvements are around type consistency and reducing ambiguity in the public interfaces.
