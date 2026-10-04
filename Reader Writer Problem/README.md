# 📖 Readers–Writers Problem — Rust

A Rust implementation of the classic **Readers–Writers Problem**, demonstrating how multiple threads can safely access a shared resource while maintaining **mutual exclusion, concurrency, fairness, and synchronization correctness**.

This implementation uses **semaphores, mutexes, condition variables, and a fair turnstile mechanism** to coordinate readers and writers. It also includes a live browser-based visualization and runtime statistics.

---

## 🧠 The Problem

The Readers–Writers Problem models a shared resource accessed by two types of processes:

- **Readers** — can access the resource simultaneously.
- **Writers** — require exclusive access.

The challenge is to allow maximum reader concurrency while ensuring that:

- A writer never runs while a reader is accessing the resource.
- Two writers never access the resource simultaneously.
- Readers are allowed to run concurrently.
- Waiting writers are not indefinitely starved by incoming readers.

---

## 💡 The Solution

The implementation uses a **turnstile-based fair Readers–Writers solution**.

The basic idea is:

```text
                 ┌──────────────┐
                 │   Turnstile  │
                 └──────┬───────┘
                        │
              ┌─────────┴─────────┐
              │                   │
           Readers             Writers
              │                   │
       ┌──────▼──────┐      ┌─────▼─────┐
       │  read_count │      │    wrt    │
       └──────┬──────┘      └─────┬─────┘
              │                   │
              └─────────┬─────────┘
                        │
                  Shared Resource
```

### Reader

Multiple readers can enter together.

```text
Reader → Turnstile → First reader locks resource
                  → Read
                  → Last reader releases resource
```

### Writer

Only one writer can enter at a time.

```text
Writer → Turnstile → Wait for resource
                  → Write exclusively
                  → Release resource
```

The **turnstile** ensures that once a writer starts waiting, new readers cannot continuously jump ahead of it.

---

## ⚖️ Fairness

The default mode uses a **turnstile mechanism** to provide fairer access between readers and writers.

The program records the waiting time of every reader and writer, allowing the behaviour of the synchronization mechanism to be observed rather than simply assumed.

---

## 🔄 Execution Flow

```text
                Program Starts
                      │
                      ▼
            Initialize Semaphores
                      │
                      ▼
             Start Visualization
                      │
                      ▼
          ┌───────────┴───────────┐
          │                       │
       Readers                 Writers
          │                       │
          ▼                       ▼
     Enter Turnstile         Enter Turnstile
          │                       │
          ▼                       ▼
   Join reader group         Wait for resource
          │                       │
          ▼                       ▼
     Read resource            Write resource
          │                       │
          ▼                       ▼
     Leave group             Release resource
          │                       │
          └───────────┬───────────┘
                      ▼
                 Next iteration
                      │
                      ▼
                  Summary
```

---

## 🌐 Live Visualization

The program also provides a browser-based visualization of the execution.

After starting the program, open:

```text
http://127.0.0.1:8080
```

The visualization receives events directly from the running Rust program and displays:

- Reader/writer states
- Waiting processes
- Protocol activity
- Runtime statistics
- Event log
- Execution replay and pause controls

---

## 🛠️ Technologies & Concepts

| Technology / Concept | Purpose |
|---|---|
| **Rust** | Core implementation |
| **Cargo** | Build and package management |
| **Threads** | Concurrent readers and writers |
| **Mutex** | Protect shared state |
| **Condvar** | Coordinate waiting threads |
| **Semaphore** | Control access to the shared resource |
| **Turnstile** | Improve fairness |
| **SSE** | Stream execution events to browser |
| **HTTP Server** | Serve the visualization |
| **Readers–Writers Problem** | Synchronization model |

### OS Concepts Demonstrated

- Process/thread synchronization
- Mutual exclusion
- Critical sections
- Semaphores
- Mutexes
- Condition variables
- Reader concurrency
- Writer exclusivity
- Starvation prevention
- Fairness
- Race-condition prevention

---

## ⚙️ Configuration

The program supports configurable:

```text
Readers
Writers
Iterations
Port
Execution mode
```

### Default Configuration

```text
Readers     : 3
Writers     : 2
Iterations  : 3
Port        : 8080
Mode        : Fair (Turnstile)
```

---

## 📈 Runtime Statistics

At the end of execution, the program reports statistics such as total execution time, reads/writes, maximum concurrent readers, reader/writer waiting times, final value, and invariant status.

Example:

```text
=== Summary ===
Total time            : 1398 ms
Total reads / writes  : 9 / 6
Max concurrent readers: 3
Reader wait avg / max : 260 ms / 414 ms
Writer wait avg / max : 177 ms / 259 ms
Final value           : 6
Invariants: OK (no violations)
```

### What this demonstrates

**9 reads / 6 writes**

All three readers perform three iterations, while both writers perform three iterations.

**Maximum concurrent readers = 3**

All three readers successfully access the resource concurrently.

**Final value = 6**

Six successful writes increment the shared resource from `0` to `6`.

**Invariants: OK**

No reader/writer synchronization violations occurred during the execution.

---

## 🚀 How to Run

### Prerequisites

- Rust
- Cargo

Check your installation:

```bash
rustc --version
cargo --version
```

### Clone the Repository

```bash
git clone <your-repository-url>
cd <your-repository>
```

### Run with Default Configuration

```bash
cargo run
```

### Custom Configuration

```bash
cargo run -- 5 2 10
```

This runs:

```text
5 Readers
2 Writers
10 Iterations
```

### Run in Basic Mode

```bash
cargo run -- --basic
```

The basic mode disables the fair turnstile mechanism.

### Change Visualization Port

```bash
cargo run -- --port=8081
```

Then open:

```text
http://127.0.0.1:8081
```

---

## 📁 Project Structure

```text
reader-writer/
│
├── src/
│   └── main.rs          # Readers–Writers implementation
│
├── rw_viz.html          # Live browser visualization
├── Cargo.toml           # Rust project configuration
├── Cargo.lock
└── README.md
```

---

## 🔐 Synchronization Checks

The program continuously verifies the core Readers–Writers invariants.

### Valid states

```text
Multiple Readers
      ✓

One Writer
      ✓

Reader + Writer
      ✗

Multiple Writers
      ✗
```

Violations are tracked during execution and reported in the final summary.

---

## 🎓 Learning Outcome

This project demonstrates how a theoretical Operating Systems synchronization problem can be turned into a real concurrent system.

Through the implementation, we explore:

- How threads coordinate access to shared resources
- Why synchronization is necessary
- How semaphores and condition variables work together
- Why readers can share a critical section
- Why writers require exclusive access
- How starvation can occur
- How a turnstile can improve fairness
- How synchronization correctness can be measured and verified

More importantly, the program makes the synchronization behaviour **visible** — not just through code, but through live execution, waiting times, statistics, and invariant checks.

---

## 👨‍💻 Author

**H Nidhish Sheka**

B.Tech — Information Science Engineering

> *Operating Systems assignment — Readers–Writers Problem*
